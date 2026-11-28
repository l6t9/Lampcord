use crate::config;
#[cfg(not(feature = "video"))]
use std::iter;
#[cfg(not(feature = "video"))]
use std::marker::PhantomData;
use crate::models;
use crate::ui::attachments;
use crate::ui::classes::message_embed;
use crate::ui::classes::message_embed as cls;
use crate::ui::classes::message_embed as css;
use crate::ui::context_menu;
use crate::ui::emoji_text;
use crate::ui::twemoji;
#[cfg(feature = "video")]
use crate::ui::video_player::Poster;
#[cfg(feature = "video")]
use crate::ui::video_player;
use crate::ui;
use crate::utils::format;
use crate::utils::link;
#[cfg(feature = "video")]
use crate::video;
use requiem_ui::colors::{SURFACE_HIGH, TEXT_ANCHOR, TEXT_DEFAULT, TEXT_MUTED, TEXT_STRONG};
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::StackProps;
use requiem_ui::components::TextVariant;
use requiem_ui::components::TooltipPosition;
use requiem_ui::components::TooltipProps;
use requiem_ui::components::icon;
use requiem_ui::components::tooltip;
use requiem_ui::components::{ButtonProps, TextProps, div, image};
use requiem_ui::components;
use requiem_ui::icons::Icon;
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::signal;
use requiem_ui::style::ComputedStyle;
use requiem_ui::timer;
use requiem_ui::widget::primitives::ClickAreaWidget;
use requiem_ui::widget::primitives::ImageWidget;
use requiem_ui::widget::primitives::MarkdownFeatures;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::font_style::Weight;
use skia_safe::{Canvas, Color, Paint, RRect, Rect};
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashSet;
use std::mem;
use std::time::Duration;

use std::rc::Rc;

use crate::images::ImageCache;
use crate::models::{Attachment, Embed, Message};
use crate::ui::media_viewer::{MediaItem, MediaViewerState, ViewerAuthor};
use requiem_ui::icons::{DOWNLOAD, PENCIL, TRASH};
use requiem_ui::text_cache::MeasureCache;

fn cached_text(
    text: impl Into<String>,
    variant: TextVariant,
    measure_cache: Option<&Rc<MeasureCache>>,
    cache_key: Option<u64>,
) -> TextProps {
    let mut props = TextProps::styled(text, variant);
    if let Some(mc) = measure_cache {
        props = props.measure_cache(mc.clone());
    }
    if let Some(key) = cache_key {
        props = props.cache_key(key);
    }
    props
}

/// Makes `node` open `url` when clicked. Without a url the node is returned as is.
fn linked(node: Node, url: Option<&str>) -> Node {
    let Some(url) = url.filter(|url| url.starts_with("http")).map(str::to_string) else {
        return node;
    };
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .on_click(move || link::open_url(&url))
        .build()
        .child(node.class(message_embed::EMBED_LINK_TEXT))
}

pub(super) fn embed_markdown(
    text: &str,
    line_height: f32,
    images: &ImageCache,
    backend: &config::BackendConfig,
    tick: &Signal<u64>,
    measure_cache: Option<&Rc<MeasureCache>>,
    cache_key: u64,
) -> Node {
    components::message_text(components::MessageTextProps {
        content: text.to_string(),
        size: 14.0,
        color: TEXT_DEFAULT,
        line_height: Some(line_height),
        measure_cache: measure_cache.cloned(),
        cache_key: Some(cache_key),
        emoji_provider: Some(twemoji::tile),
        custom_emoji: Some(emoji_text::resolver(images, backend)),
        markdown: Some(MarkdownFeatures::EMBED),
        image_tick: (text.contains("<:") || text.contains("<a:")).then(|| tick.clone()),
        ..Default::default()
    })
}

#[derive(Clone)]
pub struct ViewerLink {
    state: MediaViewerState,
    items: Rc<Vec<MediaItem>>,
    menu: Option<AttachmentMenu>,
    file_actions: Option<FileActions>,
}

#[derive(Clone)]
pub struct FileActions {
    pub download: Rc<dyn Fn(&Attachment)>,
    pub edit: Option<Rc<dyn Fn(&Attachment)>>,
    pub delete: Option<Rc<dyn Fn(&Attachment)>>,
}

pub type AttachmentMenu = Rc<dyn Fn(&Attachment, f32, f32)>;

impl ViewerLink {
    pub fn with_menu(mut self, menu: AttachmentMenu) -> Self {
        self.menu = Some(menu);
        self
    }

    pub fn with_file_actions(mut self, actions: FileActions) -> Self {
        self.file_actions = Some(actions);
        self
    }

    fn menu_on(viewer: Option<&ViewerLink>, node: Node, attachment: &Attachment) -> Node {
        let Some(menu) = viewer.and_then(|v| v.menu.clone()) else {
            return node;
        };
        let attachment = attachment.clone();
        node.on_context_menu(move |x, y| menu(&attachment, x, y))
    }

