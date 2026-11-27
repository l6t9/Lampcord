use crate::ui::reply;
use crate::utils::backoff::Backoff;
use futures_util::future;
use std::collections::HashMap;
use std::path::PathBuf;
use std::slice;
use std::sync::{Arc, Mutex};
use tokio::time;

use crate::models::id::marker::ChannelMarker;
use crate::models::{Id, Message};
use crate::rest::endpoints::attachments::{self, MessageAttachmentRef, UploadFileRequest};
use crate::rest::{HttpClient, RestError, UploadStatus};

#[derive(Debug, Clone, PartialEq)]
pub struct UploadFile {
    pub local_id: u64,
    pub path: PathBuf,
    pub filename: String,
    pub size: u64,
    pub spoiler: bool,

    pub description: Option<String>,

    pub uploaded_filename: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UploadProgress {
    pub sent: u64,
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UploadError {
    pub reason: String,

    pub uploaded: Vec<(u64, String)>,
}

pub fn describe_error(error: &RestError) -> String {
    match error {
        RestError::Status { status, body, .. } => {
            let message = serde_json::from_str::<serde_json::Value>(body)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .and_then(|m| m.as_str().map(str::to_string))
                });
            match (message, status.as_u16()) {
                (Some(message), _) => message,
                (None, 413) => requiem_i18n::t!("uploads-the-file-is-too-large").to_string(),
                (None, code) => requiem_i18n::t!("uploads-server-rejected", code = code),
            }
        }
        RestError::Io(err) => requiem_i18n::t!("uploads-read-failed", error = err.to_string()),
        RestError::Request(err) => {
            requiem_i18n::t!("uploads-network-error", error = err.to_string())
        }
        other => other.to_string(),
    }
}

pub fn content_type_for(filename: &str) -> Option<&'static str> {
    let ext = filename.rsplit_once('.')?.1.to_ascii_lowercase();
    Some(match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "mkv" => "video/x-matroska",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "txt" | "log" => "text/plain",
        "md" => "text/markdown",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",
        "gz" => "application/gzip",
        "tar" => "application/x-tar",
        _ => return None,
    })
}

const MAX_UPLOAD_ATTEMPTS: u32 = 13;

async fn upload_one(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    index: usize,
    file: &UploadFile,
    on_sent: Arc<dyn Fn(u64) + Send + Sync>,
) -> Result<String, RestError> {
    let request = UploadFileRequest {
        id: index.to_string(),
        filename: file.filename.clone(),
        file_size: file.size,
        is_clip: false,
        original_content_type: content_type_for(&file.filename).map(str::to_string),
    };
    let slot = attachments::create_upload_urls(http, channel_id, slice::from_ref(&request))
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| RestError::Status {
            method: wreq::Method::POST,
            url: format!("/channels/{channel_id}/attachments"),
            status: wreq::StatusCode::BAD_GATEWAY,
            body: String::new(),
        })?;
    on_sent(0);
    let mut backoff = Backoff::new();
    let mut from = 0;
    let mut last_error = None;
    for attempt in 1..=MAX_UPLOAD_ATTEMPTS {
        if attempt > 1 {
            match http.upload_status(&slot.upload_url).await {
                Ok(UploadStatus::Complete) => return Ok(slot.upload_filename),
                Ok(UploadStatus::ResumeFrom(received)) => from = received.min(file.size),
                Err(err) => {
                    log::warn!(
                        "couldn't ask how much of {} was stored: {err}",
                        file.filename
                    );
                    last_error = Some(err);
                    time::sleep(backoff.fail(None)).await;
                    continue;
                }
            }
        }
        let progress = on_sent.clone();
        match http
            .put_file(&slot.upload_url, &file.path, file.size, from, move |sent| {
                progress(sent)
            })
            .await
        {
            Ok(()) => return Ok(slot.upload_filename),
            Err(err @ RestError::Status { .. }) if matches!(&err, RestError::Status { status, .. } if status.is_client_error()) =>
            {
                return Err(err);
            }
            Err(err @ RestError::Io(_)) => return Err(err),
            Err(err) => {
                log::warn!(
                    "upload of {} failed (attempt {attempt}/{MAX_UPLOAD_ATTEMPTS}): {err}",
                    file.filename
                );
                last_error = Some(err);
                time::sleep(backoff.fail(None)).await;
            }
        }
    }
    Err(last_error.expect("at least one attempt ran"))
}

