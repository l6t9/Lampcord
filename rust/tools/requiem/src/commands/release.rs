use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use requiem_update::{
    Artifact, KIND_IDENTITY, KIND_UPDATE, KeyEntry, KeySet, Signed, SigningKey, UpdateManifest,
    fingerprint, sha256_hex, sign_key_from_seed,
};
use reqwest::Client;
use reqwest::Url;
use semver::Version;
use std::env;
use std::fs;
use std::time::Duration;
use tokio::time;

use crate::util::{ok, step, warn};

const KEY_ENV: &str = "REQUIEM_SIGNING_KEY";

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn year(secs: u64) -> u64 {
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(month <= 2)) as u64
}

fn signing_key() -> Result<SigningKey> {
    let seed = env::var(KEY_ENV).with_context(|| format!("{KEY_ENV} is not set (base64 of the 32-byte signing seed; `cargo requiem release keygen` makes one)"))?;
    let bytes: [u8; 32] = STANDARD
        .decode(seed.trim())
        .ok()
        .and_then(|b| b.try_into().ok())
        .with_context(|| format!("{KEY_ENV} is not 32 bytes of base64"))?;
    Ok(sign_key_from_seed(&bytes))
}

pub fn keygen(id: Option<&str>, out: &Path) -> Result<i32> {
    let id = id
        .map(String::from)
        .unwrap_or_else(|| format!("primary-{}", year(now())));
    let id = id.as_str();
    if out.exists() {
        bail!(
            "{} exists already; refusing to overwrite a signing key",
            out.display()
        );
    }
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).map_err(|e| anyhow::anyhow!("no randomness: {e}"))?;
    fs::write(out, STANDARD.encode(seed)).with_context(|| format!("writing {}", out.display()))?;
    let key = sign_key_from_seed(&seed).verifying_key();
    step(&format!("Key `{id}` written to {}", out.display()));
    ok("keep that file secret; for CI put its contents in the REQUIEM_SIGNING_KEY secret");
    ok(&format!(
        "public key entry: {}",
        serde_json::to_string(&KeyEntry::new(id, &key))?
    ));
    ok(&format!("fingerprint:      {}", fingerprint(&key)));
    warn("the spare key's file belongs offline, not in CI");
    Ok(0)
}

pub struct SignArgs {
    pub zip: PathBuf,
    pub version: String,
    pub channel: String,
    pub target: String,
    pub key_id: String,

    pub bases: Vec<String>,

    pub artifact_urls: Vec<String>,
    pub serial: Option<u64>,
    pub valid_days: u64,
    pub notes_url: Option<String>,
    pub revoked: Vec<String>,
    pub out: PathBuf,
}

pub fn sign(args: SignArgs) -> Result<i32> {
    let key = signing_key()?;
    let bytes = fs::read(&args.zip).with_context(|| format!("reading {}", args.zip.display()))?;
    let file = args
        .zip
        .file_name()
        .and_then(|n| n.to_str())
        .context("zip has no file name")?;
    if args.bases.is_empty() && args.artifact_urls.is_empty() {
        bail!("give at least one --base or --artifact-url");
    }
    let version_text = args.version.trim_start_matches('v').to_string();
    let urls: Vec<String> = args
        .bases
        .iter()
        .map(|b| {
            format!(
                "{}/v1/releases/{version_text}/{file}",
                b.trim_end_matches('/')
            )
        })
        .chain(args.artifact_urls.iter().cloned())
        .collect();
    let t = now();
    let manifest = UpdateManifest {
        schema: 1,
        channel: args.channel,
        serial: args.serial.unwrap_or(t),
        version: args.version.trim_start_matches('v').to_string(),
        released_at: t,
        expires_at: t + args.valid_days * 86_400,
        revoked: args.revoked,
        notes_url: args.notes_url,
        sources: args.bases.clone(),
        artifacts: vec![Artifact {
            target: args.target,
            sha256: sha256_hex(&bytes),
            size: bytes.len() as u64,
            urls,
        }],
    };
    Version::parse(&manifest.version)
        .with_context(|| format!("`{}` is not a semantic version", manifest.version))?;
    write_signed(
        &args.out,
        &Signed::sign(KIND_UPDATE, &manifest, &args.key_id, &key)?,
    )?;
    ok(&format!(
        "{} v{} ({} bytes), serial {}, valid {} days -> {}",
        manifest.channel,
        manifest.version,
        manifest.artifacts[0].size,
        manifest.serial,
        args.valid_days,
        args.out.display()
    ));
    Ok(0)
}