    pub fn for_message(
        state: &MediaViewerState,
        message: &Message,
        backend: &config::BackendConfig,
        timestamp: Option<String>,
    ) -> Self {
        let author = ViewerAuthor {
            id: message.author.id,
            discriminator: message.author.discriminator.clone(),
            username: message.author.display_name(None),
            avatar_url: message
                .author
                .avatar
                .as_deref()
                .map(|hash| backend.avatar_url(message.author.id, hash)),
            timestamp,
        };
        let mut items = Vec::new();
        for a in message.attachments.iter().filter(|a| a.is_image()) {
            items.push(MediaItem {
                url: a.proxy_url.clone().unwrap_or_else(|| a.url.clone()),
                video_url: None,
                width: a.width,
                height: a.height,
                placeholder: a.placeholder.clone(),
                filename: Some(a.filename.clone()),
                size: Some(a.size),
                alt: None,
                author: Some(author.clone()),
                is_attachment: true,
                zoom_only: false,
            });
        }
        for embed in &message.embeds {
            let gifv = embed.kind.as_deref() == Some("gifv");
            if embed.video.is_some() && !gifv {
                continue;
            }
            let media = if gifv {
                embed.thumbnail.as_ref().or(embed.image.as_ref())
            } else {
                embed.image.as_ref().or(embed.thumbnail.as_ref())
            };
            if let Some(media) = media {
                let video_url = embed
                    .video
                    .as_ref()
                    .filter(|_| gifv)
                    .map(|v| v.proxy_url.clone().unwrap_or_else(|| v.url.clone()));
                items.push(MediaItem {
                    url: media.proxy_url.clone().unwrap_or_else(|| media.url.clone()),
                    video_url,
                    width: media.width,
                    height: media.height,
                    placeholder: media.placeholder.clone(),
                    filename: None,
                    size: None,
                    alt: None,
                    author: Some(author.clone()),
                    is_attachment: false,
                    zoom_only: false,
                });
            }
        }
        Self {
            state: state.clone(),
            items: Rc::new(items),
            menu: None,
            file_actions: None,
        }
    }
}

#[derive(Clone, Copy)]
#[cfg(feature = "video")]
pub struct VideoContext<'a> {
    pub videos: &'a video::VideoCache,
    pub fullscreen: &'a Signal<Option<video_player::FullscreenVideoState>>,
}

#[cfg(not(feature = "video"))]
pub struct VideoContext<'a>(PhantomData<&'a ()>);

const MAX_MEDIA_WIDTH: f32 = 400.0;
const MAX_MEDIA_HEIGHT: f32 = 300.0;

const MAX_ATTACHMENT_WIDTH: f32 = 550.0;

thread_local! {

    static WIDTH_CAP: Cell<Option<f32>> = const { Cell::new(None) };
}

pub fn with_width_cap<R>(cap: f32, f: impl FnOnce() -> R) -> R {
    let previous = WIDTH_CAP.with(|c| c.replace(Some(cap)));
    let result = f();
    WIDTH_CAP.with(|c| c.set(previous));
    result
}

pub(super) fn capped_width(node: Node, exact: bool) -> Node {
    match WIDTH_CAP.with(|c| c.get()) {
        Some(cap) if exact => node.style(format!("width: {cap}px")),
        Some(cap) => node.style(format!("max-width: {cap}px")),
        None => node,
    }
}

pub(super) fn cap_width(width: f32) -> f32 {
    WIDTH_CAP
        .with(|c| c.get())
        .map_or(width, |cap| width.min(cap))
}
const MAX_ATTACHMENT_HEIGHT: f32 = 350.0;

const EMBED_CARD_RADIUS: f32 = 4.0;
const EMBED_ACCENT_WIDTH: f32 = 4.0;

const EMBED_DEFAULT_ACCENT: Color = Color::from_argb(51, 0x96, 0x97, 0x9e);

struct EmbedCardWidget {
    accent: Color,
}

impl Widget for EmbedCardWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let rrect = RRect::new_rect_xy(bounds, EMBED_CARD_RADIUS, EMBED_CARD_RADIUS);

        canvas.save();
        canvas.clip_rrect(rrect, None, true);

        let mut bg_paint = Paint::default();
        bg_paint.set_anti_alias(true);
        bg_paint.set_color(SURFACE_HIGH);
        canvas.draw_rect(bounds, &bg_paint);

        let mut accent_paint = Paint::default();
        accent_paint.set_anti_alias(true);
        accent_paint.set_color(self.accent);
        canvas.draw_rect(
            Rect::from_xywh(rect.x, rect.y, EMBED_ACCENT_WIDTH, rect.height),
            &accent_paint,
        );

        canvas.restore();
    }
}

pub(super) fn scaled_dims(
    width: Option<u32>,
    height: Option<u32>,
    decoded: Option<&skia_safe::Image>,
    max_w: f32,
    max_h: f32,
) -> Option<(f32, f32)> {
    let (w, h) = match (width, height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => (w as f32, h as f32),
        _ => {
            let img = decoded?;
            (img.width() as f32, img.height() as f32)
        }
    };
    let scale = (max_w / w).min(max_h / h).min(1.0);
    Some((w * scale, h * scale))
}

fn media_row(img: Node) -> Node {
    StackProps::horizontal()
        .gap(0)
        .full_width(false)
        .build()
        .child(img)
}

fn media_row_with_bar(img: Node, bar: Option<Node>) -> Node {
    match bar {
        Some(bar) => media_row(
            div()
                .class(message_embed::MEDIA_WRAP)
                .child(img)
                .child(bar),
        ),
        None => media_row(img),
    }
}

