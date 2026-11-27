use crate::images;
use crate::models;
use crate::ui::classes::attachments;
use crate::ui::classes::attachments as css;
use crate::ui::media_viewer;
use crate::ui;
use crate::utils::time;
#[cfg(feature = "video")]
use crate::video_thumb;
use requiem_ui::clipboard;
use requiem_ui::colors::TEXT_CRITICAL;
use requiem_ui::components::StackProps;
use requiem_ui::components::TextVariant;
use requiem_ui::components::forget_progress;
use requiem_ui::components::{ButtonVariant, ModalSize};
use requiem_ui::icons::IconPath;
use requiem_ui::widget::anim::ToggleMemo;
use requiem_ui::widget::primitives::ClickAreaWidget;
use skia_safe::images as skia_safe_images;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::path;
use std::process;
use std::rc::Rc;
use std::sync::Arc;
use std::time as std_time;
use std::time::Duration;
use std::time::SystemTime;

use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::ToggleState;
use skia_safe::Image;

use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::rest::endpoints::attachments::MAX_ATTACHMENTS_PER_MESSAGE;
use crate::uploads::{UploadFile, UploadProgress};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileKind {
    Image,
    Video,
    Acrobat,
    Ae,
    Sketch,
    Ai,
    Archive,
    Code,
    Document,
    Photoshop,
    Spreadsheet,
    Webcode,
    Audio,
    Unknown,
}

pub fn file_kind(filename: &str) -> FileKind {
    let name = filename.to_lowercase();
    let ends = |exts: &[&str]| exts.iter().any(|e| name.ends_with(&format!(".{e}")));

    if ends(&["psd"]) {
        return FileKind::Photoshop;
    }
    if ends(&["svg"]) {
        return FileKind::Webcode;
    }
    if ends(&[
        "png", "jpg", "jpeg", "gif", "webp", "bmp", "apng", "avif", "tif", "tiff", "ico", "heic",
    ]) {
        return FileKind::Image;
    }
    if ends(&[
        "mp4", "webm", "mov", "mkv", "avi", "m4v", "wmv", "mpg", "mpeg", "ogv",
    ]) {
        return FileKind::Video;
    }
    if ends(&["pdf"]) {
        FileKind::Acrobat
    } else if name.contains(".ae") {
        FileKind::Ae
    } else if ends(&["sketch"]) {
        FileKind::Sketch
    } else if ends(&["ai"]) {
        FileKind::Ai
    } else if ends(&["rar", "zip", "7z", "tar", "tar.gz"]) {
        FileKind::Archive
    } else if ends(&[
        "c++",
        "cpp",
        "cc",
        "c",
        "h",
        "hpp",
        "mm",
        "m",
        "json",
        "js",
        "ts",
        "rb",
        "rake",
        "py",
        "asm",
        "fs",
        "pyc",
        "dtd",
        "cgi",
        "bat",
        "rss",
        "java",
        "graphml",
        "idb",
        "lua",
        "o",
        "gml",
        "prl",
        "sls",
        "conf",
        "cmake",
        "make",
        "sln",
        "vbe",
        "cxx",
        "wbf",
        "vbs",
        "r",
        "wml",
        "php",
        "bash",
        "applescript",
        "fcgi",
        "yaml",
        "ex",
        "exs",
        "sh",
        "ml",
        "actionscript",
    ]) {
        FileKind::Code
    } else if ends(&[
        "txt", "rtf", "doc", "docx", "md", "pages", "ppt", "pptx", "pptm", "key", "log",
    ]) {
        FileKind::Document
    } else if ends(&["xls", "xlsx", "numbers", "csv"]) {
        FileKind::Spreadsheet
    } else if ends(&["html", "xhtml", "htm", "xml", "xsd", "css", "styl"]) {
        FileKind::Webcode
    } else if ends(&["mp3", "ogg", "opus", "wav", "aiff", "flac"]) {
        FileKind::Audio
    } else {
        FileKind::Unknown
    }
}

pub fn icon_asset(kind: FileKind) -> Option<&'static str> {
    Some(match kind {
        FileKind::Image => return None,
        FileKind::Video => "file_icon/video",
        FileKind::Acrobat => "file_icon/acrobat",
        FileKind::Ae => "file_icon/ae",
        FileKind::Sketch => "file_icon/sketch",
        FileKind::Ai => "file_icon/ai",
        FileKind::Archive => "file_icon/archive",
        FileKind::Code => "file_icon/code",
        FileKind::Document => "file_icon/document",
        FileKind::Photoshop => "file_icon/photoshop",
        FileKind::Spreadsheet => "file_icon/spreadsheet",
        FileKind::Webcode => "file_icon/webcode",
        FileKind::Audio => "file_icon/audio",
        FileKind::Unknown => "file_icon/unknown",
    })
}

const ICON_PX: (u32, u32) = (144, 192);

thread_local! {
    static ICONS: RefCell<HashMap<FileKind, Option<Image>>> = RefCell::new(HashMap::new());
}

pub fn icon_image(kind: FileKind) -> Option<Image> {
    ICONS.with(|icons| {
        icons
            .borrow_mut()
            .entry(kind)
            .or_insert_with(|| {
                icon_asset(kind).and_then(|name| ui::asset_svg(name, ICON_PX.0, ICON_PX.1))
            })
            .clone()
    })
}

pub fn card_icon(kind: FileKind) -> Option<Image> {
    icon_image(if kind == FileKind::Image {
        FileKind::Unknown
    } else {
        kind
    })
}

const MAX_THUMBNAIL_BYTES: u64 = 30 * 1024 * 1024;

#[cfg(feature = "video")]
fn video_thumbnail(path: &path::Path) -> Option<Image> {
    video_thumb::first_frame(path)
        .map_err(|e| log::warn!("no thumbnail for {}: {e}", path.display()))
        .ok()
}