pub fn resign(input: &Path, key_id: &str, valid_days: u64, out: &Path) -> Result<i32> {
    let key = signing_key()?;
    let keys = KeySet::from_entries([KeyEntry::new(key_id, &key.verifying_key())])?;
    let envelope = Signed::parse(
        &fs::read_to_string(input).with_context(|| format!("reading {}", input.display()))?,
    )?;
    let mut manifest: UpdateManifest = envelope
        .open(&keys, KIND_UPDATE)
        .context("the existing feed does not verify with this key")?;
    let t = now();
    manifest.serial = t.max(manifest.serial + 1);
    manifest.expires_at = t + valid_days * 86_400;
    write_signed(out, &Signed::sign(KIND_UPDATE, &manifest, key_id, &key)?)?;
    ok(&format!(
        "v{} re-signed, serial {}, valid {} days -> {}",
        manifest.version,
        manifest.serial,
        valid_days,
        out.display()
    ));
    Ok(0)
}

pub fn identity(input: PathBuf, key_id: &str, out: &Path) -> Result<i32> {
    let key = signing_key()?;
    let text =
        fs::read_to_string(&input).with_context(|| format!("reading {}", input.display()))?;
    serde_json::from_str::<serde_json::Value>(&text)
        .with_context(|| format!("{} is not valid JSON", input.display()))?;
    write_signed(out, &Signed::sign_text(KIND_IDENTITY, text, key_id, &key))?;
    ok(&format!("identity signed -> {}", out.display()));
    Ok(0)
}

fn answer_text(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| {
            v.get("message")
                .or_else(|| v.get("error"))
                .and_then(|m| m.as_str().map(String::from))
        })
        .unwrap_or_else(|| body.chars().take(200).collect())
}

async fn bearer(client: &Client, token_env: Option<&str>, audience: &str) -> Result<String> {
    if let Some(name) = token_env {
        return env::var(name).with_context(|| format!("{name} is not set"));
    }
    let url = env::var("ACTIONS_ID_TOKEN_REQUEST_URL").context("not in a GitHub Actions job with `permissions: id-token: write` (ACTIONS_ID_TOKEN_REQUEST_URL is not set); use --token-env for a local test")?;
    let request_token = env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN")
        .context("ACTIONS_ID_TOKEN_REQUEST_TOKEN is not set")?;
    let url = Url::parse_with_params(&url, [("audience", audience)])?;
    let body: serde_json::Value = client
        .get(url)
        .bearer_auth(request_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(body["value"]
        .as_str()
        .context("GitHub's answer has no token")?
        .to_string())
}

async fn put(
    client: &Client,
    token_env: Option<&str>,
    base: &str,
    path: &str,
    body: Vec<u8>,
) -> Result<(u16, String)> {
    let url = format!("{base}{path}");
    let mut last = String::new();
    for attempt in 1..=4 {
        let token = bearer(client, token_env, base).await?;
        match client
            .put(&url)
            .bearer_auth(token)
            .body(body.clone())
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status().as_u16();
                let text = response.text().await.unwrap_or_default();
                if status < 500 {
                    return Ok((status, answer_text(&text)));
                }
                last = format!("HTTP {status}: {}", answer_text(&text));
            }
            Err(error) => last = error.to_string(),
        }
        warn(&format!("{path}: {last} (attempt {attempt} of 4)"));
        time::sleep(Duration::from_secs(2 * attempt)).await;
    }
    bail!("{path}: giving up: {last}")
}

fn expect_ok(path: &str, result: (u16, String)) -> Result<()> {
    let (status, message) = result;
    if !(200..300).contains(&status) {
        bail!("{path}: the server refused ({status}): {message}");
    }
    ok(&format!("{path}: {message}"));
    Ok(())
}

fn files_under(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            files_under(&path, out)?
        } else {
            out.push(path)
        }
    }
    Ok(())
}