pub(super) fn viewable_row(img: Node, url: &str, viewer: Option<&ViewerLink>, bar: Option<Node>) -> Node {
    let Some(link) = viewer else {
        return media_row_with_bar(img, bar);
    };
    let Some(index) = link.items.iter().position(|item| item.url == url) else {
        return media_row_with_bar(img, bar);
    };
    let link = link.clone();

    media_row_with_bar(
        Node::new(
            "div",
            ClickAreaWidget::new()
                .with_on_click(move || link.state.open(link.items.to_vec(), index)),
        )
        .child(img),
        bar,
    )
}

thread_local! {

    static REVEALED_SPOILERS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

fn spoiler_cover(media: Node, id: &str) -> Node {
    let revealed = signal::signal(REVEALED_SPOILERS.with(|r| r.borrow().contains(id)));
    let (click_revealed, key) = (revealed.clone(), id.to_string());
    let pill = TextProps::new(requiem_i18n::t!("common-spoiler"), 15.0_f32)
        .line_height(20.0_f32)
        .weight(Weight::SEMI_BOLD)
        .color(Color::from_rgb(0xe4, 0xe4, 0xe6))
        .build();
    Node::new(
        "div",
        ClickAreaWidget::new().with_on_click(move || {
            if !click_revealed.peek() {
                REVEALED_SPOILERS.with(|r| r.borrow_mut().insert(key.clone()));
                click_revealed.set(true);
            }
        }),
    )
    .class(css::SPOILER_CONTAINER)
    .bind_class(revealed.clone(), "", css::SPOILER_HIDDEN)
    .child(media.bind_class(revealed.clone(), "", css::SPOILER_IMAGE_HIDDEN))
    .child(
        div()
            .class(css::SPOILER_PILL_HOLDER)
            .bind_class(revealed, css::SPOILER_PILL_GONE, "")
            .child(div().class(css::SPOILER_PILL).child(pill)),
    )
}

pub(super) fn media_node(
    url: &str,
    width: Option<u32>,
    height: Option<u32>,
    placeholder: Option<&str>,
    images: &ImageCache,
    max_w: f32,
    max_h: f32,
    class: &'static str,
    tick: &Signal<u64>,
) -> Option<Node> {
    let edge = (max_w.max(max_h) * 2.0).ceil() as i32;
    let decoded = images.get_or_decode_image_capped(url, edge);
    let placeholder_img = if decoded.is_none() {
        placeholder.and_then(|hash| images.get_or_decode_placeholder(hash))
    } else {
        None
    };
    let dims_source = decoded.as_ref().or(placeholder_img.as_ref());
    let (w, h) = scaled_dims(width, height, dims_source, max_w, max_h)?;
    let initial = decoded.or(placeholder_img);

    let bound_images = images.clone();
    let bound_url = url.to_string();
    Some(
        image(initial, w, h)
            .class(class)
            .bind_image(tick.clone(), move || {
                bound_images.get_or_decode_image_capped(&bound_url, edge)
            }),
    )
}

#[cfg(feature = "video")]
fn video_still_url(url: &str) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}format=webp")
}

#[cfg(feature = "video")]
fn embed_poster(
    media: Option<&models::EmbedMedia>,
    images: &ImageCache,
) -> Poster {
    let Some(media) = media else {
        return Poster::default();
    };
    let url = media.proxy_url.clone().unwrap_or_else(|| media.url.clone());
    let placeholder = media
        .placeholder
        .as_deref()
        .and_then(|hash| images.get_or_decode_placeholder(hash));
    Poster::new(images, Some(url), placeholder)
}

pub fn attachment_images(
    attachments: &[Attachment],
    images: &ImageCache,
    tick: &Signal<u64>,
    video_ctx: Option<VideoContext>,
    viewer: Option<&ViewerLink>,
) -> Vec<Node> {
    let in_mosaic: Vec<MosaicItem> = attachments
        .iter()
        .filter(|a| a.is_image() && !a.is_spoiler())
        .take(MOSAIC_MAX)
        .map(MosaicItem::of)
        .collect();
    let use_mosaic = in_mosaic.len() >= 2;
    let mosaic_node = use_mosaic.then(|| mosaic(&in_mosaic, cap_width(MOSAIC_WIDTH), images, tick, viewer));
    let mosaic_ids: Vec<_> = if use_mosaic {
        in_mosaic
            .iter()
            .filter_map(|item| item.attachment.map(|a| a.id))
            .collect()
    } else {
        Vec::new()
    };
    let images_nodes = attachments
        .iter()
        .filter(|a| a.is_image() && !mosaic_ids.contains(&a.id))
        .filter_map(|a| {
            let url = a.proxy_url.as_deref().unwrap_or(&a.url);
            let node = media_node(
                url,
                a.width,
                a.height,
                a.placeholder.as_deref(),
                images,
                cap_width(MAX_ATTACHMENT_WIDTH),
                MAX_ATTACHMENT_HEIGHT,
                message_embed::ATTACHMENT_IMAGE,
                tick,
            )?;
            let bar = media_actions(a, viewer, false);
            let row = viewable_row(ViewerLink::menu_on(viewer, node, a), url, viewer, bar);
            Some(if a.is_spoiler() {
                spoiler_cover(row, &a.id.to_string())
            } else {
                row
            })
        });

    #[cfg(feature = "video")]
    let viewer_for_video = viewer;
    #[cfg(feature = "video")]
    let video_nodes = attachments
        .iter()
        .filter(|a| a.is_video())
        .filter_map(move |a| {
            let bar = media_actions(a, viewer_for_video, true);
            let ctx = video_ctx.as_ref()?;
            let url = a.proxy_url.as_deref().unwrap_or(&a.url);
            let placeholder = a
                .placeholder
                .as_deref()
                .and_then(|hash| images.get_or_decode_placeholder(hash));
            let poster = Poster::new(
                images,
                Some(video_still_url(url)),
                placeholder,
            );
            let node = video_player::attachment_video_node(
                url,
                poster,
                a.width,
                a.height,
                ctx.videos,
                cap_width(MAX_ATTACHMENT_WIDTH),
                MAX_ATTACHMENT_HEIGHT,
                ctx.fullscreen,
                message_embed::ATTACHMENT_IMAGE,
            );
            Some(media_row_with_bar(
                ViewerLink::menu_on(viewer_for_video, node, a),
                bar,
            ))
        });
    #[cfg(not(feature = "video"))]
    let video_nodes = {
        let _ = video_ctx;
        iter::empty()
    };

    mosaic_node
        .into_iter()
        .chain(images_nodes)
        .chain(video_nodes)
        .collect()
}