#[cfg(not(feature = "video"))]
fn video_thumbnail(_path: &path::Path) -> Option<Image> {
    None
}

#[derive(Clone)]
pub struct PendingAttachment {
    pub id: u64,
    pub path: PathBuf,

    pub filename: String,
    pub size: u64,
    pub kind: FileKind,

    pub thumbnail: Option<Image>,
    pub spoiler: bool,

    pub description: String,

    pub uploaded_filename: Option<String>,
}

impl PendingAttachment {
    fn to_upload(&self) -> UploadFile {
        UploadFile {
            local_id: self.id,
            path: self.path.clone(),
            filename: self.filename.clone(),
            size: self.size,
            spoiler: self.spoiler,
            description: Some(self.description.clone()).filter(|d| !d.trim().is_empty()),
            uploaded_filename: self.uploaded_filename.clone(),
        }
    }
}

#[derive(Clone)]
pub struct PendingUpload {
    pub nonce: u64,
    pub channel: Id<ChannelMarker>,
    pub content: String,
    pub files: Vec<PendingAttachment>,
    pub sent: u64,

    pub total: u64,

    pub created_ms: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FailureNotice {
    pub id: u64,
    pub channel: Id<ChannelMarker>,

    pub after: Option<Id<MessageMarker>>,

    pub time: String,
}

#[derive(Debug, Default, PartialEq)]
pub struct AddResult {
    pub added: usize,

    pub over_limit: usize,

    pub unreadable: Vec<String>,

    pub empty: Vec<String>,
}

pub const MENU_DOM_ID: &str = "attach-menu";
pub const MENU_BUTTON_DOM_ID: &str = "composer-attach-btn";

#[derive(Clone)]
pub struct AttachmentsBackend {
    pub pick_files: Rc<dyn Fn(Id<ChannelMarker>)>,

    pub send: Rc<dyn Fn(Id<ChannelMarker>, u64, String, Vec<UploadFile>)>,

    pub cancel: Rc<dyn Fn(u64)>,

    pub delete_upload: Rc<dyn Fn(String)>,
}

impl AttachmentsBackend {
    pub fn inert() -> Self {
        Self {
            pick_files: Rc::new(|_| {}),
            send: Rc::new(|_, _, _, _| {}),
            cancel: Rc::new(|_| {}),
            delete_upload: Rc::new(|_| {}),
        }
    }
}

#[derive(Clone)]
pub struct ModifyState {
    pub target: Signal<Option<(Id<ChannelMarker>, u64)>>,
    pub open: Signal<ToggleState>,
    pub filename: Rc<RefCell<String>>,
    pub description: Rc<RefCell<String>>,
    pub spoiler: Signal<bool>,

    pub spoiler_toggle: ToggleMemo,

    pub sent: Signal<Option<String>>,
    pub sent_save: Rc<RefCell<Option<Rc<dyn Fn(String, String)>>>>,
}

#[derive(Clone)]
pub struct AttachmentsState {
    pub pending: Signal<HashMap<Id<ChannelMarker>, Vec<PendingAttachment>>>,

    pub uploads: Signal<Vec<PendingUpload>>,
    pub notices: Signal<Vec<FailureNotice>>,

    pub menu_open: Signal<bool>,

    pub drag_hover: Signal<bool>,

    pub shift_held: Signal<bool>,
    pub modify: ModifyState,

    pub too_many_open: Signal<ToggleState>,
    next_id: Rc<Cell<u64>>,

    tray_scroll: Rc<RefCell<HashMap<Id<ChannelMarker>, Rc<Cell<f32>>>>>,
    pub backend: AttachmentsBackend,
}

impl AttachmentsState {
    pub fn new(backend: AttachmentsBackend) -> Self {
        Self {
            pending: signal(HashMap::new()),
            uploads: signal(Vec::new()),
            notices: signal(Vec::new()),
            menu_open: signal(false),
            drag_hover: signal(false),
            shift_held: signal(false),
            modify: ModifyState {
                target: signal(None),
                open: signal(ToggleState::new(false)),
                filename: Rc::default(),
                description: Rc::default(),
                spoiler: signal(false),
                spoiler_toggle: ToggleMemo::new(false),
                sent: signal(None),
                sent_save: Rc::default(),
            },
            too_many_open: signal(ToggleState::new(false)),
            next_id: Rc::new(Cell::new(1)),
            tray_scroll: Rc::default(),
            backend,
        }
    }

    pub fn inert() -> Self {
        Self::new(AttachmentsBackend::inert())
    }

    fn next_id(&self) -> u64 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }

    pub fn count(&self, channel: Id<ChannelMarker>) -> usize {
        self.pending
            .with_untracked(|all| all.get(&channel).map_or(0, Vec::len))
    }

    pub fn add_paths(&self, channel: Id<ChannelMarker>, paths: Vec<PathBuf>) -> AddResult {
        let mut result = AddResult::default();
        let mut added = Vec::new();
        for path in paths {
            let filename = path
                .file_name()
                .map_or_else(|| "file".to_string(), |n| n.to_string_lossy().into_owned());
            let metadata = match fs::metadata(&path) {
                Ok(m) if m.is_file() && m.len() == 0 => {
                    result.empty.push(filename);
                    continue;
                }
                Ok(m) if m.is_file() => m,
                _ => {
                    result.unreadable.push(filename);
                    continue;
                }
            };
            added.push((path, filename, metadata.len()));
        }
        if self.count(channel) + added.len() > MAX_ATTACHMENTS_PER_MESSAGE {
            result.over_limit = added.len();
            self.too_many_open.set(ToggleState::new(false).set(true));
            return result;
        }
        let added: Vec<PendingAttachment> = added
            .into_iter()
            .map(|(path, filename, size)| {
                let kind = file_kind(&filename);
                let thumbnail = match kind {
                    FileKind::Image if size <= MAX_THUMBNAIL_BYTES => fs::read(&path)
                        .ok()
                        .and_then(|bytes| Image::from_encoded(skia_safe::Data::new_copy(&bytes))),
                    FileKind::Video => video_thumbnail(&path),
                    _ => None,
                };
                PendingAttachment {
                    id: self.next_id(),
                    path,
                    filename,
                    size,
                    kind,
                    thumbnail,
                    spoiler: false,
                    description: String::new(),
                    uploaded_filename: None,
                }
            })
            .collect();
        result.added = added.len();
        if !added.is_empty() {
            self.pending
                .update(|all| all.entry(channel).or_default().extend(added));
        }
        result
    }