pub async fn upload_and_send(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    content: &str,
    nonce: &str,
    files: &[UploadFile],
    on_progress: Arc<dyn Fn(UploadProgress) + Send + Sync>,
) -> Result<Message, UploadError> {
    let total: u64 = files
        .iter()
        .filter(|f| f.uploaded_filename.is_none())
        .map(|f| f.size)
        .sum();
    let sent_per_file: Arc<Mutex<HashMap<u64, u64>>> = Arc::default();
    let uploaded: Arc<Mutex<Vec<(u64, String)>>> = Arc::new(Mutex::new(
        files
            .iter()
            .filter_map(|f| f.uploaded_filename.clone().map(|n| (f.local_id, n)))
            .collect(),
    ));

    let uploads = files
        .iter()
        .enumerate()
        .filter(|(_, f)| f.uploaded_filename.is_none())
        .map(|(index, file)| {
            let on_sent: Arc<dyn Fn(u64) + Send + Sync> = {
                let (sent_per_file, on_progress, local_id) =
                    (sent_per_file.clone(), on_progress.clone(), file.local_id);
                Arc::new(move |sent| {
                    let sent = {
                        let mut map = sent_per_file.lock().expect("progress map poisoned");
                        map.insert(local_id, sent);
                        map.values().sum()
                    };
                    on_progress(UploadProgress { sent, total });
                })
            };
            let uploaded = uploaded.clone();
            async move {
                let name = upload_one(http, channel_id, index, file, on_sent).await?;
                uploaded
                    .lock()
                    .expect("uploaded list poisoned")
                    .push((file.local_id, name));
                Ok::<_, RestError>(())
            }
        });
    let fail = |error: &RestError| UploadError {
        reason: describe_error(error),
        uploaded: uploaded.lock().expect("uploaded list poisoned").clone(),
    };
    if let Err(error) = future::try_join_all(uploads).await {
        return Err(fail(&error));
    }

    let names: HashMap<u64, String> = uploaded
        .lock()
        .expect("uploaded list poisoned")
        .iter()
        .cloned()
        .collect();
    let refs: Vec<MessageAttachmentRef> = files
        .iter()
        .enumerate()
        .map(|(i, f)| MessageAttachmentRef {
            id: i.to_string(),
            filename: f.filename.clone(),
            uploaded_filename: names.get(&f.local_id).cloned().unwrap_or_default(),
            is_spoiler: f.spoiler,
            description: f.description.clone().filter(|d| !d.trim().is_empty()),
            original_content_type: content_type_for(&f.filename).map(str::to_string),
        })
        .collect();
    let reply = reply::take_outgoing(channel_id);
    attachments::send_message_with_attachments(
        http,
        channel_id,
        content,
        nonce,
        &refs,
        reply.as_ref(),
    )
    .await
    .map_err(|e| fail(&e))
}

#[cfg(test)]
mod tests {
    use crate::instances::Instance;
    use std::env;
    use std::fs;
    use std::io::Error;
    use std::io::ErrorKind;
    use std::process;
    use std::sync::Mutex;
    use tokio::net::TcpListener;
    use tokio::task;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;
    use crate::auth::AuthCoordinator;
    use crate::config::BackendConfig;

    fn offline_client() -> HttpClient {
        let backend = BackendConfig {
            rest_base_url: String::new(),
            gateway_url: String::new(),
            cdn_base_url: String::new(),
        };
        HttpClient::new(
            backend.clone(),
            AuthCoordinator::new(backend, "secret-token".to_string()),
        )
        .expect("client")
    }