const MOSAIC_MAX: usize = 10;

const MOSAIC_WIDTH: f32 = 550.0;
const MOSAIC_GAP: f32 = 4.0;

/// A picture cropped to fill `w` by `h`.
pub(super) fn cover_picture(
    url: &str,
    placeholder: Option<&str>,
    w: f32,
    h: f32,
    images: &ImageCache,
    tick: &Signal<u64>,
) -> Node {
    let edge = (w.max(h) * 2.0).ceil() as i32;
    let decoded = images.get_or_decode_image_capped(url, edge);
    let placeholder = if decoded.is_none() {
        placeholder.and_then(|hash| images.get_or_decode_placeholder(hash))
    } else {
        None
    };
    let (bound_images, bound_url) = (images.clone(), url.to_string());
    Node::new(
        "div",
        ImageWidget::new(decoded.or(placeholder))
            .with_cover()
            .with_cover_centered(),
    )
    .style(format!("width: {w}px; height: {h}px; flex-shrink: 0"))
    .bind_image(tick.clone(), move || {
        bound_images.get_or_decode_image_capped(&bound_url, edge)
    })
}

/// One picture in a mosaic. Attachments carry their menu, other media do not.
pub(super) struct MosaicItem<'a> {
    pub url: &'a str,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub placeholder: Option<&'a str>,
    pub attachment: Option<&'a Attachment>,
}

impl<'a> MosaicItem<'a> {
    fn of(attachment: &'a Attachment) -> Self {
        Self {
            url: attachment.proxy_url.as_deref().unwrap_or(&attachment.url),
            width: attachment.width,
            height: attachment.height,
            placeholder: attachment.placeholder.as_deref(),
            attachment: Some(attachment),
        }
    }
}

pub(super) fn mosaic(
    items: &[MosaicItem],
    width: f32,
    images: &ImageCache,
    tick: &Signal<u64>,
    viewer: Option<&ViewerLink>,
) -> Node {
    let aspect = |i: usize| match (items[i].width, items[i].height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => w as f32 / h as f32,
        _ => 1.0,
    };
    let cell = |i: usize, w: f32, h: f32| -> Node {
        let item = &items[i];
        let url = item.url.to_string();
        let picture = cover_picture(&url, item.placeholder, w, h, images, tick);
        let opened = match viewer {
            Some(link) => match link.items.iter().position(|item| item.url == url) {
                Some(index) => {
                    let link = link.clone();
                    Node::new(
                        "div",
                        ClickAreaWidget::new()
                            .with_on_click(move || link.state.open(link.items.to_vec(), index)),
                    )
                    .child(picture)
                }
                None => picture,
            },
            None => picture,
        };
        match item.attachment {
            Some(attachment) => ViewerLink::menu_on(viewer, opened, attachment),
            None => opened,
        }
    };
    let row = |cells: Vec<Node>| div().class(css::MOSAIC_ROW).children(cells);
    let half = (width - MOSAIC_GAP) / 2.0;
    let third = (width - MOSAIC_GAP * 2.0) / 3.0;
    let n = items.len();
    let mut rows: Vec<Node> = Vec::new();
    let mut next = 0;
    let pair = |rows: &mut Vec<Node>, next: &mut usize| {
        let (a, b) = (*next, *next + 1);
        let height = (half / ((aspect(a) + aspect(b)) / 2.0)).clamp(140.0, 280.0);
        rows.push(row(vec![cell(a, half, height), cell(b, half, height)]));
        *next += 2;
    };
    match n {
        2 => pair(&mut rows, &mut next),
        3 => {
            let big = (width - MOSAIC_GAP) * 2.0 / 3.0;
            let small = (width - MOSAIC_GAP) / 3.0;
            let height = (width * 350.0 / 600.0).ceil();
            let stacked = (height - MOSAIC_GAP) / 2.0;
            let column = div()
                .class(css::MOSAIC_COLUMN)
                .children([cell(1, small, stacked), cell(2, small, stacked)]);
            rows.push(row(vec![cell(0, big, height), column]));
        }
        4 => {
            let height = (350.0 - MOSAIC_GAP) / 2.0;
            rows.push(row(vec![cell(0, half, height), cell(1, half, height)]));
            rows.push(row(vec![cell(2, half, height), cell(3, half, height)]));
        }
        _ => {
            match n % 3 {
                1 => {
                    let height = (width / aspect(0)).clamp(120.0, 280.0);
                    rows.push(row(vec![cell(0, width, height)]));
                    next = 1;
                }
                2 => pair(&mut rows, &mut next),
                _ => {}
            }
            while next < n {
                rows.push(row((next..next + 3)
                    .map(|i| cell(i, third, third))
                    .collect()));
                next += 3;
            }
        }
    }
    div().class(css::MOSAIC).children(rows)
}