    pub fn remove(&self, channel: Id<ChannelMarker>, id: u64) {
        let mut removed = None;
        self.pending.update(|all| {
            if let Some(list) = all.get_mut(&channel) {
                if let Some(index) = list.iter().position(|a| a.id == id) {
                    removed = Some(list.remove(index));
                }
                if list.is_empty() {
                    all.remove(&channel);
                }
            }
        });
        if let Some(name) = removed.and_then(|a| a.uploaded_filename) {
            (self.backend.delete_upload)(name);
        }
    }

    fn update_file(
        &self,
        channel: Id<ChannelMarker>,
        id: u64,
        f: impl FnOnce(&mut PendingAttachment),
    ) {
        self.pending.update(|all| {
            if let Some(a) = all
                .get_mut(&channel)
                .and_then(|list| list.iter_mut().find(|a| a.id == id))
            {
                f(a);
            }
        });
    }

    pub fn toggle_spoiler(&self, channel: Id<ChannelMarker>, id: u64) {
        self.update_file(channel, id, |a| a.spoiler = !a.spoiler);
    }

    pub fn open_modify(&self, channel: Id<ChannelMarker>, id: u64) {
        let Some(file) = self.pending.with_untracked(|all| {
            all.get(&channel)
                .and_then(|list| list.iter().find(|a| a.id == id).cloned())
        }) else {
            return;
        };
        *self.modify.filename.borrow_mut() = file.filename;
        *self.modify.description.borrow_mut() = file.description;
        self.modify.spoiler.set(file.spoiler);
        self.modify.spoiler_toggle.reset(file.spoiler);
        self.modify.sent.set(None);
        self.modify.target.set(Some((channel, id)));
        self.modify.open.set(ToggleState::new(false).set(true));
    }

    pub fn open_modify_sent(
        &self,
        attachment: &models::Attachment,
        on_save: Rc<dyn Fn(String, String)>,
    ) {
        let name = attachment
            .filename
            .strip_prefix("SPOILER_")
            .unwrap_or(&attachment.filename)
            .to_string();
        *self.modify.filename.borrow_mut() = name;
        *self.modify.description.borrow_mut() = attachment.description.clone().unwrap_or_default();
        self.modify.spoiler.set(attachment.is_spoiler());
        self.modify.spoiler_toggle.reset(attachment.is_spoiler());
        *self.modify.sent_save.borrow_mut() = Some(on_save);
        self.modify.target.set(None);
        self.modify.sent.set(Some(attachment.filename.clone()));
        self.modify.open.set(ToggleState::new(false).set(true));
    }

    pub fn close_modify(&self) {
        self.modify.open.update(|t| *t = t.set(false));
    }

    pub fn save_modify(&self) {
        if self.modify.sent.peek().is_some() {
            let filename = self.modify.filename.borrow().trim().to_string();
            let description = self.modify.description.borrow().clone();

            let filename = if self.modify.spoiler.peek() {
                format!("SPOILER_{filename}")
            } else {
                filename
            };
            if let Some(save) = self.modify.sent_save.borrow().clone() {
                save(filename, description);
            }
        } else if let Some((channel, id)) = self.modify.target.peek() {
            let filename = self.modify.filename.borrow().trim().to_string();
            let description = self.modify.description.borrow().clone();
            let spoiler = self.modify.spoiler.peek();
            self.update_file(channel, id, |a| {
                if !filename.is_empty() {
                    a.filename = filename;
                }
                a.description = description;
                a.spoiler = spoiler;
            });
        }
        self.close_modify();
    }

    pub fn close_too_many(&self) {
        self.too_many_open.update(|t| *t = t.set(false));
    }

    pub fn begin_send(
        &self,
        channel: Id<ChannelMarker>,
        content: String,
    ) -> Option<(u64, Vec<UploadFile>)> {
        let mut files = Vec::new();
        self.pending
            .update(|all| files = all.remove(&channel).unwrap_or_default());
        if files.is_empty() {
            return None;
        }
        let created_ms = time::now_ms() as i64;

        let nonce = time::snowflake_at(created_ms) | (self.next_id() & 0x3f_ffff);
        let uploads: Vec<UploadFile> = files.iter().map(PendingAttachment::to_upload).collect();
        let total = files
            .iter()
            .filter(|f| f.uploaded_filename.is_none())
            .map(|f| f.size)
            .sum();
        self.uploads.update(|list| {
            list.push(PendingUpload {
                nonce,
                channel,
                content,
                files,
                sent: 0,
                total,
                created_ms,
            })
        });
        Some((nonce, uploads))
    }

    pub fn progress(&self, nonce: u64, progress: UploadProgress) {
        self.uploads.update(|list| {
            if let Some(upload) = list.iter_mut().find(|u| u.nonce == nonce) {
                upload.sent = progress.sent;
                upload.total = progress.total;
            }
        });
    }

    fn take_upload(&self, nonce: u64) -> Option<PendingUpload> {
        let index = self
            .uploads
            .with_untracked(|list| list.iter().position(|u| u.nonce == nonce))?;
        let mut taken = None;
        self.uploads.update(|list| taken = Some(list.remove(index)));
        forget_progress(nonce);
        taken
    }

    pub fn sent(&self, nonce: u64) {
        self.take_upload(nonce);
    }