    async fn one_shot_server(
        status_line: &'static str,
    ) -> (String, task::JoinHandle<(String, Vec<u8>)>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/upload?sig=abc", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut received = Vec::new();
            let mut buffer = [0u8; 8192];
            let (head_end, length) = loop {
                let n = socket.read(&mut buffer).await.unwrap();
                received.extend_from_slice(&buffer[..n]);
                if let Some(pos) = received.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&received[..pos]).to_lowercase();
                    let length = head
                        .lines()
                        .find_map(|l| l.strip_prefix("content-length: "))
                        .and_then(|v| v.trim().parse::<usize>().ok())
                        .unwrap_or(0);
                    break (pos + 4, length);
                }
            };
            while received.len() < head_end + length {
                let n = socket.read(&mut buffer).await.unwrap();
                received.extend_from_slice(&buffer[..n]);
            }
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 {status_line}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
            (
                String::from_utf8_lossy(&received[..head_end]).to_lowercase(),
                received[head_end..].to_vec(),
            )
        });
        (url, handle)
    }

    #[tokio::test]
    async fn put_file_streams_the_whole_file_with_progress_and_no_auth_header() {
        let path = env::temp_dir().join(format!("upload-test-{}.bin", process::id()));
        let data: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        fs::write(&path, &data).unwrap();

        let (url, server) = one_shot_server("200 OK").await;
        let progress = Arc::new(Mutex::new(Vec::new()));
        let sink = progress.clone();
        offline_client()
            .put_file(&url, &path, data.len() as u64, 0, move |sent| {
                sink.lock().unwrap().push(sent)
            })
            .await
            .unwrap();
        let (head, body) = server.await.unwrap();
        fs::remove_file(&path).ok();

        assert_eq!(body, data);
        assert!(
            head.contains(&format!("content-length: {}", data.len())),
            "{head}"
        );
        assert!(
            !head.contains("authorization"),
            "the storage host must not get our token: {head}"
        );
        let progress = progress.lock().unwrap();
        assert!(progress.len() > 1, "progress is reported in chunks");
        assert!(progress.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(*progress.last().unwrap(), data.len() as u64);
    }

    #[tokio::test]
    async fn a_resumed_put_sends_the_rest_with_a_content_range() {
        let path =
            env::temp_dir().join(format!("upload-test-resume-{}.bin", process::id()));
        let data: Vec<u8> = (0..1000u32).map(|i| (i % 251) as u8).collect();
        fs::write(&path, &data).unwrap();
        let (url, server) = one_shot_server("200 OK").await;
        offline_client()
            .put_file(&url, &path, 1000, 400, |_| {})
            .await
            .unwrap();
        let (head, body) = server.await.unwrap();
        fs::remove_file(&path).ok();
        assert_eq!(body, data[400..]);
        assert!(head.contains("content-length: 600"), "{head}");
        assert!(head.contains("content-range: bytes 400-999/1000"), "{head}");
        assert!(
            head.contains("content-type: application/octet-stream"),
            "{head}"
        );
    }

    #[tokio::test]
    async fn the_status_probe_is_a_bodyless_put_that_reads_the_range_header() {
        let (url, server) = one_shot_server("308 Resume Incomplete\r\nrange: bytes=0-99").await;
        let status = offline_client().upload_status(&url).await.unwrap();
        let (head, body) = server.await.unwrap();
        assert_eq!(status, UploadStatus::ResumeFrom(100));
        assert!(body.is_empty());
        assert!(head.starts_with("put "), "{head}");
        assert!(
            head.contains("content-range: bytes */*") && head.contains("content-length: 0"),
            "{head}"
        );
        assert!(
            !head.contains("content-type"),
            "the probe has no content type: {head}"
        );

        let (url, server) = one_shot_server("308 Resume Incomplete").await;
        assert_eq!(
            offline_client().upload_status(&url).await.unwrap(),
            UploadStatus::ResumeFrom(0),
            "no Range header: nothing is stored"
        );
        server.await.unwrap();
        let (url, server) = one_shot_server("201 Created").await;
        assert_eq!(
            offline_client().upload_status(&url).await.unwrap(),
            UploadStatus::Complete
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn put_file_reports_a_rejected_upload() {
        let path =
            env::temp_dir().join(format!("upload-test-rejected-{}.bin", process::id()));
        fs::write(&path, b"tiny").unwrap();
        let (url, server) = one_shot_server("403 Forbidden").await;
        let error = offline_client()
            .put_file(&url, &path, 4, 0, |_| {})
            .await
            .unwrap_err();
        server.await.unwrap();
        fs::remove_file(&path).ok();
        assert!(
            matches!(error, RestError::Status { status, .. } if status.as_u16() == 403),
            "{error:?}"
        );
        assert!(!describe_error(&error).is_empty());
    }

    #[test]
    fn describes_server_and_io_errors() {
        let body = |b: &str| RestError::Status {
            method: wreq::Method::POST,
            url: "u".into(),
            status: wreq::StatusCode::BAD_REQUEST,
            body: b.into(),
        };
        assert_eq!(
            describe_error(&body(
                r#"{"message":"Request entity too large","code":40005}"#
            )),
            "Request entity too large"
        );
        let too_big = RestError::Status {
            method: wreq::Method::PUT,
            url: "u".into(),
            status: wreq::StatusCode::PAYLOAD_TOO_LARGE,
            body: String::new(),
        };
        assert_eq!(describe_error(&too_big), "The file is too large.");
        assert!(describe_error(&body("not json")).contains("400"));
        assert!(
            describe_error(&RestError::Io(Error::new(
                ErrorKind::NotFound,
                "gone"
            )))
            .contains("gone")
        );
    }

    #[tokio::test]
    #[ignore]
    async fn live_upload_and_send_round_trip() {
        let backend = Instance::official()
            .backend()
            .expect("official");
        let token = env::var("REQUIEM_TEST_TOKEN").expect("REQUIEM_TEST_TOKEN");
        let channel: u64 = env::var("REQUIEM_TEST_CHANNEL")
            .expect("REQUIEM_TEST_CHANNEL")
            .parse()
            .unwrap();
        let http = HttpClient::new(backend.clone(), AuthCoordinator::new(backend, token)).unwrap();
        let path = env::temp_dir().join("live-upload.txt");
        fs::write(&path, b"uploaded by the client's own upload code\n").unwrap();
        let file = UploadFile {
            local_id: 1,
            path: path.clone(),
            filename: "live-upload.txt".into(),
            size: fs::metadata(&path).unwrap().len(),
            spoiler: false,
            description: None,
            uploaded_filename: None,
        };
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = events.clone();
        let message = upload_and_send(
            &http,
            Id::new(channel),
            "live upload test",
            "1",
            &[file],
            Arc::new(move |e| sink.lock().unwrap().push(e)),
        )
        .await
        .expect("upload and send");
        println!(
            "sent message {} with {} attachment(s): {:?}",
            message.id,
            message.attachments.len(),
            events.lock().unwrap()
        );
        assert_eq!(message.attachments.len(), 1);
        assert_eq!(message.attachments[0].filename, "live-upload.txt");
    }
}