pub fn file_cards(
    attachments: &[Attachment],
    after_media: bool,
    viewer: Option<&ViewerLink>,
    measure_cache: Option<&Rc<MeasureCache>>,
) -> Vec<Node> {
    let playable_video = |a: &Attachment| cfg!(feature = "video") && a.is_video();
    attachments
        .iter()
        .filter(|a| !a.is_image() && !playable_video(a))
        .enumerate()
        .map(|(i, a)| {
            let url = a.url.clone();
            let name = cached_text(
                a.filename.clone(),
                TextVariant::TEXT_MD_NORMAL,
                measure_cache,
                None,
            )
            .color(TEXT_ANCHOR)
            .build()
            .class(css::FILE_CARD_NAME);
            let link = ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![css::FILE_CARD_LINK.to_string()])
                .on_click(move || link::open_url(&url))
                .build()
                .child(name);
            let size = cached_text(
                format::file_size(a.size),
                TextVariant::TEXT_XS_NORMAL,
                measure_cache,
                None,
            )
            .color(TEXT_MUTED)
            .build();
            let icon = image(
                attachments::card_icon(attachments::file_kind(&a.filename)),
                30.0,
                40.0,
            )
            .class(css::FILE_CARD_ICON);
            let mut card = capped_width(div().class(css::FILE_CARD), true)
                .child(icon)
                .child(
                    div()
                        .class(css::FILE_CARD_INNER)
                        .child(div().class(css::FILE_CARD_NAME_ROW).child(link))
                        .child(size),
                );
            if i == 0 && !after_media {
                card = card.class(css::FILE_CARD_FIRST);
            }
            if let Some(actions) = viewer.and_then(|v| v.file_actions.clone()) {
                card = card.child(action_bar(a, &actions, true, css::FILE_CARD_ACTIONS));
            }
            ViewerLink::menu_on(viewer, card, a)
        })
        .collect()
}

fn media_actions(
    attachment: &Attachment,
    viewer: Option<&ViewerLink>,
    downloadable: bool,
) -> Option<Node> {
    let actions = viewer?.file_actions.clone()?;
    if actions.edit.is_none() && actions.delete.is_none() && !downloadable {
        return None;
    }
    Some(action_bar(
        attachment,
        &actions,
        downloadable,
        message_embed::MEDIA_ACTIONS,
    ))
}

fn action_bar(
    attachment: &Attachment,
    actions: &FileActions,
    download: bool,
    bar_class: &'static str,
) -> Node {
    let button = |label: &str, path: Icon, color: Color, on_click: Rc<dyn Fn(&Attachment)>| {
        let attachment = attachment.clone();
        let node = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![css::FILE_CARD_ACTION.to_string()])
            .on_click(move || on_click(&attachment))
            .build()
            .child(icon(path, color).class(css::FILE_CARD_ACTION_ICON));
        tooltip(
            node,
            TooltipProps::default()
                .text(label.to_string())
                .position(TooltipPosition::Top),
        )
    };
    let mut bar = div().class(bar_class);
    if let Some(edit) = &actions.edit {
        bar = bar.child(button(
            &requiem_i18n::t!("message-embed-modify-attachment"),
            PENCIL,
            ui::TEXT_SECONDARY,
            edit.clone(),
        ));
    }
    if let Some(delete) = &actions.delete {
        bar = bar.child(button(
            &requiem_i18n::t!("message-embed-delete"),
            TRASH,
            ui::TEXT_SECONDARY,
            delete.clone(),
        ));
    }
    if download {
        bar = bar.child(button(
            &requiem_i18n::t!("message-embed-download"),
            DOWNLOAD,
            ui::TEXT_SECONDARY,
            actions.download.clone(),
        ));
    }
    bar
}

pub fn download_attachment(
    images: &ImageCache,
    on_save: Option<Rc<dyn Fn(Vec<u8>, String)>>,
    attachment: &Attachment,
) {
    fn poll(
        images: ImageCache,
        on_save: Rc<dyn Fn(Vec<u8>, String)>,
        url: String,
        filename: String,
        attempt: u32,
    ) {
        match images.get_or_request(&url) {
            Some(bytes) => on_save(bytes.to_vec(), filename),
            None if attempt < 600 => {
                timer::set_timeout(Duration::from_millis(100), move || {
                    poll(images, on_save, url, filename, attempt + 1)
                });
            }
            None => context_menu::notify_failure(&requiem_i18n::t!(
                "message-embed-unable-to-download-file"
            )),
        }
    }
    match on_save {
        Some(on_save) => poll(
            images.clone(),
            on_save,
            attachment.url.clone(),
            attachment.filename.clone(),
            0,
        ),
        None => context_menu::notify_failure(&requiem_i18n::t!(
            "message-embed-unable-to-save-file"
        )),
    }
}