    pub fn message_created(&self, nonce: &str) {
        if let Ok(nonce) = nonce.parse::<u64>() {
            self.sent(nonce);
        }
    }

    pub fn cancelled(&self, nonce: u64) -> Option<(Id<ChannelMarker>, String)> {
        self.take_upload(nonce).map(|u| (u.channel, u.content))
    }

    pub fn failed(
        &self,
        nonce: u64,
        uploaded: &[(u64, String)],
        after: Option<Id<MessageMarker>>,
    ) -> Option<(Id<ChannelMarker>, String)> {
        let upload = self.take_upload(nonce)?;
        let notice = FailureNotice {
            id: self.next_id(),
            channel: upload.channel,
            after,
            time: time::clock(&chrono::Local::now()),
        };
        self.notices.update(|list| list.push(notice));
        let restored: Vec<PendingAttachment> = upload
            .files
            .into_iter()
            .map(|mut file| {
                if let Some((_, name)) = uploaded.iter().find(|(id, _)| *id == file.id) {
                    file.uploaded_filename = Some(name.clone());
                }
                file
            })
            .collect();
        self.pending.update(|all| {
            let list = all.entry(upload.channel).or_default();
            let mut merged = restored;
            merged.append(list);
            merged.truncate(MAX_ATTACHMENTS_PER_MESSAGE);
            *list = merged;
        });
        Some((upload.channel, upload.content))
    }

    pub fn dismiss_notice(&self, id: u64) {
        self.notices.update(|list| list.retain(|n| n.id != id));
    }
}

use requiem_ui::components::{
    ButtonProps, CheckboxProps, ConfirmModalProps, FormItemProps, ModalProps, TextInputProps,
    TextProps, TooltipPosition, TooltipProps, div, icon, tooltip,
};
use requiem_ui::node::Node;
use requiem_ui::widget::primitives::{HScrollWidget, ImageWidget};
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::ui::{TEXT_DEFAULT, TEXT_SECONDARY};
use requiem_ui::icons::{EYE, EYE_SLASH, PENCIL, TRASH};

const MEDIA_WIDTH: f32 = 200.0;
const MEDIA_HEIGHT: f32 = 174.0;

pub fn thumbnail_size(image_width: f32, image_height: f32) -> (f32, f32) {
    fit_within(image_width, image_height, MEDIA_WIDTH, MEDIA_HEIGHT)
}

fn fit_within(image_width: f32, image_height: f32, max_width: f32, max_height: f32) -> (f32, f32) {
    if image_width <= 0.0 || image_height <= 0.0 {
        return (max_width, max_height);
    }
    let scale = (max_width / image_width)
        .min(max_height / image_height)
        .min(1.0);
    (
        (image_width * scale).round().max(1.0),
        (image_height * scale).round().max(1.0),
    )
}

#[derive(Clone)]
pub struct TrayViewer {
    pub viewer: media_viewer::MediaViewerState,
    pub images: images::ImageCache,
}

impl TrayViewer {
    fn open(&self, file: &PendingAttachment) {
        let bytes = match fs::read(&file.path) {
            Ok(bytes) => bytes,
            Err(err) => {
                log::warn!("can't open {} in the viewer: {err}", file.path.display());
                return;
            }
        };
        let url = format!(
            "file:///{}",
            file.path.display().to_string().replace('\\', "/")
        );
        self.images.insert_local(&url, Arc::from(bytes));
        let item = media_viewer::MediaItem {
            url,
            video_url: None,
            width: file.thumbnail.as_ref().map(|i| i.width() as u32),
            height: file.thumbnail.as_ref().map(|i| i.height() as u32),
            placeholder: None,
            filename: Some(file.filename.clone()),
            size: Some(file.size),
            alt: Some(file.description.clone()).filter(|d| !d.trim().is_empty()),
            author: None,
            is_attachment: false,
            zoom_only: true,
        };
        self.viewer.open(vec![item], 0);
    }
}

pub fn build_tray(
    state: &AttachmentsState,
    channel: Id<ChannelMarker>,
    viewer: Option<TrayViewer>,
) -> Node {
    let state = state.clone();
    Node::reactive(move || {
        let files = state
            .pending
            .with(|all| all.get(&channel).cloned().unwrap_or_default());
        if files.is_empty() {
            return div();
        }
        let offset = state
            .tray_scroll
            .borrow_mut()
            .entry(channel)
            .or_default()
            .clone();
        Node::new("div", HScrollWidget::new().with_shared_offset(offset))
            .class(attachments::ATTACHMENT_TRAY)
            .children(
                files
                    .iter()
                    .map(|file| build_card(&state, channel, file, viewer.as_ref()))
                    .collect::<Vec<_>>(),
            )
    })
}

pub fn paste_from_clipboard(state: &AttachmentsState, channel: Id<ChannelMarker>) -> bool {
    let files = clipboard::get_files();
    if !files.is_empty() {
        state.add_paths(channel, files);
        return true;
    }
    if clipboard::get_text().is_some_and(|t| !t.is_empty()) {
        return false;
    }
    let Some((width, height, rgba)) = clipboard::get_image() else {
        return false;
    };
    let Some(path) = write_pasted_png(width, height, &rgba) else {
        return false;
    };
    state.add_paths(channel, vec![path]).added > 0
}

