use crate::ui::classes::message_components;
use crate::ui::classes::message_components as cls;
use crate::utils::link;
use std::cell::Cell;
use std::rc::Rc;

use requiem_ui::colors::{TEXT_DEFAULT, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::components::{ButtonProps, ButtonSize, ButtonVariant, TextProps, TextVariant, div};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::text_cache::MeasureCache;

use super::message_embed::{
    MosaicItem, ViewerLink, cap_width, capped_width, cover_picture, embed_markdown, media_node,
    mosaic, viewable_row,
};
use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::Message;
use crate::models::component::{Component, button_style, component_type};
use crate::utils::format::file_size;

const TEXT_LINE_HEIGHT: f32 = 19.25;
const THUMBNAIL_SIZE: f32 = 85.0;
const GALLERY_WIDTH: f32 = 600.0;
const GALLERY_MAX: usize = 10;
const SINGLE_HEIGHT: f32 = 350.0;

/// Container border and padding on both sides.
const CONTAINER_INSET: f32 = 34.0;

struct Ctx<'a> {
    images: &'a ImageCache,
    backend: &'a BackendConfig,
    tick: &'a Signal<u64>,
    viewer: Option<&'a ViewerLink>,
    measure_cache: Option<&'a Rc<MeasureCache>>,
    key: u64,

    /// Width a gallery can fill: the message's, less any container's inset.
    width: Cell<f32>,
}

/// The message's components, top level first. Legacy rows and Components V2
/// share this renderer.
pub fn cards(
    message: &Message,
    images: &ImageCache,
    backend: &BackendConfig,
    tick: &Signal<u64>,
    viewer: Option<&ViewerLink>,
    measure_cache: Option<&Rc<MeasureCache>>,
) -> Option<Node> {
    if message.components.is_empty() {
        return None;
    }
    let ctx = Ctx {
        images,
        backend,
        tick,
        viewer,
        measure_cache,
        key: message.id.get() ^ (1 << 60),
        width: Cell::new(cap_width(GALLERY_WIDTH)),
    };
    let nodes: Vec<Node> = message
        .components
        .iter()
        .enumerate()
        .filter_map(|(index, component)| node(component, &ctx, index as u64 + 1))
        .collect();
    if nodes.is_empty() {
        return None;
    }
    Some(
        div()
            .class(message_components::COMPONENTS)
            .children(nodes),
    )
}

fn node(component: &Component, ctx: &Ctx, salt: u64) -> Option<Node> {
    match component.kind {
        component_type::TEXT_DISPLAY => Some(text(component.text(), ctx, salt)),
        component_type::SEPARATOR => Some(separator(component)),
        component_type::SECTION => Some(section(component, ctx, salt)),
        component_type::THUMBNAIL => thumbnail(component, ctx),
        component_type::MEDIA_GALLERY => gallery(component, ctx),
        component_type::FILE => file(component),
        component_type::CONTAINER => Some(container(component, ctx, salt)),
        component_type::ACTION_ROW => {
            let children: Vec<Node> = component
                .components
                .iter()
                .filter_map(|child| node(child, ctx, salt))
                .collect();
            (!children.is_empty()).then(|| div().class(cls::ROW).children(children))
        }
        component_type::BUTTON => Some(button(component)),
        kind if (3..=8).contains(&kind) => Some(select(component)),
        _ => None,
    }
}

fn text(content: &str, ctx: &Ctx, salt: u64) -> Node {
    div()
        .class(message_components::TEXT)
        .child(embed_markdown(
            content,
            TEXT_LINE_HEIGHT,
            ctx.images,
            ctx.backend,
            ctx.tick,
            ctx.measure_cache,
            ctx.key ^ (salt << 32),
        ))
}

fn separator(component: &Component) -> Node {
    let mut node = div();
    if component.divider.unwrap_or(true) {
        node = node.class(cls::DIVIDER);
    }
    if component.spacing == Some(2) {
        node = node.class(cls::SPACING_LARGE);
    }
    node
}

fn container(component: &Component, ctx: &Ctx, salt: u64) -> Node {
    let mut node = div().class(cls::CONTAINER);
    if let Some(rgb) = component.accent_color {
        node = node.child(
            div()
                .class(cls::CONTAINER_ACCENT)
                .style(format!("background-color: #{:06x}", rgb & 0xff_ffff)),
        );
    }
    let outer = ctx.width.get();
    ctx.width.set(outer - CONTAINER_INSET);
    for (index, child) in component.components.iter().enumerate() {
        if let Some(child) = self::node(child, ctx, (salt << 8) ^ (index as u64 + 1)) {
            node = node.child(child);
        }
    }
    ctx.width.set(outer);
    capped_width(node, false)
}