/// Embeds Discord shows as the bare media: direct image links and GIF
/// providers, not a card.
fn is_bare_media(embed: &Embed) -> bool {
    if matches!(embed.kind.as_deref(), Some("image" | "gifv")) {
        return true;
    }
    let gif_host = embed.url.as_deref().is_some_and(|url| {
        let host = url
            .split("://")
            .nth(1)
            .and_then(|rest| rest.split('/').next())
            .unwrap_or_default()
            .to_ascii_lowercase();
        ["tenor.com", "tenor.co", "klipy.com"]
            .iter()
            .any(|gif| host == *gif || host.ends_with(&format!(".{gif}")))
    });
    gif_host && (embed.video.is_some() || embed.image.is_some() || embed.thumbnail.is_some())
}

pub fn should_hide_content(content: &str, embeds: &[Embed]) -> bool {
    let [only] = embeds else { return false };
    is_bare_media(only) && only.url.as_deref().is_some_and(|url| url == content.trim())
}

pub fn embed_cards(
    embeds: &[Embed],
    images: &ImageCache,
    backend: &config::BackendConfig,
    tick: &Signal<u64>,
    video_ctx: Option<VideoContext>,
    viewer: Option<&ViewerLink>,
    measure_cache: Option<&Rc<MeasureCache>>,
    selection_base: u64,
) -> Vec<Node> {
    embeds
        .iter()
        .enumerate()
        .filter_map(|(index, embed)| {
            if is_bare_media(embed) {
                let poster = embed.thumbnail.as_ref().or(embed.image.as_ref());

                #[cfg(feature = "video")]
                if let (Some(video), Some(ctx)) = (embed.video.as_ref(), video_ctx.as_ref()) {
                    let url = video.proxy_url.as_deref().unwrap_or(&video.url);
                    let node = video_player::gifv_video_node(
                        url,
                        embed_poster(poster, images),
                        video.width.or(poster.and_then(|m| m.width)),
                        video.height.or(poster.and_then(|m| m.height)),
                        ctx.videos,
                        cap_width(MAX_MEDIA_WIDTH),
                        MAX_MEDIA_HEIGHT,
                        message_embed::ATTACHMENT_IMAGE,
                    );
                    let poster_url = poster
                        .map(|m| m.proxy_url.as_deref().unwrap_or(&m.url))
                        .unwrap_or_default();
                    return Some(viewable_row(node, poster_url, viewer, None));
                }
                let media = poster?;
                let url = media.proxy_url.as_deref().unwrap_or(&media.url);
                let node = media_node(
                    url,
                    media.width,
                    media.height,
                    media.placeholder.as_deref(),
                    images,
                    cap_width(MAX_MEDIA_WIDTH),
                    MAX_MEDIA_HEIGHT,
                    message_embed::ATTACHMENT_IMAGE,
                    tick,
                )?;
                Some(viewable_row(node, url, viewer, None))
            } else {
                Some(embed_card(
                    embed,
                    images,
                    backend,
                    tick,
                    video_ctx.as_ref(),
                    viewer,
                    measure_cache,
                    selection_base ^ ((index as u64 + 1) << 48),
                ))
            }
        })
        .collect()
}