pub async fn publish(
    dir: &Path,
    base: &str,
    token_env: Option<&str>,
    part_size: u64,
) -> Result<i32> {
    let base = base.trim_end_matches('/');
    let mut files = Vec::new();
    files_under(&dir.join("v1"), &mut files)?;

    let mut items: Vec<(String, PathBuf)> = files
        .into_iter()
        .map(|p| {
            (
                format!(
                    "/{}",
                    p.strip_prefix(dir)
                        .expect("under dir")
                        .to_string_lossy()
                        .replace('\\', "/")
                ),
                p,
            )
        })
        .collect();

    items.retain(|(path, _)| path != "/v1/keys.json");
    items.sort_by_key(|(path, _)| {
        if path.starts_with("/v1/releases/") {
            0
        } else {
            1
        }
    });
    if items.is_empty() {
        bail!("nothing to upload under {}", dir.join("v1").display());
    }
    let client = Client::builder()
        .timeout(Duration::from_secs(600))
        .build()?;
    step(&format!("Publishing {} files to {base}", items.len()));

    for (path, file) in items {
        let bytes = fs::read(&file).with_context(|| format!("reading {}", file.display()))?;
        if !path.starts_with("/v1/releases/") {
            expect_ok(&path, put(&client, token_env, base, &path, bytes).await?)?;
            continue;
        }
        let (size, sha256) = (bytes.len() as u64, sha256_hex(&bytes));
        let parts = size.div_ceil(part_size).max(1) as u32;
        let complete = serde_json::to_vec(
            &serde_json::json!({ "parts": parts, "size": size, "sha256": sha256 }),
        )?;

        let head = client.head(format!("{base}{path}")).send().await?;

        let on_server = head.status().is_success()
            && head
                .headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                == Some(size);
        if on_server {
            expect_ok(
                &path,
                put(
                    &client,
                    token_env,
                    base,
                    &format!("{path}/complete"),
                    complete,
                )
                .await?,
            )?;
            continue;
        }
        if size <= part_size {
            expect_ok(&path, put(&client, token_env, base, &path, bytes).await?)?;
            continue;
        }
        for (index, chunk) in bytes.chunks(part_size as usize).enumerate() {
            expect_ok(
                &format!("{path} part {}/{parts}", index + 1),
                put(
                    &client,
                    token_env,
                    base,
                    &format!("{path}/parts/{index}"),
                    chunk.to_vec(),
                )
                .await?,
            )?;
        }
        expect_ok(
            &path,
            put(
                &client,
                token_env,
                base,
                &format!("{path}/complete"),
                complete,
            )
            .await?,
        )?;
    }
    Ok(0)
}

pub fn keys_json(out: &Path) -> Result<i32> {
    let keys = requiem_update::official_keys();
    let entries: Vec<KeyEntry> = keys
        .ids()
        .filter_map(|id| keys.get(id).map(|key| KeyEntry::new(id, key)))
        .collect();
    if entries.is_empty() {
        bail!("the client has no official keys compiled in");
    }
    if let Some(dir) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir)?;
    }
    fs::write(
        out,
        serde_json::to_string_pretty(&requiem_update::KeysDocument {
            schema: 1,
            keys: entries.clone(),
        })?,
    )
    .with_context(|| format!("writing {}", out.display()))?;
    for entry in &entries {
        ok(&format!(
            "{}  {}",
            entry.id,
            fingerprint(&entry.verifying_key()?)
        ));
    }
    Ok(0)
}

pub fn verify(file: &Path, key_id: &str, public: &str) -> Result<i32> {
    let keys = KeySet::from_entries([KeyEntry {
        id: key_id.to_string(),
        public: public.to_string(),
    }])?;
    let envelope = Signed::parse(
        &fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?,
    )?;
    let manifest: UpdateManifest = envelope.open(&keys, KIND_UPDATE)?;
    ok(&format!(
        "signature valid: {} v{}, serial {}, expires {}",
        manifest.channel, manifest.version, manifest.serial, manifest.expires_at
    ));
    Ok(0)
}

fn write_signed(out: &Path, signed: &Signed) -> Result<()> {
    if let Some(dir) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir)?;
    }
    fs::write(out, signed.to_json()).with_context(|| format!("writing {}", out.display()))
}

#[cfg(test)]
mod tests {
    use super::year;

    #[test]
    fn years_come_out_right_around_boundaries() {
        assert_eq!(year(0), 1970);
        assert_eq!(year(951_782_400), 2000);
        assert_eq!(year(1_798_761_599), 2026);
        assert_eq!(year(1_798_761_600), 2027);
        assert_eq!(year(1_792_181_471), 2026);
    }
}