fn section(component: &Component, ctx: &Ctx, salt: u64) -> Node {
    let mut texts = div().class(cls::SECTION_TEXT);
    for (index, child) in component.components.iter().enumerate() {
        if child.kind == component_type::TEXT_DISPLAY {
            texts = texts.child(text(child.text(), ctx, (salt << 8) ^ (index as u64 + 1)));
        }
    }
    let mut row = div().class(cls::SECTION).child(texts);
    if let Some(accessory) = component.accessory.as_deref() {
        if accessory.kind == component_type::BUTTON {
            row = row.class(cls::SECTION_CENTERED);
        }
        if let Some(accessory) = node(accessory, ctx, salt) {
            row = row.child(div().class(cls::SECTION_ACCESSORY).child(accessory));
        }
    }
    row
}

fn thumbnail(component: &Component, ctx: &Ctx) -> Option<Node> {
    let media = component.media.as_ref()?;
    Some(
        div()
            .class(message_components::THUMBNAIL)
            .child(cover_picture(
                media.source(),
                media.placeholder.as_deref(),
                THUMBNAIL_SIZE,
                THUMBNAIL_SIZE,
                ctx.images,
                ctx.tick,
            )),
    )
}

fn gallery(component: &Component, ctx: &Ctx) -> Option<Node> {
    let width = ctx.width.get();
    let items: Vec<MosaicItem> = component
        .items
        .iter()
        .take(GALLERY_MAX)
        .map(|item| MosaicItem {
            url: item.media.source(),
            width: item.media.width,
            height: item.media.height,
            placeholder: item.media.placeholder.as_deref(),
            attachment: None,
        })
        .collect();
    let pictures = match items.as_slice() {
        [] => return None,
        [only] => {
            let tile = media_node(
                only.url,
                only.width,
                only.height,
                only.placeholder,
                ctx.images,
                width,
                SINGLE_HEIGHT,
                cls::THUMBNAIL,
                ctx.tick,
            )?;
            viewable_row(tile, only.url, ctx.viewer, None)
        }
        _ => mosaic(&items, width, ctx.images, ctx.tick, ctx.viewer),
    };
    Some(div().class(cls::GALLERY).child(pictures))
}

fn file(component: &Component) -> Option<Node> {
    let media = component.file.as_ref()?;
    let name = component
        .name
        .clone()
        .or_else(|| media.url.rsplit('/').next().map(str::to_string))
        .filter(|name| !name.is_empty())?;
    let url = media.url.clone();
    let mut card = div()
        .class(message_components::FILE)
        .child(
            TextProps::styled(name, TextVariant::TEXT_SM_SEMIBOLD)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
        );
    if let Some(size) = component.size {
        card = card.child(
            TextProps::styled(file_size(size), TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_SUBTLE)
                .build(),
        );
    }
    Some(
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .on_click(move || link::open_url(&url))
            .build()
            .child(card),
    )
}

fn button(component: &Component) -> Node {
    let style = component.style.unwrap_or(button_style::PRIMARY);
    let variant = match style {
        button_style::PRIMARY | button_style::SUCCESS => ButtonVariant::Primary,
        button_style::DANGER => ButtonVariant::CriticalPrimary,
        _ => ButtonVariant::Secondary,
    };
    let mut label = component.label.clone().unwrap_or_default();
    if let Some(emoji) = component
        .emoji
        .as_ref()
        .filter(|emoji| emoji.id.is_none())
        .and_then(|emoji| emoji.name.as_deref())
    {
        label = format!("{emoji} {label}").trim().to_string();
    }
    let url = component.url.clone().filter(|_| style == button_style::LINK);
    let mut props = ButtonProps::default()
        .variant(variant)
        .size(ButtonSize::Small)
        .label(label)
        .disabled(component.disabled);
    if style == button_style::SUCCESS {
        props = props.extra_classes(vec![cls::BUTTON_SUCCESS.to_string()]);
    }
    if let Some(url) = url {
        props = props.on_click(move || link::open_url(&url));
    }
    props.build()
}

fn select(component: &Component) -> Node {
    let placeholder = component.placeholder.clone().unwrap_or_default();
    div()
        .class(message_components::SELECT)
        .child(
            TextProps::styled(placeholder, TextVariant::TEXT_SM_NORMAL)
                .color(TEXT_DEFAULT)
                .line_clamp(1_u32)
                .build(),
        )
}