fn embed_card(
    embed: &Embed,
    images: &ImageCache,
    backend: &config::BackendConfig,
    tick: &Signal<u64>,
    video_ctx: Option<&VideoContext>,
    viewer: Option<&ViewerLink>,
    measure_cache: Option<&Rc<MeasureCache>>,
    selection_base: u64,
) -> Node {
    let accent = match embed.color {
        Some(rgb) => Color::from_argb(
            255,
            ((rgb >> 16) & 0xff) as u8,
            ((rgb >> 8) & 0xff) as u8,
            (rgb & 0xff) as u8,
        ),
        None => EMBED_DEFAULT_ACCENT,
    };

    let mut card = capped_width(
        Node::new("div", EmbedCardWidget { accent })
            .class(message_embed::EMBED_CARD),
        false,
    );

    if let Some(provider) = &embed.provider {
        card = card.child(
            cached_text(
                provider.name.clone(),
                TextVariant::TEXT_XS_NORMAL,
                measure_cache,
                Some(selection_base ^ 1),
            )
            .color(TEXT_DEFAULT)
            .build()
            .class(message_embed::EMBED_PROVIDER),
        );
    }

    if let Some(author) = &embed.author {
        let icon_url = author
            .proxy_icon_url
            .as_deref()
            .or(author.icon_url.as_deref());
        let mut author_row = div().class(message_embed::EMBED_AUTHOR);
        if let Some(icon_url) = icon_url {
            if let Some(node) = media_node(
                icon_url,
                Some(24),
                Some(24),
                None,
                images,
                24.0,
                24.0,
                message_embed::EMBED_AUTHOR_ICON,
                tick,
            ) {
                author_row = author_row.child(node);
            }
        }

        author_row = author_row.child(linked(
            cached_text(
                author.name.clone(),
                TextVariant::TEXT_SM_SEMIBOLD,
                measure_cache,
                Some(selection_base ^ 2),
            )
            .line_height(22.0_f32)
            .color(TEXT_STRONG)
            .build(),
            author.url.as_deref(),
        ));
        card = card.child(author_row);
    }

    if let Some(title) = &embed.title {
        let title_color = if embed.url.is_some() {
            TEXT_ANCHOR
        } else {
            TEXT_STRONG
        };
        card = card.child(
            div()
                .class(message_embed::EMBED_TITLE)
                .child(linked(
                    cached_text(
                        title.clone(),
                        TextVariant::TEXT_MD_SEMIBOLD,
                        measure_cache,
                        Some(selection_base ^ 3),
                    )
                    .color(title_color)
                    .build(),
                    embed.url.as_deref(),
                )),
        );
    }

    if let Some(description) = &embed.description {
        card = card.child(
            div()
                .class(message_embed::EMBED_DESCRIPTION)
                .child(embed_markdown(
                    description,
                    18.0,
                    images,
                    backend,
                    tick,
                    measure_cache,
                    selection_base ^ 4,
                )),
        );
    }

    if let Some(fields) = &embed.fields {
        let field_node = |field: &models::EmbedField, index: u64| {
            div()
                .class(cls::EMBED_FIELD)
                .child(
                    cached_text(
                        field.name.clone(),
                        TextVariant::TEXT_SM_SEMIBOLD,
                        measure_cache,
                        Some(selection_base ^ (0x10 + index * 2)),
                    )
                    .color(TEXT_STRONG)
                    .build()
                    .class(cls::EMBED_FIELD_NAME),
                )
                .child(embed_markdown(
                    &field.value,
                    18.0,
                    images,
                    backend,
                    tick,
                    measure_cache,
                    selection_base ^ (0x11 + index * 2),
                ))
        };
        let mut row: Vec<Node> = Vec::new();
        let flush = |card: Node, row: &mut Vec<Node>| -> Node {
            if row.is_empty() {
                return card;
            }
            card.child(
                div()
                    .class(cls::EMBED_FIELD_ROW)
                    .children(mem::take(row)),
            )
        };
        for (index, field) in fields.iter().enumerate() {
            if field.inline {
                row.push(field_node(field, index.try_into().unwrap()).class(cls::EMBED_FIELD_INLINE));
                if row.len() == 3 {
                    card = flush(card, &mut row);
                }
            } else {
                card = flush(card, &mut row);
                card = card.child(field_node(field, index.try_into().unwrap()));
            }
        }
        card = flush(card, &mut row);
    }

    #[cfg_attr(not(feature = "video"), allow(unused_mut))]
    let mut media_handled = false;
    #[cfg(feature = "video")]
    if let (Some(video), Some(ctx)) = (embed.video.as_ref(), video_ctx) {
        let poster = embed.image.as_ref().or(embed.thumbnail.as_ref());
        let url = video.proxy_url.as_deref().unwrap_or(&video.url);
        let node = video_player::attachment_video_node(
            url,
            embed_poster(poster, images),
            video.width.or(poster.and_then(|m| m.width)),
            video.height.or(poster.and_then(|m| m.height)),
            ctx.videos,
            cap_width(MAX_MEDIA_WIDTH),
            MAX_MEDIA_HEIGHT,
            ctx.fullscreen,
            message_embed::EMBED_IMAGE,
        );
        card = card.child(media_row(node));
        media_handled = true;
    }
    #[cfg(not(feature = "video"))]
    let _ = video_ctx;

    if !media_handled {
        if let Some(media) = embed.image.as_ref().or(embed.thumbnail.as_ref()) {
            let url = media.proxy_url.as_deref().unwrap_or(&media.url);
            if let Some(node) = media_node(
                url,
                media.width,
                media.height,
                media.placeholder.as_deref(),
                images,
                cap_width(MAX_MEDIA_WIDTH),
                MAX_MEDIA_HEIGHT,
                message_embed::EMBED_IMAGE,
                tick,
            ) {
                card = card.child(viewable_row(node, url, viewer, None));
            }
        }
    }

    let stamp = embed
        .timestamp
        .as_deref()
        .and_then(models::embed_timestamp);
    if embed.footer.is_some() || stamp.is_some() {
        let mut footer_row = div().class(message_embed::EMBED_FOOTER);
        let mut text = String::new();
        if let Some(footer) = &embed.footer {
            let icon_url = footer
                .proxy_icon_url
                .as_deref()
                .or(footer.icon_url.as_deref());
            if let Some(icon_url) = icon_url {
                if let Some(node) = media_node(
                    icon_url,
                    Some(20),
                    Some(20),
                    None,
                    images,
                    20.0,
                    20.0,
                    message_embed::EMBED_FOOTER_ICON,
                    tick,
                ) {
                    footer_row = footer_row.child(node);
                }
            }
            text = footer.text.clone();
        }
        if let Some(stamp) = stamp {
            text = if text.is_empty() {
                stamp
            } else {
                format!("{text} \u{2022} {stamp}")
            };
        }
        footer_row = footer_row.child(
            cached_text(
                text,
                TextVariant::TEXT_XS_MEDIUM,
                measure_cache,
                Some(selection_base ^ 5),
            )
            .line_height(16.0_f32)
            .color(TEXT_DEFAULT)
            .build(),
        );
        card = card.child(footer_row);
    }

    card
}