fn write_pasted_png(width: usize, height: usize, rgba: &[u8]) -> Option<PathBuf> {
    let info = skia_safe::ImageInfo::new(
        (width as i32, height as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Unpremul,
        None,
    );
    let image =
        skia_safe_images::raster_from_data(&info, skia_safe::Data::new_copy(rgba), width * 4)?;
    let png = image.encode(None, skia_safe::EncodedImageFormat::PNG, 100)?;
    let nanos = SystemTime::now()
        .duration_since(std_time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let dir = env::temp_dir().join(format!("discord-paste-{}-{nanos}", process::id()));
    fs::create_dir_all(&dir).ok()?;
    let path = dir.join("image.png");
    fs::write(&path, png.as_bytes()).ok()?;
    Some(path)
}

pub fn build_drop_overlay(state: &AttachmentsState, channel_name: Option<String>) -> Node {
    let state = state.clone();
    let idle = state.drag_hover.clone();
    Node::reactive(move || {
        if !state.drag_hover.get() {
            return div();
        }

        let title = match (&channel_name, state.shift_held.get()) {
            (_, true) => requiem_i18n::t!("attachments-insta-upload-mode").to_string(),
            (Some(name), false) => requiem_i18n::t!("attachments-upload-to-channel", name = name),
            (None, false) => requiem_i18n::t!("attachments-upload-to").to_string(),
        };
        let white = Color::WHITE;
        let heading = TextProps::new(title, 22.0_f32)
            .line_height(28.0_f32)
            .weight(Weight::BOLD)
            .color(white)
            .center(true)
            .build()
            .class(css::DROP_TITLE);
        let instructions = TextProps::styled(
            requiem_i18n::t!("attachments-you-can-add-comments-before-uploading"),
            TextVariant::TEXT_XS_NORMAL,
        )
        .color(white)
        .center(true)
        .build();
        let icon_node = |asset: &'static str, class: &'static str| {
            div().class(css::DROP_ICON).class(class).child(Node::new(
                "div",
                ImageWidget::new(ui::asset_svg(asset, ICON_PX.0, ICON_PX.1))
                    .with_intrinsic_size(72.0, 96.0),
            ))
        };

        let icons = div()
            .class(css::DROP_ICONS)
            .child(icon_node("file_icon/document", css::DROP_ICON_ONE))
            .child(icon_node("file_icon/drop_image", css::DROP_ICON_TWO))
            .child(icon_node("file_icon/code", css::DROP_ICON_THREE));
        let modal = div()
            .class(css::DROP_MODAL)
            .child(div().class(css::DROP_MODAL_BG))
            .child(
                div()
                    .class(css::DROP_MODAL_INNER)
                    .child(icons)
                    .child(heading)
                    .child(div().class(css::DROP_INSTRUCTIONS).child(instructions)),
            );
        div().class(css::DROP_OVERLAY).child(modal)
    })
    .class(css::DROP_OVERLAY_HOST)
    .bind_class(idle, "", css::DROP_OVERLAY_HOST_IDLE)
}

pub fn spoiler_pill() -> Node {
    let label = TextProps::new(requiem_i18n::t!("common-spoiler"), 15.0_f32)
        .line_height(15.0_f32)
        .weight(Weight::SEMI_BOLD)
        .color(Color::from_rgb(0xe4, 0xe4, 0xe6))
        .build();
    div()
        .class(css::SPOILER_PILL_HOLDER)
        .child(div().class(css::SPOILER_PILL).child(label))
}

fn action_button(
    label: &str,
    path: impl IconPath,
    color: Color,
    danger: bool,
    on_click: impl FnMut() + 'static,
) -> Node {
    let mut classes = vec![css::ATTACHMENT_ACTION_BTN.to_string()];
    if danger {
        classes.push(css::ATTACHMENT_ACTION_BTN_DANGER.to_string());
    }
    let button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(classes)
        .on_click(on_click)
        .build()
        .child(icon(path, color).class(css::ATTACHMENT_ACTION_ICON));
    tooltip(
        button,
        TooltipProps::default()
            .text(label.to_string())
            .position(TooltipPosition::Top),
    )
}

fn build_card(
    state: &AttachmentsState,
    channel: Id<ChannelMarker>,
    file: &PendingAttachment,
    viewer: Option<&TrayViewer>,
) -> Node {
    let media = match &file.thumbnail {
        Some(image) => {
            let (w, h) = thumbnail_size(image.width() as f32, image.height() as f32);
            let mut thumb = Node::new(
                "div",
                ImageWidget::new(Some(image.clone())).with_intrinsic_size(w, h),
            )
            .class(css::ATTACHMENT_THUMBNAIL);
            let mut holder = div().class(css::ATTACHMENT_SPOILER_BOX);
            if file.spoiler {
                thumb = thumb.class(css::ATTACHMENT_THUMBNAIL_SPOILER);
                holder = holder
                    .class(css::ATTACHMENT_SPOILER_BOX_HIDDEN)
                    .child(thumb)
                    .child(spoiler_pill());
            } else {
                holder = holder.child(thumb);
            }

            // Videos only borrow a frame for the card; the image viewer can't open the file.
            if let Some(viewer) = viewer.filter(|_| file.kind == FileKind::Image) {
                let (viewer, file) = (viewer.clone(), file.clone());
                holder = Node::new(
                    "div",
                    ClickAreaWidget::new()
                        .with_on_click(move || viewer.open(&file)),
                )
                .child(holder);
            }
            div().class(css::ATTACHMENT_MEDIA).child(holder)
        }
        None => {
            let mut icon_box = div().class(css::ATTACHMENT_ICON_BOX).child(Node::new(
                "div",
                ImageWidget::new(card_icon(file.kind)).with_intrinsic_size(72.0, 96.0),
            ));
            if file.spoiler {
                let tag = TextProps::styled(
                    requiem_i18n::t!("common-spoiler"),
                    TextVariant::TEXT_XXS_SEMIBOLD,
                )
                .line_height(10.0_f32)
                .color(Color::BLACK)
                .build();
                icon_box = icon_box.child(
                    div()
                        .class(css::ATTACHMENT_TAGS)
                        .child(div().class(css::ATTACHMENT_TAG).child(tag)),
                );
            }

            icon_box
        }
    };
    let filename = TextProps::styled(file.filename.clone(), TextVariant::TEXT_SM_NORMAL)
        .color(TEXT_DEFAULT)
        .build()
        .class(css::ATTACHMENT_FILENAME);

    let id = file.id;
    let spoiler = {
        let state = state.clone();
        action_button(
            &requiem_i18n::t!("attachments-spoiler-attachment"),
            if file.spoiler { EYE_SLASH } else { EYE },
            TEXT_SECONDARY,
            false,
            move || state.toggle_spoiler(channel, id),
        )
    };
    let modify = {
        let state = state.clone();
        action_button(
            &requiem_i18n::t!("attachments-modify-attachment"),
            PENCIL,
            TEXT_SECONDARY,
            false,
            move || state.open_modify(channel, id),
        )
    };
    let remove = {
        let state = state.clone();
        action_button(
            &requiem_i18n::t!("attachments-remove-attachment"),
            TRASH,
            TEXT_CRITICAL,
            true,
            move || state.remove(channel, id),
        )
    };

    let actions = div().class(css::ATTACHMENT_ACTIONS).child(
        div()
            .class(css::ATTACHMENT_ACTIONS_WRAPPER)
            .child(spoiler)
            .child(modify)
            .child(remove),
    );

    let inner = div()
        .class(css::ATTACHMENT_INNER)
        .child(media)
        .child(div().class(css::ATTACHMENT_FILENAME_BOX).child(filename))
        .child(actions);
    div()
        .class(css::ATTACHMENT_CARD)
        .dom_id(format!("attachment-card-{}", file.id))
        .child(inner)
}

pub fn build_modify_modal(state: &AttachmentsState, open: ToggleState) -> Option<Node> {
    let (thumbnail, kind) = match (state.modify.sent.get(), state.modify.target.peek()) {
        (Some(name), _) => (None, file_kind(&name)),
        (None, Some((channel, id))) => {
            let file = state.pending.with_untracked(|all| {
                all.get(&channel)
                    .and_then(|list| list.iter().find(|a| a.id == id).cloned())
            })?;
            (file.thumbnail.clone(), file.kind)
        }
        (None, None) => return None,
    };

    let preview = match &thumbnail {
        Some(image) => {
            let (w, h) = fit_within(
                image.width() as f32,
                image.height() as f32,
                430.0 - 16.0,
                150.0,
            );
            Node::new(
                "div",
                ImageWidget::new(Some(image.clone())).with_intrinsic_size(w, h),
            )
            .class(css::MODIFY_PREVIEW)
        }
        None => Node::new(
            "div",
            ImageWidget::new(card_icon(kind)).with_intrinsic_size(72.0, 96.0),
        )
        .class(css::MODIFY_PREVIEW),
    };
    let input = |text: String, placeholder: &str, multiline: bool, sink: Rc<RefCell<String>>| {
        let props = TextInputProps::new()
            .text(text)
            .placeholder(placeholder.to_string())
            .on_change(move |text: &str| *sink.borrow_mut() = text.to_string());
        if multiline {
            props.multiline(true).rows(3_u32).build_area()
        } else {
            let save = state.clone();

            let just_opened =
                open.on && open.changed_at.elapsed() < Duration::from_millis(500);
            props
                .autofocus(just_opened)
                .on_submit(move |_text: String| save.save_modify())
                .build()
        }
    };
    let filename = FormItemProps::default()
        .title(requiem_i18n::t!("attachments-filename").to_string())
        .children(input(
            state.modify.filename.borrow().clone(),
            "",
            false,
            state.modify.filename.clone(),
        ))
        .build();
    let mut body = StackProps::vertical()
        .gap(16)
        .full_width(false)
        .build()
        .child(preview)
        .child(filename);
    if kind == FileKind::Image {
        body = body.child(
            FormItemProps::default()
                .title(requiem_i18n::t!("attachments-description-alt-text").to_string())
                .children(input(
                    state.modify.description.borrow().clone(),
                    &requiem_i18n::t!("attachments-add-a-description"),
                    true,
                    state.modify.description.clone(),
                ))
                .build(),
        );
    }
    let spoiler = {
        let draft = state.modify.spoiler.clone();
        CheckboxProps::new(
            state
                .modify
                .spoiler_toggle
                .track(state.modify.spoiler.get()),
            move |on: bool| draft.set(on),
        )
        .label(requiem_i18n::t!("attachments-mark-as-spoiler").to_string())
        .build()
    };
    let body = StackProps::vertical()
        .gap(24)
        .full_width(false)
        .build()
        .child(body)
        .child(spoiler);

    let close = {
        let state = state.clone();
        move || state.close_modify()
    };
    let cancel = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .label(requiem_i18n::t!("attachments-cancel").to_string())
        .on_click(close.clone())
        .build();
    let save = {
        let state = state.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .label(requiem_i18n::t!("attachments-save").to_string())
            .on_click(move || state.save_modify())
            .build()
    };
    let node = ModalProps::default()
        .size(ModalSize::Medium)
        .heading(requiem_i18n::t!("attachments-modify-attachment"))
        .dismissable(true)
        .on_dismiss(close)
        .open(open)
        .body(body)
        .footer(vec![cancel, save])
        .build();
    Some(node)
}

pub fn build_too_many_modal(state: &AttachmentsState, open: ToggleState) -> Node {
    let state = state.clone();
    ConfirmModalProps::new(
        requiem_i18n::t!("attachments-too-many-uploads"),
        requiem_i18n::t!("attachments-got-it"),
        move || state.close_too_many(),
        open,
    )
    .subtitle(requiem_i18n::t!(
        "attachments-too-many-uploads-body",
        max = MAX_ATTACHMENTS_PER_MESSAGE
    ))
    .notice()
    .build()
}

#[cfg(test)]
mod tests {
    use crate::test_assets;
    use requiem_ui::font_system::TextFonts;
    use requiem_ui::layout;
    use requiem_ui::paint;
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::style::cascade;
    use requiem_ui::style::parser;
    use requiem_ui::widget::WidgetTree;
    use requiem_ui::widget::primitives::BoxWidget;
    use std::cell::RefCell;
    use taffy::AvailableSpace;
    use taffy::Size;

    use super::*;

    fn temp_files(tag: &str, names: &[&str]) -> (PathBuf, Vec<PathBuf>) {
        let dir =
            env::temp_dir().join(format!("attachments-test-{}-{tag}", process::id()));
        fs::create_dir_all(&dir).unwrap();
        let paths = names
            .iter()
            .map(|n| {
                let p = dir.join(n);
                fs::write(&p, b"data").unwrap();
                p
            })
            .collect();
        (dir, paths)
    }

    fn recording() -> (AttachmentsState, Rc<RefCell<Vec<String>>>) {
        let log: Rc<RefCell<Vec<String>>> = Rc::default();
        let (sends, cancels, deletes) = (log.clone(), log.clone(), log.clone());
        let backend = AttachmentsBackend {
            pick_files: Rc::new(|_| {}),
            send: Rc::new(move |_, nonce, content, files| {
                sends
                    .borrow_mut()
                    .push(format!("send {nonce} {content:?} {}", files.len()))
            }),
            cancel: Rc::new(move |nonce| cancels.borrow_mut().push(format!("cancel {nonce}"))),
            delete_upload: Rc::new(move |name| deletes.borrow_mut().push(format!("delete {name}"))),
        };
        (AttachmentsState::new(backend), log)
    }

    #[test]
    fn classifies_files_like_the_real_client() {
        assert_eq!(file_kind("cat.PNG"), FileKind::Image);
        assert_eq!(file_kind("clip.mp4"), FileKind::Video);
        assert_eq!(file_kind("report.pdf"), FileKind::Acrobat);
        assert_eq!(file_kind("backup.tar.gz"), FileKind::Archive);
        assert_eq!(
            file_kind("main.rs"),
            FileKind::Unknown,
            "rs isn't in the client's code list"
        );
        assert_eq!(file_kind("script.py"), FileKind::Code);
        assert_eq!(file_kind("notes.md"), FileKind::Document);
        assert_eq!(file_kind("budget.csv"), FileKind::Spreadsheet);
        assert_eq!(
            file_kind("logo.svg"),
            FileKind::Webcode,
            "svg is web code, not an image thumbnail"
        );
        assert_eq!(file_kind("song.flac"), FileKind::Audio);
        assert_eq!(file_kind("mystery"), FileKind::Unknown);
    }

    #[test]
    fn every_non_image_kind_has_an_icon_that_decodes() {
        if !test_assets::install() {
            return;
        }
        for kind in [
            FileKind::Video,
            FileKind::Acrobat,
            FileKind::Ae,
            FileKind::Sketch,
            FileKind::Ai,
            FileKind::Archive,
            FileKind::Code,
            FileKind::Document,
            FileKind::Photoshop,
            FileKind::Spreadsheet,
            FileKind::Webcode,
            FileKind::Audio,
            FileKind::Unknown,
        ] {
            let image = icon_image(kind).unwrap_or_else(|| panic!("{kind:?} icon should decode"));
            assert_eq!((image.width(), image.height()), (144, 192), "{kind:?}");
        }
        assert!(icon_image(FileKind::Image).is_none());
        assert!(
            card_icon(FileKind::Image).is_some(),
            "file cards fall back to the generic icon"
        );
    }

    #[test]
    fn a_batch_past_ten_files_is_refused_whole() {
        let (s, _) = recording();
        let channel = Id::new(1);
        let names: Vec<String> = (0..11).map(|i| format!("f{i}.txt")).collect();
        let (dir, paths) = temp_files("cap", &names.iter().map(String::as_str).collect::<Vec<_>>());
        assert_eq!(s.add_paths(channel, paths[..3].to_vec()).added, 3);
        let result = s.add_paths(channel, paths[3..].to_vec());
        assert_eq!(
            (result.added, result.over_limit),
            (0, 8),
            "3 + 8 > 10: none of the 8 go in"
        );
        assert_eq!(s.count(channel), 3);
        assert!(s.too_many_open.peek().on, "the dialog opens");
        assert_eq!(
            s.add_paths(channel, paths[3..10].to_vec()).added,
            7,
            "exactly 10 is fine"
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn folders_missing_and_empty_files_are_reported_not_added() {
        let (s, _) = recording();
        let (dir, _) = temp_files("bad", &["a.txt"]);
        fs::write(dir.join("empty.txt"), b"").unwrap();
        let result = s.add_paths(
            Id::new(2),
            vec![dir.clone(), dir.join("nope.bin"), dir.join("empty.txt")],
        );
        assert_eq!(result.added, 0);
        assert_eq!(result.unreadable.len(), 2);
        assert_eq!(result.empty, vec!["empty.txt".to_string()]);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn spoiler_and_modify_edit_the_tray_file() {
        let (s, _) = recording();
        let channel = Id::new(3);
        let (dir, paths) = temp_files("modify", &["cat.png"]);
        s.add_paths(channel, paths);
        let id = s.pending.with_untracked(|all| all[&channel][0].id);
        s.toggle_spoiler(channel, id);
        assert!(s.pending.with_untracked(|all| all[&channel][0].spoiler));

        s.open_modify(channel, id);
        assert!(s.modify.open.peek().on);
        assert!(s.modify.spoiler.peek(), "the draft starts from the file");
        *s.modify.filename.borrow_mut() = "  dog.png ".into();
        *s.modify.description.borrow_mut() = "a dog".into();
        s.modify.spoiler.set(false);
        s.save_modify();
        let file = s.pending.with_untracked(|all| all[&channel][0].clone());
        assert_eq!(
            (
                file.filename.as_str(),
                file.description.as_str(),
                file.spoiler
            ),
            ("dog.png", "a dog", false)
        );
        assert!(!s.modify.open.peek().on);

        s.open_modify(channel, id);
        *s.modify.filename.borrow_mut() = "   ".into();
        s.save_modify();
        assert_eq!(
            s.pending
                .with_untracked(|all| all[&channel][0].filename.clone()),
            "dog.png"
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn sending_empties_the_tray_into_a_pending_message_until_it_arrives() {
        let (s, _) = recording();
        let channel = Id::new(4);
        let (dir, paths) = temp_files("send", &["a.txt", "b.txt"]);
        s.add_paths(channel, paths);
        s.toggle_spoiler(channel, s.pending.with_untracked(|all| all[&channel][0].id));
        let (nonce, files) = s.begin_send(channel, "hi".into()).expect("files queued");
        assert_eq!(s.count(channel), 0, "the tray empties straight away");
        assert_eq!(files.len(), 2);
        assert!(files[0].spoiler);
        assert_eq!(
            s.uploads
                .with_untracked(|u| (u.len(), u[0].content.clone(), u[0].total)),
            (1, "hi".to_string(), 8)
        );
        assert!(nonce > 1 << 22, "snowflake-shaped");

        s.progress(nonce, UploadProgress { sent: 4, total: 8 });
        assert_eq!(s.uploads.with_untracked(|u| u[0].sent), 4);
        s.message_created(&nonce.to_string());
        assert!(
            s.uploads.with_untracked(Vec::is_empty),
            "the real message replaces the pending row"
        );
        assert!(
            s.begin_send(channel, "x".into()).is_none(),
            "nothing left to send"
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn cancel_gives_back_the_text_but_not_the_files() {
        let (s, _) = recording();
        let channel = Id::new(5);
        let (dir, paths) = temp_files("cancel", &["a.txt"]);
        s.add_paths(channel, paths);
        let (nonce, _) = s.begin_send(channel, "draft".into()).unwrap();
        assert_eq!(s.cancelled(nonce), Some((channel, "draft".to_string())));
        assert_eq!(s.count(channel), 0);
        assert!(s.uploads.with_untracked(Vec::is_empty));
        assert_eq!(s.cancelled(nonce), None, "only once");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_failed_send_restores_text_and_files_keeping_finished_uploads() {
        let (s, log) = recording();
        let channel = Id::new(6);
        let (dir, paths) = temp_files("fail", &["a.txt", "b.txt"]);
        s.add_paths(channel, paths);
        let (nonce, files) = s.begin_send(channel, "retry me".into()).unwrap();
        let after = Some(Id::new(99));
        let restored = s.failed(
            nonce,
            &[(files[0].local_id, "uploads/1/a.txt".into())],
            after,
        );
        assert_eq!(restored, Some((channel, "retry me".to_string())));
        let tray = s.pending.with_untracked(|all| all[&channel].clone());
        assert_eq!(tray.len(), 2);
        assert_eq!(
            tray[0].uploaded_filename.as_deref(),
            Some("uploads/1/a.txt")
        );
        assert_eq!(tray[1].uploaded_filename, None);
        let notices = s.notices.peek();
        assert_eq!(
            (notices.len(), notices[0].channel, notices[0].after),
            (1, channel, after)
        );

        let (_, again) = s.begin_send(channel, String::new()).unwrap();
        assert_eq!(
            again[0].uploaded_filename.as_deref(),
            Some("uploads/1/a.txt")
        );
        let nonce = s.uploads.peek()[0].nonce;
        s.failed(nonce, &[(tray[0].id, "uploads/1/a.txt".into())], None);
        s.remove(channel, tray[0].id);
        assert_eq!(
            log.borrow().last().map(String::as_str),
            Some("delete uploads/1/a.txt")
        );

        s.dismiss_notice(notices[0].id);
        assert_eq!(s.notices.peek().len(), 1);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn thumbnails_fit_the_media_area_without_upscaling() {
        assert_eq!(thumbnail_size(640.0, 360.0), (200.0, 113.0));
        assert_eq!(thumbnail_size(500.0, 1000.0), (87.0, 174.0));
        assert_eq!(
            thumbnail_size(64.0, 48.0),
            (64.0, 48.0),
            "small images keep their size"
        );
    }

    #[test]
    fn tray_cards_and_action_bar_match_the_client() {
        TextFonts::install_new(16 * 1024 * 1024);
        let (s, _) = recording();
        let channel = Id::new(7);
        let (dir, paths) = temp_files("layout", &["a.txt", "b.zip"]);
        s.add_paths(channel, paths);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        tree.mount(root, build_tray(&s, channel, None));
        let theme = parser::parse_stylesheet(&format!(
            "{}\n{}",
            requiem_ui::BUNDLED_CSS,
            ui::THEME_CSS
        ));
        cascade::apply_stylesheet(&mut tree, &theme);
        layout::compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(1100.0),
                height: AvailableSpace::Definite(400.0),
            },
        );
        let rects = paint::all_absolute_rects(&tree);
        let ids: Vec<u64> = s
            .pending
            .with_untracked(|all| all[&channel].iter().map(|a| a.id).collect());
        for id in ids {
            let card_id =
                paint::find_by_dom_id(&tree, &format!("attachment-card-{id}")).unwrap();
            let card = rects[&card_id];
            assert_eq!((card.width, card.height), (218.0, 218.0));
            let inner_id = tree.nodes[card_id].children[0];
            let inner = rects[&inner_id];
            assert_eq!((inner.x - card.x, inner.y - card.y), (9.0, 9.0));
            let bar_id = *tree.nodes[inner_id].children.last().unwrap();
            let bar = rects[&bar_id];
            assert_eq!((bar.width, bar.height), (96.0, 32.0));
            assert_eq!(
                (bar.x + bar.width - (card.x + card.width), bar.y - card.y),
                (15.0, 1.0)
            );
        }
        fs::remove_dir_all(dir).ok();
    }
}