#[cfg(test)]
mod viewer_tests {
    use requiem_ui::event::CursorIcon;
    use requiem_ui::font_system;
    use requiem_ui::layout;
    use requiem_ui::paint as requiem_ui_paint;
    use requiem_ui::style::cascade;
    use requiem_ui::style::parser;
    use std::sync::OnceLock;
    use super::*;
    use crate::models::Id;
    use crate::models::User;
    use crate::ui::Fonts;
    use requiem_ui::event::{MouseButton, UiEvent};
    use requiem_ui::font_system::TextFonts;
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::theme::make_typeface;
    use requiem_ui::widget::WidgetTree;
    use requiem_ui::widget::primitives::BoxWidget;
    use skia_safe::font_style::Weight;
    use taffy::{AvailableSpace, Size};
    use tokio::runtime::Runtime;

    fn attachment(name: &str, content_type: &str) -> Attachment {
        // Ids are snowflakes, so derive a stable numeric one from the name.
        let id = name
            .bytes()
            .fold(1u64, |hash, byte| hash * 31 + u64::from(byte))
            .to_string();
        serde_json::from_value(serde_json::json!({
            "id": id, "filename": name, "size": 1000, "content_type": content_type,
            "width": 400, "height": 200, "url": format!("https://cdn.test/{name}"), "proxy_url": format!("https://media.test/{name}"),
        }))
        .unwrap()
    }

    #[test]
    fn clicking_a_message_image_opens_the_viewer_on_it() {
        let message = Message {
            pinned: false,
            id: Id::new(1),
            channel_id: Id::new(1),
            guild_id: None,
            author: User {
                id: Id::new(1),
                username: "x".into(),
                discriminator: "0".into(),
                global_name: Some("Wumpus".into()),
                avatar: None,
                email: None,
                phone: None,
                verified: None,
                mfa_enabled: None,
                bot: false,
                premium_type: None,
                primary_guild: None,
                avatar_decoration_data: None,
                collectibles: None,
                display_name_styles: None,
            },
            content: String::new(),
            timestamp: "2026-01-01T15:29:00+00:00".into(),
            edited_timestamp: None,
            timestamp_ms: OnceLock::new(),
            mentions: Vec::new(),
            mention_everyone: false,
            mention_roles: Vec::new(),
            embeds: Vec::new(),
            attachments: vec![
                attachment("a.png", "image/png"),
                attachment("notes.txt", "text/plain"),
                attachment("b.png", "image/png"),
            ],
            components: Vec::new(),
            sticker_items: Vec::new(),
            message_snapshots: Vec::new(),
            nonce: None,
            message_reference: None,
            referenced_message: None,
            thread: None,
            reactions: Vec::new(),
            kind: 0,
            flags: 0,
            webhook_id: None,
            interaction: None,
        };
        let runtime = Runtime::new().unwrap();
        let images = ImageCache::new(runtime.handle().clone(), wreq::Client::new(), || {});
        let backend = config::BackendConfig {
            rest_base_url: String::new(),
            gateway_url: String::new(),
            cdn_base_url: String::new(),
        };
        let state = MediaViewerState::inert();
        let link = ViewerLink::for_message(&state, &message, &backend, Some("3:29 PM".into()));
        assert_eq!(link.items.len(), 2, "non-images aren't in the carousel");

        let fonts = Fonts {
            typeface: make_typeface(Weight::MEDIUM),
            text_fonts: TextFonts::new(1024 * 1024),
        };
        font_system::install_ambient(fonts.text_fonts.clone());
        let tick = signal::signal(0u64);
        let nodes = attachment_images(&message.attachments, &images, &tick, None, Some(&link));
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".into());
        for node in nodes {
            tree.mount(root, node);
        }
        let theme = parser::parse_stylesheet(&format!(
            "{}\n.root {{ display: flex; flex-direction: column; width: 800px; height: 800px; }}",
            requiem_ui::BUNDLED_CSS
        ));
        cascade::apply_stylesheet(&mut tree, &theme);
        layout::compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(800.0),
                height: AvailableSpace::Definite(800.0),
            },
        );

        let (x, y) = (550.0 / 2.0 + 2.0 + 60.0, 70.0);
        requiem_ui_paint::dispatch_event(&mut tree, &UiEvent::MouseMoved { x, y });
        assert_eq!(
            requiem_ui_paint::cursor_at(&tree, x, y),
            CursorIcon::Pointer,
            "images read as clickable"
        );
        requiem_ui_paint::dispatch_event(
            &mut tree,
            &UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                shift: false,
            },
        );
        requiem_ui_paint::dispatch_event(
            &mut tree,
            &UiEvent::MouseUp {
                x,
                y,
                button: MouseButton::Left,
            },
        );
        assert!(state.is_open(), "the click opened the viewer");
        assert_eq!(state.index.peek(), 1, "on the image that was clicked");
    }
}
