use crate::ui::classes::reply as reply_cls;
use crate::ui::classes::message_list as cls;
use crate::channel_lookup;
use crate::gateway::events::GatewayEvent;
use crate::models::id::marker;
use crate::models::message::message_flags;
use crate::models::message::message_type;
use crate::models;
use crate::plugins;
use crate::query;
use crate::rest::endpoints::messages;
use crate::rest::endpoints::reactions as endpoints_reactions;
use crate::state::NavEvent;
use crate::typing_local;
use crate::ui::MessageListScroll;
use crate::ui::author_style::NameFill;
use crate::ui::author_style::TagPlace;
use crate::ui::author_style;
use crate::ui::channel_start;
use crate::ui::classes::author_style as classes_author_style;
use crate::ui::classes::commands_panel as classes_commands_panel;
use crate::ui::classes::message_embed as classes_message_embed;
use crate::ui::classes::message_list;
use crate::ui::classes::pins;
use crate::ui::classes::profile_popout as classes_profile_popout;
use crate::ui::classes::threads as classes_threads;
use crate::ui::code_block::Segment;
use crate::ui::code_block;
use crate::ui::command_composer;
use crate::ui::commands_panel;
use crate::ui::context_menu;
use crate::ui::context_menus;
use crate::ui::dm_start;
use crate::ui::emoji_button_icon::EmojiButtonIconWidget;
use crate::ui::emoji_popout;
use crate::ui::emoji_skin;
use crate::ui::emoji_text;
use crate::ui::home;
use crate::ui::invite_embed;
use crate::ui::join_message;
use crate::ui::media_viewer;
use crate::ui::message_components;
use crate::ui::message_edit;
use crate::ui::message_embed::ViewerLink;
use crate::ui::message_embed;
use crate::ui::message_keys;
use crate::ui::message_toolbar;
use crate::ui::pending_uploads;
use crate::ui::pins as ui_pins;
use crate::ui::reactions;
use crate::ui::reply::AuthorPart;
use crate::ui::reply::Snippet;
use crate::ui::reply;
use crate::ui::role_popout;
use crate::ui::slash_commands;
use crate::ui::threads;
use crate::ui::time_panel;
use crate::ui::twemoji;
use crate::ui::typing_bar;
use crate::ui;
use crate::utils::prefs;
use crate::utils::time as utils_time;
use requiem_ui::app;
use requiem_ui::colors::{MENTION_BACKGROUND, TEXT_MENTION};
use requiem_ui::colors;
use requiem_ui::components::Anchor;
use requiem_ui::components::TextLinkProps;
use requiem_ui::components::TooltipPosition;
use requiem_ui::components::TooltipProps;
use requiem_ui::components::hover_tooltip;
use requiem_ui::components::{ButtonSize, ButtonVariant};
use requiem_ui::components::{HeadingLevel, TextVariant};
use requiem_ui::components::{line_clamp_class, popover_backdrop};
use requiem_ui::components;
use requiem_ui::control;
use requiem_ui::event::NamedKey;
use requiem_ui::icon_handle::IconSpec;
use requiem_ui::icons::{Icon, IconPath};
use requiem_ui::signal;
use requiem_ui::timer;
use requiem_ui::timestamp;
use requiem_ui::widget::primitives::HoverBoxWidget;
use requiem_ui::widget::primitives::ImageWidget;
use requiem_ui::widget::primitives::MentionTarget;
use requiem_ui::widget::primitives;
use std::cell::Cell;
use std::cmp;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::env;
use std::hash::Hash;
use std::ops::Range;
use std::hash::Hasher;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use requiem_ui::components::{
    ButtonProps, HeadingProps, MessageTextProps, TextInputProps, TextProps, div, icon,
    message_text, scroll_view,
};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::style::ComputedStyle;
use requiem_ui::text_cache::MeasureCache;
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::primitives::PresenceStatus;
use requiem_ui::widget::primitives::{
    AnimatedIconWidget, AutocompleteCandidate, AutocompleteTrigger, MarkdownFeatures,
    ScrollPosition, ScrollViewWidget,
};
use requiem_ui::widget::{AbsoluteRect, Widget, WidgetId, WidgetTree};
use skia_safe::font_style::Weight;
use skia_safe::{Canvas, Color};
use tokio::sync::mpsc::UnboundedSender;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::lottie::{asset_json, fix_mask_gaps, recolor_fills_and_strokes};
use crate::models::id::marker::{ChannelMarker, UserMarker};
use crate::models::{Channel, Id, Permissions};
use crate::query::QueryClient;
use crate::rest::HttpClient;
use crate::state::{AppState, NewMarker};
use crate::ui::attachments::{self, AttachmentsState};
use crate::ui::emoji_picker::{self, EmojiPickerState};
use crate::ui::profile_popout::{self, ProfilePopoutState};
use crate::ui::{Fonts, TEXT_DEFAULT, TEXT_PLACEHOLDER, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::icons::{APPS, CHAT_CHECK, FILE_UP, PLUS_LARGE, POLLS, TEXT, THREAD_PLUS};

const GIF_BUTTON_LOTTIE: &str = "lottie/gif_button";

const GIF_BUTTON_IDLE_FRAME: f64 = 0.0;
const GIF_BUTTON_HOVER_START: f64 = 90.0;
const GIF_BUTTON_HOVER_END: f64 = 130.0;
const GIF_BUTTON_FPS: f64 = 60.0;

fn sanitized_gif_button_json(color: Color) -> &'static str {
    static GRAY: OnceLock<String> = OnceLock::new();
    static WHITE: OnceLock<String> = OnceLock::new();

    fn base_pipeline() -> String {
        fix_mask_gaps(asset_json(GIF_BUTTON_LOTTIE))
            .expect("Discord's gif button animation should be valid JSON")
    }

    fn recolored(cache: &'static OnceLock<String>, color: Color) -> &'static str {
        cache.get_or_init(|| {
            let json = base_pipeline();
            let (r, g, b) = (
                f64::from(color.r()) / 255.0,
                f64::from(color.g()) / 255.0,
                f64::from(color.b()) / 255.0,
            );
            recolor_fills_and_strokes(&json, r, g, b).expect("still valid JSON after recoloring")
        })
    }

    if color == TEXT_STRONG {
        recolored(&WHITE, TEXT_STRONG)
    } else {
        recolored(&GRAY, ui::TEXT_SECONDARY)
    }
}

struct GifButtonIconWidget {
    icon_gray: AnimatedIconWidget,
    icon_white: AnimatedIconWidget,
    hover_started_at: Cell<Option<Instant>>,
}

impl GifButtonIconWidget {
    fn new() -> Self {
        Self {
            icon_gray: AnimatedIconWidget::new(
                sanitized_gif_button_json(ui::TEXT_SECONDARY),
                GIF_BUTTON_IDLE_FRAME,
                GIF_BUTTON_IDLE_FRAME,
                false,
            ),
            icon_white: AnimatedIconWidget::new(
                sanitized_gif_button_json(TEXT_STRONG),
                GIF_BUTTON_IDLE_FRAME,
                GIF_BUTTON_IDLE_FRAME,
                false,
            ),
            hover_started_at: Cell::new(None),
        }
    }
}

impl Widget for GifButtonIconWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        if style.is_hovered && self.hover_started_at.get().is_none() {
            self.hover_started_at.set(Some(Instant::now()));
        } else if !style.is_hovered {
            self.hover_started_at.set(None);
        }

        let frame = match self.hover_started_at.get() {
            Some(hover_started_at) => {
                let hover_span = (GIF_BUTTON_HOVER_END - GIF_BUTTON_HOVER_START) / GIF_BUTTON_FPS;
                let hover_elapsed = hover_started_at.elapsed().as_secs_f64();
                GIF_BUTTON_HOVER_START + hover_elapsed.min(hover_span) * GIF_BUTTON_FPS
            }
            None => GIF_BUTTON_IDLE_FRAME,
        };

        let icon = if style.is_hovered {
            &self.icon_white
        } else {
            &self.icon_gray
        };
        icon.set_scene(frame, frame, false);
        icon.paint(canvas, rect, style);
    }
}

#[derive(Clone)]
pub struct ChatColors {
    pub text_default: Color,
    pub text_muted: Color,
    pub text_strong: Color,
    pub text_placeholder: Color,
    pub mention_chip_background: Color,
    pub mention_chip_text: Color,
}

impl ChatColors {
    pub fn desktop() -> Self {
        ChatColors {
            text_default: TEXT_DEFAULT,
            text_muted: TEXT_SUBTLE,
            text_strong: TEXT_STRONG,
            text_placeholder: TEXT_PLACEHOLDER,
            mention_chip_background: MENTION_BACKGROUND,
            mention_chip_text: TEXT_MENTION,
        }
    }
}

const MESSAGE_LINE_HEIGHT: f32 = 22.0;

mod tsx_list;

use crate::ui_tsx::UiAction;
use requiem_tsx::{HostComponents, StateQueries};

pub fn tsx_queries(queries: StateQueries) -> StateQueries {
    tsx_list::queries(queries)
}

pub fn tsx_components(components: HostComponents) -> HostComponents {
    tsx_list::components(components)
}

pub fn handle_tsx(action: UiAction) {
    tsx_list::handle(action);
}

const TSX_LIST_VERSION: u64 = 1;

mod virtualization {
    use std::cell;
    use std::collections;
    use std::cell::RefCell as CellRefCell;
    use std::collections::HashMap as CollectionsHashMap;
    use requiem_ui::widget::primitives;
    use std::rc::Rc as RcRc;
    use std::cell::Cell;
    use requiem_ui::signal;
    use crate::models;
    use crate::models::id::marker;
    use std::time;


    pub const ROW_HEIGHT_START: f32 = 48.0;

    pub const ROW_HEIGHT_CONTINUATION: f32 = 26.0;

    pub const GROUP_SPACING: f32 = 17.0;

    pub const OVERSCAN_PX: f32 = 900.0;

    pub const MUST_BUILD_MARGIN_PX: f32 = 250.0;

    pub const NEW_ROWS_PER_REBUILD: usize = 8;

    pub const MAX_FOLLOW_UPS: u32 = 120;

    pub const ASSUMED_VIEWPORT_PX: f32 = 2400.0;

    pub const PIN_THRESHOLD_PX: f32 = 16.0;

    pub const EDGE_PX: f32 = 400.0;

    pub const LOAD_MORE_BUFFER_PX: f32 = 500.0;

    thread_local! {

        pub static MEASURED: cell::RefCell<collections::HashMap<u64, f32>> = CellRefCell::new(CollectionsHashMap::new());

        pub static WHEEL: (primitives::WheelCarry, Cell<u64>) = (RcRc::new(CellRefCell::new(None)), Cell::new(0));

        pub static LOAD_MORE_ASKED: Cell<Option<u64>> = const { Cell::new(None) };

        pub static WINDOW_TICK: signal::Signal<u64> = signal::signal(0);

        pub static FOLLOW_UPS: Cell<u32> = const { Cell::new(0) };

        pub static PENDING_JUMP: signal::Signal<Option<(models::Id<marker::ChannelMarker>, models::Id<marker::MessageMarker>)>> = signal::signal(None);

        pub static PENDING_PRESENT: signal::Signal<Option<(models::Id<marker::ChannelMarker>, u64)>> = signal::signal(None);
        pub static PRESENT_REQUESTS: Cell<u64> = const { Cell::new(0) };

        pub static JUMPED: signal::Signal<Option<(models::Id<marker::MessageMarker>, time::Instant)>> = signal::signal(None);

        pub static NEWEST_SEEN: cell::RefCell<collections::HashMap<models::Id<marker::ChannelMarker>, models::Id<marker::MessageMarker>>> = CellRefCell::new(CollectionsHashMap::new());
    }
}

pub fn request_jump(
    channel_id: Id<ChannelMarker>,
    message_id: Id<marker::MessageMarker>,
) {
    virtualization::PENDING_JUMP.with(|jump| jump.set(Some((channel_id, message_id))));
}

pub fn jump_to_present(
    message_scroll: &Signal<HashMap<Id<ChannelMarker>, MessageListScroll>>,
    channel_id: Id<ChannelMarker>,
) {
    message_scroll.update(|m| {
        m.insert(channel_id, MessageListScroll::default());
    });
    virtualization::WINDOW_TICK.with(|tick| tick.update(|n| *n += 1));
}

struct MessageRowPlaceholder {
    height: f32,
}

impl Widget for MessageRowPlaceholder {
    fn measure(
        &self,
        _known_dimensions: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::Size<f32> {
        taffy::Size {
            width: 0.0,
            height: self.height,
        }
    }
}

mod skeleton {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use requiem_ui::style::ComputedStyle;
    use requiem_ui::widget::{AbsoluteRect, Widget};
    use skia_safe::{Canvas, Color, Paint, RRect, Rect};

    const GROUPS: usize = 26;
    const GROUP_RANGE: u64 = 4;
    const ATTACHMENTS: usize = 8;
    const GROUP_SPACING: f32 = 16.0;
    const LINE: f32 = 22.0;
    const BLOB_HEIGHT: f32 = 16.0;
    const BLOB_TOP: f32 = 3.0;
    const BLOB_GAP: f32 = 4.0;
    const CONTENT_X: f32 = 72.0;
    const PADDING_RIGHT: f32 = 16.0;
    const AVATAR: f32 = 40.0;
    const AVATAR_X: f32 = 16.0;
    const CONTENT_OPACITY: f32 = 0.06;
    const AVATAR_OPACITY: f32 = 0.08;
    const ATTACHMENT_OPACITY: f32 = 0.03;
    const ATTACHMENT_PADDING: f32 = 6.0;

    struct Group {
        username_width: f32,
        username_opacity: f32,
        lines: Vec<Vec<f32>>,

        attachment: Option<(usize, f32, f32)>,
    }

    pub struct Spec {
        groups: Vec<Group>,
        pub height: f32,
    }

    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        fn range(&mut self, lo: u64, hi: u64) -> u64 {
            lo + self.next() % (hi - lo + 1)
        }

        fn unit(&mut self) -> f32 {
            (self.next() % 10_000) as f32 / 10_000.0
        }
    }

    fn generate(seed: u64) -> Spec {
        let mut rng = Rng(seed | 1);
        let mut groups: Vec<Group> = (0..GROUPS)
            .map(|_| {
                let messages = rng.range(1, GROUP_RANGE) as usize;
                Group {
                    username_width: rng.range(80, 120) as f32,
                    username_opacity: 0.1 + rng.unit() * 0.1,
                    lines: (0..messages)
                        .map(|_| {
                            (0..rng.range(3, 8))
                                .map(|_| rng.range(30, 80) as f32)
                                .collect()
                        })
                        .collect(),
                    attachment: None,
                }
            })
            .collect();
        let mut free: Vec<usize> = (0..GROUPS).collect();
        for _ in 0..ATTACHMENTS {
            let group = free.swap_remove(rng.range(0, free.len() as u64 - 1) as usize);
            let at = rng.range(0, groups[group].lines.len() as u64 - 1) as usize;
            groups[group].attachment =
                Some((at, rng.range(140, 400) as f32, rng.range(100, 320) as f32));
        }
        let height = groups
            .iter()
            .map(|g| {
                GROUP_SPACING
                    + LINE
                    + g.lines.len() as f32 * LINE
                    + g.attachment.map_or(0.0, |(_, _, h)| h + ATTACHMENT_PADDING)
            })
            .sum();
        Spec { groups, height }
    }

    thread_local! {
        static SPECS: RefCell<HashMap<u64, Rc<Spec>>> = RefCell::new(HashMap::new());
    }

    pub fn spec_for(channel_id: u64) -> Rc<Spec> {
        SPECS.with(|specs| {
            specs
                .borrow_mut()
                .entry(channel_id)
                .or_insert_with(|| Rc::new(generate(channel_id)))
                .clone()
        })
    }

    pub struct HistorySkeleton {
        pub spec: Rc<Spec>,
        pub color: Color,
    }

    impl HistorySkeleton {
        fn blob(&self, canvas: &Canvas, rect: Rect, radius: f32, opacity: f32) {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(self.color);
            paint.set_alpha_f(opacity);
            canvas.draw_rrect(RRect::new_rect_xy(rect, radius, radius), &paint);
        }

        fn line(&self, canvas: &Canvas, x: f32, y: f32, right: f32, widths: &[f32], opacity: f32) {
            let mut bx = x;
            for &w in widths {
                if bx + w > right {
                    break;
                }
                self.blob(
                    canvas,
                    Rect::from_xywh(bx, y + BLOB_TOP, w, BLOB_HEIGHT),
                    BLOB_HEIGHT / 2.0,
                    opacity,
                );
                bx += w + BLOB_GAP;
            }
        }
    }

    impl Widget for HistorySkeleton {
        fn measure(
            &self,
            _known: taffy::Size<Option<f32>>,
            _available: taffy::Size<taffy::AvailableSpace>,
        ) -> taffy::Size<f32> {
            taffy::Size {
                width: 0.0,
                height: self.spec.height,
            }
        }

        fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
            canvas.save();
            canvas.clip_rect(
                Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
                None,
                false,
            );
            let (x, right) = (rect.x + CONTENT_X, rect.x + rect.width - PADDING_RIGHT);
            let mut y = rect.y;
            for group in &self.spec.groups {
                y += GROUP_SPACING;
                let circle = Rect::from_xywh(rect.x + AVATAR_X, y + 2.0, AVATAR, AVATAR);
                self.blob(canvas, circle, AVATAR / 2.0, AVATAR_OPACITY);
                self.line(
                    canvas,
                    x,
                    y,
                    right,
                    &[group.username_width],
                    group.username_opacity,
                );
                y += LINE;
                for (i, words) in group.lines.iter().enumerate() {
                    self.line(canvas, x, y, right, words, CONTENT_OPACITY);
                    y += LINE;
                    if let Some((_, w, h)) = group.attachment.filter(|(at, _, _)| *at == i) {
                        self.blob(
                            canvas,
                            Rect::from_xywh(x, y + 4.0, w.min(right - x), h),
                            6.0,
                            ATTACHMENT_OPACITY,
                        );
                        y += h + ATTACHMENT_PADDING;
                    }
                }
            }
            canvas.restore();
        }
    }
}

#[allow(clippy::too_many_arguments)]
struct RebuildTimer(Instant);

fn rows_are_reused() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| env::var("REQUIEM_ROW_REUSE").map_or(true, |v| v != "0"))
}

fn row_version(message: &models::Message, extra: &[u64]) -> u64 {
    let mut h = DefaultHasher::new();
    message.id.get().hash(&mut h);
    message.content.hash(&mut h);
    message.author.id.get().hash(&mut h);
    message.kind.hash(&mut h);
    message.flags.hash(&mut h);
    message.webhook_id.hash(&mut h);
    message.mention_everyone.hash(&mut h);
    message
        .mentions
        .iter()
        .for_each(|u| u.id.get().hash(&mut h));
    message
        .mention_roles
        .iter()
        .for_each(|r| r.get().hash(&mut h));
    for a in &message.attachments {
        (&a.id, &a.filename, a.size, &a.url, a.width, a.height).hash(&mut h);
    }
    if !message.embeds.is_empty() {
        serde_json::to_string(&message.embeds)
            .unwrap_or_default()
            .hash(&mut h);
    }
    for r in &message.reactions {
        (r.count, r.me, &r.emoji.id, &r.emoji.name).hash(&mut h);
    }
    if let Some(reference) = &message.referenced_message {
        (
            reference.id.get(),
            &reference.content,
            reference.author.id.get(),
        )
            .hash(&mut h);
    }
    if let Some(reference) = &message.message_reference {
        serde_json::to_string(reference)
            .unwrap_or_default()
            .hash(&mut h);
    }
    if !message.message_snapshots.is_empty() {
        serde_json::to_string(&message.message_snapshots)
            .unwrap_or_default()
            .hash(&mut h);
    }
    message.interaction.is_some().hash(&mut h);
    extra.hash(&mut h);
    h.finish()
}

fn text_hash(text: &str) -> u64 {
    let mut h = DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

thread_local! {

    static REBUILD_PARTS_MS: Cell<(f64, f64)> = const { Cell::new((0.0, 0.0)) };
}

impl Drop for RebuildTimer {
    fn drop(&mut self) {
        let (rows, composer) = REBUILD_PARTS_MS.with(Cell::take);
        log::debug!(target: "requiem_ui::perf", "message list rebuilt in {:.1}ms (rows {:.1}ms, composer {:.1}ms)", self.0.elapsed().as_secs_f64() * 1000.0, rows, composer);
    }
}

pub fn build_channel_header(
    state: &AppState,
    channel: Option<&Channel>,
    colors: &ChatColors,
    toolbar: Option<Node>,
) -> Node {
    let header_label = match channel.and_then(|c| c.name.as_deref()) {
        Some(name) => name.to_string(),
        None => requiem_i18n::t!("message-list-select-a-channel").to_string(),
    };
    let mut header = div()
        .class(message_list::CHANNEL_HEADER)
        .children([
            icon(
                channel.map_or(TEXT, |c| ui::channel_icon(state, c)),
                colors.text_muted,
            )
            .class(message_list::CHANNEL_HEADER_ICON),
            HeadingProps::default()
                .level(HeadingLevel::H3)
                .variant(TextVariant::HEADING_MD_MEDIUM)
                .content(header_label)
                .color(colors.text_strong)
                .build()
                .class(line_clamp_class(1))
                .class(message_list::CHANNEL_HEADER_TITLE),
        ]);
    if let Some(toolbar) = toolbar {
        header = header
            .child(div().class(message_list::CHANNEL_HEADER_SPACER))
            .child(toolbar);
    }
    header
}

pub fn header_icon_button(
    icon_path: impl IconPath,
    label: String,
    selected: bool,
    on_click: impl FnMut() + 'static,
) -> Node {
    TooltipProps::default()
        .text(label)
        .position(TooltipPosition::Bottom)
        .build(header_icon(icon_path, selected, on_click))
}

/// A header button that opens a popout: selected and without its tooltip
/// while the popout is shown.
pub fn header_popout_button(
    icon_path: impl IconPath,
    label: String,
    shown: bool,
    on_click: impl FnMut() + 'static,
) -> Node {
    if shown {
        header_icon(icon_path, true, on_click)
    } else {
        header_icon_button(icon_path, label, false, on_click)
    }
}

fn header_icon(icon_path: impl IconPath, selected: bool, on_click: impl FnMut() + 'static) -> Node {
    let button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::HEADER_ICON_BTN.to_string()])
        .on_click(on_click)
        .build()
        .child(icon(icon_path, ui::TEXT_SECONDARY).class(cls::HEADER_ICON));
    if selected {
        button.class(cls::HEADER_ICON_BTN_SELECTED)
    } else {
        button
    }
}

/// The state shared by every row of one `build`.
struct RowEnv<'a> {
    state: &'a AppState,
    channel: &'a Channel,
    fonts: &'a Fonts,
    images: &'a ImageCache,
    backend: &'a BackendConfig,
    colors: &'a ChatColors,
    profile_popout: &'a ProfilePopoutState,
    queries: &'a QueryClient,
    attachments: &'a AttachmentsState,
    media_viewer: &'a media_viewer::MediaViewerState,
    measure_cache: &'a Rc<MeasureCache>,
    images_ready: &'a Signal<u64>,
    menu_env: &'a context_menus::MenuEnv,
    guild_roles: &'a [models::Role],
    mentions: &'a Rc<HashMap<u64, String>>,
    channels: &'a Rc<HashMap<u64, String>>,
    role_mentions: &'a Rc<HashMap<u64, (String, Option<Color>)>>,
    emoji_click: &'a primitives::EmojiClickHandler,
    messages: &'a [&'a models::Message],
    plugin_overlays: &'a HashMap<Id<marker::MessageMarker>, plugins::IncomingOutcome>,
    show_original: &'a HashSet<Id<marker::MessageMarker>>,
    editing: Option<Id<marker::MessageMarker>>,
    replying: Option<Id<marker::MessageMarker>>,
    current_user_id: Option<u64>,
    can_add_reactions: bool,
    #[cfg(feature = "video")]
    video_ctx: Option<message_embed::VideoContext<'a>>,
}

/// The author of a message as its row shows them.
struct RowAuthor {
    user: models::User,
    name: String,
    style: author_style::AuthorStyle,
}

/// One message with what the caller worked out about its place in the list.
struct RowInput<'a> {
    message: &'a models::Message,
    starts_group: bool,
    row_key: u64,
    row_version: u64,
    author: RowAuthor,
}

/// The system row announcing a pinned message.
fn pinned_row(env: &RowEnv<'_>, input: &RowInput<'_>) -> Node {
    let channel = env.channel;
    let colors = env.colors;
    let profile_popout = env.profile_popout;
    let menu_env = env.menu_env;
    let messages = env.messages;
    let message = input.message;
    let starts_group = input.starts_group;
    let row_key = input.row_key;
    let row_version = input.row_version;
    let RowAuthor {
        name: author_name,
        style: author_style,
        ..
    } = &input.author;
    let newest = messages.last().is_some_and(|m| m.id == message.id);
    let mut row = Node::new(
        "div",
        HoverBoxWidget::new(
            message_toolbar::on_row_hover(message.id.get()),
        )
        .focusable(newest),
    )
    .class(message_list::MESSAGE_ROW)
    .class(pins::SYSTEM_ROW)
    .dom_id(message.id.get().to_string());
    if starts_group {
        row = row.class(pins::SYSTEM_ROW_START);
    }
    row.child(ui_pins::system_icon(colors.text_muted))
        .child(ui_pins::system_body(
            message,
            author_name,
            author_style.fill.as_ref(),
            colors.text_muted,
            profile_popout,
        ))
        .child(message_toolbar::slot(
            menu_env,
            &Rc::new(message_target(message, channel)),
        ))
        .on_context_menu(message_menu_handler(menu_env, message, channel))
        .retained(row_key, row_version)
}

/// The system row announcing that someone joined.
fn join_row(env: &RowEnv<'_>, input: &RowInput<'_>) -> Node {
    let channel = env.channel;
    let colors = env.colors;
    let profile_popout = env.profile_popout;
    let menu_env = env.menu_env;
    let messages = env.messages;
    let message = input.message;
    let starts_group = input.starts_group;
    let row_key = input.row_key;
    let row_version = input.row_version;
    let RowAuthor {
        name: author_name,
        style: author_style,
        ..
    } = &input.author;
    let newest = messages.last().is_some_and(|m| m.id == message.id);
    let mut row = Node::new(
        "div",
        HoverBoxWidget::new(
            message_toolbar::on_row_hover(message.id.get()),
        )
        .focusable(newest),
    )
    .class(message_list::MESSAGE_ROW)
    .class(pins::SYSTEM_ROW)
    .dom_id(message.id.get().to_string());
    if starts_group {
        row = row.class(pins::SYSTEM_ROW_START);
    }
    row.child(join_message::system_icon())
        .child(join_message::system_body(
            message,
            author_name,
            author_style.fill.as_ref(),
            colors.text_muted,
            profile_popout,
        ))
        .child(message_toolbar::slot(
            menu_env,
            &Rc::new(message_target(message, channel)),
        ))
        .on_context_menu(message_menu_handler(menu_env, message, channel))
        .retained(row_key, row_version)
}

/// The system row announcing a new thread.
fn thread_created_row(env: &RowEnv<'_>, input: &RowInput<'_>) -> Node {
    let channel = env.channel;
    let colors = env.colors;
    let profile_popout = env.profile_popout;
    let menu_env = env.menu_env;
    let messages = env.messages;
    let message = input.message;
    let starts_group = input.starts_group;
    let row_key = input.row_key;
    let row_version = input.row_version;
    let RowAuthor {
        name: author_name,
        style: author_style,
        ..
    } = &input.author;
    let newest = messages.last().is_some_and(|m| m.id == message.id);
    let mut row = Node::new(
        "div",
        HoverBoxWidget::new(
            message_toolbar::on_row_hover(message.id.get()),
        )
        .focusable(newest),
    )
    .class(message_list::MESSAGE_ROW)
    .class(pins::SYSTEM_ROW)
    .dom_id(message.id.get().to_string());
    if starts_group {
        row = row.class(pins::SYSTEM_ROW_START);
    }
    let card = threads::thread_card(message, true);
    if card.is_some() {
        row = row
            .class(classes_threads::HAS_THREAD)
            .child(threads::spine(message, true));
    }
    row.child(threads::system_icon(colors.text_muted))
        .child(threads::system_body(
            message,
            author_name,
            author_style.fill.as_ref(),
            colors.text_muted,
            profile_popout,
            card,
        ))
        .child(message_toolbar::slot(
            menu_env,
            &Rc::new(message_target(message, channel)),
        ))
        .on_context_menu(message_menu_handler(menu_env, message, channel))
        .retained(row_key, row_version)
}

/// The avatar column, plus the start of the message body: the command or reply line above it.
fn leading_and_prefix(env: &RowEnv<'_>, input: &RowInput<'_>) -> (Node, Node) {
    let state = env.state;
    let channel = env.channel;
    let fonts = env.fonts;
    let images = env.images;
    let backend = env.backend;
    let profile_popout = env.profile_popout;
    let menu_env = env.menu_env;
    let guild_roles = env.guild_roles;
    let mentions = env.mentions;
    let channels = env.channels;
    let message = input.message;
    let starts_group = input.starts_group;
    let RowAuthor {
        user: author,
        name: author_name,
        ..
    } = &input.author;
    let leading = if starts_group {
        let image = author.avatar.as_deref().and_then(|hash| {
            let url = backend.avatar_url(message.author.id, hash);
            images.get_or_decode_image(&url)
        });
        let mut avatar_props =
            ui::user_avatar_props(message.author.id, &author.discriminator, fonts);
        if let Some(image) = image {
            avatar_props = avatar_props.image(image);
        }
        let avatar_node = avatar_props
            .build()
            .class(message_list::MESSAGE_AVATAR);
        profile_popout::clickable(
            avatar_node,
            message.id,
            profile_popout,
            Some(message_list::MESSAGE_AVATAR_BTN),
        )
        .on_context_menu(user_menu_handler(
            menu_env,
            message,
            channel,
            author_name.clone(),
        ))
    } else {
        div().class(message_list::MESSAGE_AVATAR_SPACER)
    };

    let replied = starts_group && message.is_reply();

    let command_user = message
        .interaction
        .as_ref()
        .filter(|_| starts_group && !replied)
        .and_then(|interaction| {
            Some((
                serde_json::from_value::<models::User>(
                    interaction.get("user")?.clone(),
                )
                .ok()?,
                interaction.get("name")?.as_str()?.to_string(),
            ))
        });
    let replied = replied || command_user.is_some();
    let leading = if replied {
        leading.class(message_list::MESSAGE_AVATAR_REPLIED)
    } else {
        leading
    };
    let mut body = div().class(message_list::MESSAGE_BODY);
    if let Some((user, command)) = &command_user {
        let runner = state.users.resolve(user);
        let style =
            author_style::resolve(state, guild_roles, channel.guild_id, &runner);
        let mut avatar =
            ui::user_avatar_props(runner.id, &runner.discriminator, fonts);
        if let Some(image) = runner.avatar.as_deref().and_then(|hash| {
            images.get_or_decode_image(&backend.avatar_url(runner.id, hash))
        }) {
            avatar = avatar.image(image);
        }
        let name = author_style::name_node(
            author_display_name(state, channel, &runner),
            TextVariant::TEXT_SM_MEDIUM,
            16.0,
            style.fill.as_ref(),
            Color::from_argb(163, 0xfb, 0xfb, 0xfb),
        );
        let tag = style.tag.as_ref().map(|tag| {
            author_style::tag_chip(
                tag,
                TagPlace::Reply,
                images,
                backend,
            )
        });
        body = body.child(reply::used_line(
            Some(avatar.build()),
            name,
            tag,
            reply::command_chip(command),
        ));
    } else if message.is_reply() && starts_group {
        body = body.child(reply_preview(
            state,
            channel,
            guild_roles,
            message,
            images,
            backend,
            fonts,
            profile_popout,
            mentions,
            channels,
        ));
    } else if message.is_forward() && starts_group {
        body = body.child(forward_context(env, input));
    }
    (leading, body)
}

fn forward_source_name(
    state: &AppState,
    reference: &models::message::MessageReference,
) -> String {
    if let Some(channel_id) = reference.channel_id {
        if let Some((_, channel)) = ui::find_channel(state, channel_id) {
            if let Some(name) = channel.name {
                return name;
            }
        }
        let dm_name = state.dms.list.with_untracked(|dms| {
            dms.iter()
                .find(|dm| dm.id == channel_id)
                .map(|dm| home::dm_title(state, dm))
        });
        if let Some(name) = dm_name {
            return name;
        }
    }
    requiem_i18n::t!("message-list-forwarded-unknown-channel").to_string()
}

fn forward_context(env: &RowEnv<'_>, input: &RowInput<'_>) -> Node {
    let message = input.message;
    if !message.is_forward() {
        return div();
    }
    let Some(reference) = message.message_reference.as_ref() else {
        return div();
    };
    let muted = env.colors.text_muted;
    let mut line = div()
        .class(cls::FORWARD_CONTEXT)
        .child(
            icon(requiem_ui::icons::ARROW_ANGLE_RIGHT_UP, muted)
                .class(cls::FORWARD_ICON),
        )
        .child(
            TextProps::styled(
                requiem_i18n::t!("message-list-forwarded").to_string(),
                TextVariant::TEXT_XS_MEDIUM,
            )
            .color(muted)
            .build()
            .class(cls::FORWARD_LABEL),
        )
        .child(
            TextProps::styled(
                requiem_i18n::t!("message-list-forwarded-from").to_string(),
                TextVariant::TEXT_XS_MEDIUM,
            )
            .color(muted)
            .build(),
        )
        .child(
            TextProps::styled(forward_source_name(env.state, reference), TextVariant::TEXT_XS_MEDIUM)
                .color(colors::TEXT_ANCHOR)
                .line_clamp(1_u32)
                .build()
                .class(cls::FORWARD_SOURCE),
        );
    if let (Some(channel_id), Some(message_id)) = (reference.channel_id, reference.message_id) {
        line = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .on_click(move || request_jump(channel_id, message_id))
            .build()
            .child(line);
    }
    line
}

fn forward_card(env: &RowEnv<'_>, input: &RowInput<'_>) -> Node {
    let Some(snapshot) = input.message.forward_snapshot() else {
        return div();
    };
    let header = div()
        .class(cls::FORWARD_CARD_HEADER)
        .child(
            icon(
                requiem_ui::icons::ARROW_ANGLE_RIGHT_UP,
                env.colors.text_strong,
            )
            .class(cls::FORWARD_ICON),
        )
        .child(
            TextProps::styled(
                requiem_i18n::t!("message-list-forwarded-message").to_string(),
                TextVariant::TEXT_XS_SEMIBOLD,
            )
            .color(env.colors.text_strong)
            .build()
            .class(cls::FORWARD_CARD_TITLE),
        )
        .child(
            TextProps::styled(
                snapshot.format_short_time().unwrap_or_default(),
                TextVariant::TEXT_XS_NORMAL,
            )
            .color(env.colors.text_muted)
            .build()
            .class(cls::FORWARD_CARD_TIME),
        );

    let mut body = div().class(cls::FORWARD_CARD_BODY);
    if !snapshot.content.trim().is_empty() {
        let mut names = (**env.mentions).clone();
        for user in &snapshot.mentions {
            names
                .entry(user.id.get())
                .or_insert_with(|| env.state.users.display_name(user));
        }
        body = body.child(message_text(MessageTextProps {
            content: snapshot.content.clone(),
            size: 16.0,
            color: env.colors.text_default,
            mentions: Rc::new(names),
            channels: env.channels.clone(),
            roles: env.role_mentions.clone(),
            inert: true,
            cache_key: Some(input.message.id.get() ^ (1 << 59)),
            emoji_provider: Some(twemoji::tile),
            custom_emoji: Some(emoji_text::resolver(env.images, env.backend)),
            markdown: Some(MarkdownFeatures::CHAT),
            line_height: Some(MESSAGE_LINE_HEIGHT),
            ..Default::default()
        }));
    }
    if !snapshot.sticker_items.is_empty() {
        let names = snapshot
            .sticker_items
            .iter()
            .map(|sticker| sticker.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        body = body.child(
            TextProps::styled(
                requiem_i18n::t!("message-list-forwarded-stickers", names = names).to_string(),
                TextVariant::TEXT_XS_NORMAL,
            )
            .color(ui::TEXT_SECONDARY)
            .build()
            .class(cls::FORWARD_STICKERS),
        );
    }

    let mut viewer_message = input.message.clone();
    viewer_message.content = snapshot.content.clone();
    viewer_message.attachments = snapshot.attachments.clone();
    viewer_message.embeds = snapshot.embeds.clone();
    let viewer_link = ViewerLink::for_message(
        env.media_viewer,
        &viewer_message,
        env.backend,
        snapshot.format_short_time(),
    )
    .with_menu(attachment_menu_handler(env.menu_env, &viewer_message, env.channel));

    #[cfg(feature = "video")]
    let media = message_embed::attachment_images(
        &snapshot.attachments,
        env.images,
        env.images_ready,
        env.video_ctx,
        Some(&viewer_link),
    );
    #[cfg(not(feature = "video"))]
    let media = message_embed::attachment_images(
        &snapshot.attachments,
        env.images,
        env.images_ready,
        None,
        Some(&viewer_link),
    );
    let mut accessories: Vec<Node> = media.into_iter().collect();
    accessories.extend(message_embed::file_cards(
        &snapshot.attachments,
        false,
        Some(&viewer_link),
        Some(env.measure_cache),
    ));
    #[cfg(feature = "video")]
    accessories.extend(message_embed::embed_cards(
        &snapshot.embeds,
        env.images,
        env.backend,
        env.images_ready,
        env.video_ctx,
        Some(&viewer_link),
        Some(env.measure_cache),
        input.message.id.get() ^ (1 << 58),
    ));
    #[cfg(not(feature = "video"))]
    accessories.extend(message_embed::embed_cards(
        &snapshot.embeds,
        env.images,
        env.backend,
        env.images_ready,
        None,
        Some(&viewer_link),
        Some(env.measure_cache),
        input.message.id.get() ^ (1 << 58),
    ));
    accessories.extend(
        message_components::cards_for(
            &snapshot.components,
            input.message.id.get() ^ (1 << 57),
            env.images,
            env.backend,
            env.images_ready,
            Some(&viewer_link),
            Some(env.measure_cache),
        )
        .into_iter(),
    );
    if !accessories.is_empty() {
        body = body.child(
            div()
                .class(cls::FORWARD_CARD_ACCESSORIES)
                .children(accessories),
        );
    }

    div().class(cls::FORWARD_CARD).children([header, body])
}

/// The name, tags, badges and timestamp that open a group of messages.
fn append_header(env: &RowEnv<'_>, input: &RowInput<'_>, mut body: Node) -> Node {
    let channel = env.channel;
    let images = env.images;
    let backend = env.backend;
    let colors = env.colors;
    let profile_popout = env.profile_popout;
    let menu_env = env.menu_env;
    let message = input.message;
    let starts_group = input.starts_group;
    let RowAuthor {
        user: author,
        name: author_name,
        style: author_style,
        ..
    } = &input.author;
    if starts_group {
        let mut header = div().class(message_list::MESSAGE_HEADER);
        let username_node = author_style::name_node(
            author_name.clone(),
            TextVariant::TEXT_MD_MEDIUM,
            MESSAGE_LINE_HEIGHT,
            author_style.fill.as_ref(),
            colors.text_strong,
        )
        .class(message_list::MESSAGE_HEADER_USERNAME);
        header = header.child(
            profile_popout::clickable(
                username_node,
                message.id,
                profile_popout,
                Some(message_list::MESSAGE_HEADER_USERNAME_BTN),
            )
            .on_context_menu(user_menu_handler(
                menu_env,
                message,
                channel,
                author_name.clone(),
            )),
        );
        if let Some(tag) = &author_style.tag {
            header = header.child(author_style::tag_chip(
                tag,
                TagPlace::Message,
                images,
                backend,
            ));
        }
        if let (Some(role_icon), Some(node)) = (
            author_style.icon.as_ref(),
            author_style.icon.as_ref().and_then(|icon| {
                author_style::role_icon_node(
                    icon,
                    classes_author_style::ROLE_ICON_MESSAGE,
                    images,
                    backend,
                )
            }),
        ) {
            let (name, icon) = (
                author_style.icon_role_name.clone().unwrap_or_default(),
                role_icon.clone(),
            );
            header = header.child(
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![
                        classes_author_style::ROLE_ICON_BUTTON.to_string(),
                    ])
                    .on_click_at(move |rect| {
                        role_popout::open(name.clone(), icon.clone(), rect)
                    })
                    .build()
                    .child(node),
            );
        }

        if author.bot || message.webhook_id.is_some() {
            header = header.child(
                div()
                    .class(classes_profile_popout::BOT_BADGE)
                    .class(message_list::MESSAGE_APP_TAG)
                    .child(
                        TextProps::styled(
                            requiem_i18n::t!("common-app-tag"),
                            TextVariant::TEXT_XS_SEMIBOLD,
                        )
                        .color(Color::WHITE)
                        .build()
                        .class(classes_profile_popout::BOT_BADGE_TEXT),
                    ),
            );
        }

        if let Some(timestamp) = message.format_header_timestamp() {
            header = header.child(
                TextProps::styled(timestamp, TextVariant::TEXT_XS_MEDIUM)
                    .line_height(MESSAGE_LINE_HEIGHT)
                    .color(colors.text_muted)
                    .build(),
            );
        }
        body = body.child(header);
    }
    body
}

/// The message text (or its editor), split around code blocks, and any plugin note.
fn append_content(env: &RowEnv<'_>, input: &RowInput<'_>, mut body: Node) -> Node {
    let state = env.state;
    let channel = env.channel;
    let fonts = env.fonts;
    let images = env.images;
    let backend = env.backend;
    let colors = env.colors;
    let profile_popout = env.profile_popout;
    let queries = env.queries;
    let measure_cache = env.measure_cache;
    let images_ready = env.images_ready;
    let menu_env = env.menu_env;
    let mentions = env.mentions;
    let channels = env.channels;
    let role_mentions = env.role_mentions;
    let emoji_click = env.emoji_click;
    let plugin_overlays = env.plugin_overlays;
    let show_original = env.show_original;
    let editing = env.editing;
    let message = input.message;
    let is_editing = editing == Some(message.id);

    let overlay = plugin_overlays.get(&message.id);
    let rewritten = overlay
        .and_then(|o| o.display_content.as_ref())
        .filter(|_| !show_original.contains(&message.id));
    let content = rewritten.map_or(message.content.as_str(), |r| r.value.as_str());
    if is_editing {
        let (save_env, channel_id, message_id, original) = (
            menu_env.clone(),
            channel.id,
            message.id,
            message.content.clone(),
        );
        let custom = custom_emoji_source(menu_env, channel, images_ready);
        body = body.child(message_edit::editor(
            colors,
            Some(custom),
            Rc::new(move |content| {
                context_menus::save_edit(
                    &save_env, channel_id, message_id, &original, content,
                )
            }),
        ));
    } else if !content.is_empty()
        && !message_embed::should_hide_content(content, &message.embeds)
    {
        let mention_click: Rc<
            dyn Fn(MentionTarget, AbsoluteRect),
        > = {
            let (popout, message_id, queries, inbox) = (
                profile_popout.clone(),
                message.id,
                queries.clone(),
                state.nav.inbox(),
            );
            Rc::new(move |target, at| match target {
                MentionTarget::User(id) => {
                    profile_popout::toggle_for(message_id, Some(Id::new(id)), at, &popout)
                }

                MentionTarget::Channel(id) => {
                    let inbox = inbox.clone();
                    queries.mutate(Vec::new(), move || async move {
                        inbox
                            .lock()
                            .expect("nav inbox poisoned")
                            .push(NavEvent::OpenChannel(Id::new(id)));
                        Ok::<(), String>(())
                    });
                }
                MentionTarget::Role(_) => {}
            })
        };
        let edited_label = message.edited_unix().map(|unix| {
            (
                requiem_i18n::t!("message-edited").to_string(),
                timestamp::format(unix, Some('F')).unwrap_or_default(),
            )
        });
        let text_node = |piece: &str, index: u64, edited: bool| {
            
            message_text(MessageTextProps {
                edited: edited_label.clone().filter(|_| edited),
                content: piece.to_string(),
                size: 16.0,
                color: colors.text_default,
                mentions: mentions.clone(),
                channels: channels.clone(),
                roles: role_mentions.clone(),
                special_mentions: true,
                on_mention: Some(mention_click.clone()),
                measure_cache: Some(measure_cache.clone()),

                cache_key: Some(
                    message.id.get()
                        ^ if rewritten.is_some() { 1 << 63 } else { 0 }
                        ^ (index << 32),
                ),

                emoji_provider: Some(twemoji::tile),
                custom_emoji: Some(emoji_text::resolver(images, backend)),

                emoji_scale: emoji_text::is_jumbo(piece)
                    .then_some(emoji_text::JUMBO_SCALE),
                line_height: Some(MESSAGE_LINE_HEIGHT),
                on_emoji_click: Some(emoji_click.clone()),
                markdown: Some(MarkdownFeatures::CHAT),

                image_tick: (piece.contains("<:") || piece.contains("<a:"))
                    .then(|| images_ready.clone()),
                ..Default::default()
            })
        };
        let segments = code_block::segments(content);
        if segments
            .iter()
            .all(|s| matches!(s, Segment::Text(_)))
        {
            body = body.child(text_node(content, 0, true));
        } else {
            let last = segments.len().saturating_sub(1);
            for (position, segment) in segments.into_iter().enumerate() {
                let index = position as u64;
                match segment {
                    Segment::Text(range) => {
                        let piece = &content[range];
                        if !piece.is_empty() {
                            body = body.child(text_node(piece, index, position == last));
                        }
                    }
                    Segment::Code(block) => {
                        body = body.child(code_block::node(
                            &block,
                            &fonts.text_fonts,
                            Some(message.id.get() ^ (index << 32) ^ (1 << 62)),
                        ));
                    }
                }
            }
        }
    }
    if let Some(note) = overlay.filter(|_| !is_editing).and_then(|o| {
        plugin_note(
            state,
            message.id,
            o,
            show_original.contains(&message.id),
            colors,
        )
    }) {
        body = body.child(note);
    }
    if message.is_forward() && !is_editing {
        body = body.child(forward_card(env, input));
    }
    body
}

/// Attachments, embeds, components, invites and the ephemeral footer.
fn append_accessories(env: &RowEnv<'_>, input: &RowInput<'_>, mut body: Node) -> Node {
    let state = env.state;
    let channel = env.channel;
    let fonts = env.fonts;
    let images = env.images;
    let backend = env.backend;
    let profile_popout = env.profile_popout;
    let attachments = env.attachments;
    let media_viewer = env.media_viewer;
    let measure_cache = env.measure_cache;
    let images_ready = env.images_ready;
    let menu_env = env.menu_env;
    let current_user_id = env.current_user_id;
    #[cfg(feature = "video")]
    let video_ctx = env.video_ctx;
    let message = input.message;
    let viewer_link = ViewerLink::for_message(
        media_viewer,
        message,
        backend,
        message.format_header_timestamp(),
    )
    .with_menu(attachment_menu_handler(menu_env, message, channel))
    .with_file_actions(context_menus::file_actions(
        menu_env,
        message,
        (Some(message.author.id.get()) == current_user_id).then_some(attachments),
    ));
    #[cfg(feature = "video")]
    let media = message_embed::attachment_images(
        &message.attachments,
        images,
        images_ready,
        video_ctx,
        Some(&viewer_link),
    );
    #[cfg(not(feature = "video"))]
    let media = message_embed::attachment_images(
        &message.attachments,
        images,
        images_ready,
        None,
        Some(&viewer_link),
    );
    let files = message_embed::file_cards(
        &message.attachments,
        !media.is_empty(),
        Some(&viewer_link),
        Some(measure_cache),
    );
    #[cfg(feature = "video")]
    let embeds = message_embed::embed_cards(
        &message.embeds,
        images,
        backend,
        images_ready,
        video_ctx,
        Some(&viewer_link),
        Some(measure_cache),
        message.id.get() ^ (1 << 61),
    );
    #[cfg(not(feature = "video"))]
    let embeds = message_embed::embed_cards(
        &message.embeds,
        images,
        backend,
        images_ready,
        None,
        Some(&viewer_link),
        Some(measure_cache),
        message.id.get() ^ (1 << 61),
    );
    let invites = invite_embed::cards(
        menu_env,
        fonts,
        &message.content,
        &profile_popout.overlays.on_open_guild,
    );
    let ephemeral_note = message
        .has_flag(message_flags::EPHEMERAL)
        .then(|| ephemeral_footer(state, message));
    let components = message_components::cards(
        message,
        images,
        backend,
        images_ready,
        Some(&viewer_link),
        Some(measure_cache),
    );
    let accessories: Vec<Node> = media
        .into_iter()
        .chain(files)
        .chain(embeds)
        .chain(components)
        .chain(invites)
        .chain(ephemeral_note)
        .collect();
    if !accessories.is_empty() {
        body = body.child(
            div()
                .class(classes_message_embed::MESSAGE_ACCESSORIES)
                .children(accessories),
        );
    }
    body
}

/// The reaction pills under the message.
fn append_reactions(env: &RowEnv<'_>, input: &RowInput<'_>, mut body: Node) -> Node {
    let channel = env.channel;
    let images = env.images;
    let backend = env.backend;
    let menu_env = env.menu_env;
    let can_add_reactions = env.can_add_reactions;
    let message = input.message;
    let reaction_actions = {
        let (react_env, channel_id, message_id) =
            (menu_env.clone(), channel.id, message.id);
        reactions::ReactionActions {
            react: Rc::new(move |emoji, remove| {
                reactions::react(
                    &react_env,
                    channel_id,
                    message_id,
                    emoji,
                    remove,
                    endpoints_reactions::LOCATION_INLINE_BUTTON,
                )
            }),
            add: {
                let (channel_id, guild_id, message_id) =
                    (channel.id, channel.guild_id, message.id);
                Rc::new(move |anchor| {
                    reactions::open_picker(
                        channel_id, guild_id, message_id, anchor,
                    )
                })
            },
            can_add: can_add_reactions,
        }
    };
    if let Some(reactions) =
        reactions::row(message, images, backend, &reaction_actions)
    {
        body = body.child(reactions);
    }
    body
}

/// A regular message row: avatar, body, hover toolbar and the thread spine.
fn regular_row(env: &RowEnv<'_>, input: &RowInput<'_>) -> Node {
    let channel = env.channel;
    let menu_env = env.menu_env;
    let messages = env.messages;
    let replying = env.replying;
    let current_user_id = env.current_user_id;
    let message = input.message;
    let starts_group = input.starts_group;
    let row_key = input.row_key;
    let row_version = input.row_version;
    let (leading, body) = leading_and_prefix(env, input);
    let body = append_header(env, input, body);
    let body = append_content(env, input, body);
    let body = append_accessories(env, input, body);
    let mut body = append_reactions(env, input, body);
    let is_editing = env.editing == Some(message.id);
    let thread_card = threads::thread_card(message, false);
    let has_thread_card = thread_card.is_some();
    if let Some(card) = thread_card {
        body = body.child(card);
    }

    let newest = messages.last().is_some_and(|m| m.id == message.id);
    let mut row = Node::new(
        "div",
        HoverBoxWidget::new(
            message_toolbar::on_row_hover(message.id.get()),
        )
        .focusable(newest),
    )
    .class(message_list::MESSAGE_ROW)
    .dom_id(message.id.get().to_string());
    if starts_group {
        row = row.class(message_list::MESSAGE_ROW_START);
    }
    if mentions_current_user(message, current_user_id) {
        row = row.class(message_list::MESSAGE_ROW_MENTIONED);
    }
    if is_editing {
        row = row.class(message_list::MESSAGE_ROW_EDITING);
    }
    if replying == Some(message.id)
        || message.has_flag(message_flags::EPHEMERAL)
    {
        row = row.class(reply_cls::MESSAGE_ROW_REPLYING);
    }
    if virtualization::JUMPED
        .with(|j| j.peek())
        .is_some_and(|(id, _)| id == message.id)
    {
        row = row.class(message_list::MESSAGE_ROW_JUMPED);
    }
    if has_thread_card {
        row = row
            .class(classes_threads::HAS_THREAD)
            .child(threads::spine(message, false));
    }
    row = row
        .child(leading)
        .child(body)
        .child(message_toolbar::slot(
            menu_env,
            &Rc::new(message_target(message, channel)),
        ))
        .on_context_menu(message_menu_handler(menu_env, message, channel));

    if !starts_group {
        if let Some(time) = message.format_short_time() {
            let text = TextProps::new(time, 11.0_f32)
                .color(Color::TRANSPARENT)
                .weight(Weight::MEDIUM)
                .build()
                .class(message_list::MESSAGE_HOVER_TIMESTAMP_TEXT);
            row = row.child(
                div()
                    .class(message_list::MESSAGE_HOVER_TIMESTAMP)
                    .child(text),
            );
        }
    }
    row.retained(row_key, row_version)
}

/// A hash of everything a row renders, so an unchanged row can be reused as is.
fn compute_row_version(
    env: &RowEnv<'_>,
    message: &models::Message,
    starts_group: bool,
    author: &RowAuthor,
) -> u64 {
    let state = env.state;
    let channel = env.channel;
    let images = env.images;
    let backend = env.backend;
    let guild_roles = env.guild_roles;
    let mentions = env.mentions;
    let channels = env.channels;
    let messages = env.messages;
    let plugin_overlays = env.plugin_overlays;
    let show_original = env.show_original;
    let editing = env.editing;
    let replying = env.replying;
    let current_user_id = env.current_user_id;
    let can_add_reactions = env.can_add_reactions;
    let RowAuthor {
        user: author,
        name: author_name,
        style: author_style,
        ..
    } = author;
let avatar_loaded = starts_group
    && author.avatar.as_deref().is_some_and(|hash| {
        images
            .get_or_decode_image(&backend.avatar_url(message.author.id, hash))
            .is_some()
    });
    let overlay = plugin_overlays.get(&message.id);
    let reference_name = message
        .referenced_message
        .as_ref()
        .map(|r| {
            text_hash(&author_display_name(
                state,
                channel,
                &state.users.resolve(&r.author),
            ))
        })
        .unwrap_or(0);
    // The replied-to author's avatar: without it a row built before the
    // image arrived keeps the default avatar.
    let reference_avatar = message
        .referenced_message
        .as_ref()
        .map(|r| {
            let replied = state.users.resolve(&r.author);
            let hash = replied.avatar.as_deref().unwrap_or("");
            let loaded = !hash.is_empty()
                && images
                    .get_or_decode_image(&backend.avatar_url(replied.id, hash))
                    .is_some();
            text_hash(hash) ^ u64::from(loaded)
        })
        .unwrap_or(0);
    let extra = [
        text_hash(author_name),
        author_style.version(),
        u64::from(!guild_roles.is_empty()),
        author_style.loaded(images, backend),
        u64::from(author.bot),
        text_hash(author.avatar.as_deref().unwrap_or("")),
        u64::from(avatar_loaded),
        u64::from(starts_group),
        u64::from(editing == Some(message.id)),
        u64::from(replying == Some(message.id)),
        u64::from(mentions_current_user(message, current_user_id)),
        u64::from(
            virtualization::JUMPED
                .with(|j| j.peek())
                .is_some_and(|(id, _)| id == message.id),
        ),
        u64::from(show_original.contains(&message.id)),
        text_hash(
            overlay
                .and_then(|o| o.display_content.as_ref())
                .map_or("", |r| r.value.as_str()),
        ),
        text_hash(&message.format_header_timestamp().unwrap_or_default()),
        text_hash(&message.format_short_time().unwrap_or_default()),
        reference_name,
        reference_avatar,
        channel_lookup::revision(),
        mentions.len() as u64,
        channels.len() as u64,
        u64::from(messages.last().is_some_and(|m| m.id == message.id)),
        u64::from(can_add_reactions),
        u64::from(message.pinned),
        control::locale_epoch(),
    ];
    row_version(message, &extra)
}


struct RowMeta {
    starts_group: bool,
    height: f32,
    y: f32,
    margin: f32,

    day: Option<String>,

    unread_marker: bool,
}

/// Sending a message jumps the list back to the newest one.
fn follow_own_message(
    channel: &Channel,
    messages: &[&models::Message],
    current_user_id: Option<u64>,
    message_scroll: &Signal<HashMap<Id<ChannelMarker>, MessageListScroll>>,
    scroll_state: &mut MessageListScroll,
) {
    if let Some(newest) = messages.last() {
        let previous = virtualization::NEWEST_SEEN
            .with(|seen| seen.borrow_mut().insert(channel.id, newest.id));
        if previous.is_some_and(|p| p != newest.id)
            && Some(newest.author.id.get()) == current_user_id
        {
            scroll_state.position = ScrollPosition::Pinned;
            message_scroll.update(|m| {
                m.insert(channel.id, scroll_state.clone());
            });
        }
    }
}

/// A search hit or reply asked to jump to a message: anchor the list on it when it is loaded, or
/// fetch the window around it first.
fn resolve_pending_jump(
    state: &AppState,
    channel: &Channel,
    messages: &[&models::Message],
    scroll_state: &mut MessageListScroll,
    message_scroll: &Signal<HashMap<Id<ChannelMarker>, MessageListScroll>>,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    images_ready: &Signal<u64>,
) {
    if let Some((jump_channel, jump_message)) = virtualization::PENDING_JUMP.with(Signal::get) {
        if jump_channel == channel.id {
            if messages.iter().any(|m| m.id == jump_message) {
                let (_, window_height) = app::current_window_size();
                let view =
                    (window_height - CHANNEL_HEADER_HEIGHT_PX - COMPOSER_ROW_HEIGHT_PX).max(0.0);
                scroll_state.position =
                    ScrollPosition::Anchored(primitives::ScrollAnchor {
                        key: jump_message.get().to_string(),
                        offset_from_top: (view / 2.0 - 22.0).max(16.0),
                        height: 44.0,
                    });
                message_scroll.update(|m| {
                    m.insert(channel.id, scroll_state.clone());
                });
                let started = Instant::now();
                virtualization::JUMPED.with(|j| j.set(Some((jump_message, started))));

                // Only the newest jump may end its own highlight; an older timer firing
                // mid-flash would switch the new one off and back on.
                timer::set_timeout(JUMP_FLASH, move || {
                    let current = virtualization::JUMPED
                        .with(|j| j.peek())
                        .is_some_and(|(_, at)| at == started);
                    if current {
                        virtualization::JUMPED.with(|j| j.set(None));
                        virtualization::WINDOW_TICK.with(|tick| tick.update(|n| *n += 1));
                    }
                });
                virtualization::PENDING_JUMP.with(|j| j.set(None));
            } else if let Some(http) = http {
                let key = format!("jump_around:{}:{jump_message}", channel.id);
                let (fetch_http, channel_id) = (http.clone(), channel.id);
                let window: query::QueryResult<Vec<models::Message>> = queries.query(
                    &key,
                    Duration::from_secs(30),
                    move || async move {
                        messages::get_channel_messages_around(
                            &fetch_http,
                            channel_id,
                            jump_message,
                            messages::AROUND_PAGE_SIZE,
                        )
                        .await
                    },
                );
                match window.data {
                    Some(page) => {
                        state.replace_window(channel_id, (*page).clone());
                        queries.remove(&key);
                    }
                    None if window.is_hard_error() => {
                        virtualization::PENDING_JUMP.with(|j| j.set(None))
                    }
                    None => {
                        let _ = images_ready.get();
                    }
                }
            }
        }
    }
}

/// "Jump to present" while the newest messages are not loaded: fetch them, then scroll down.
fn resolve_pending_present(
    state: &AppState,
    channel: &Channel,
    message_scroll: &Signal<HashMap<Id<ChannelMarker>, MessageListScroll>>,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    images_ready: &Signal<u64>,
) {
    if let Some((present_channel, request)) = virtualization::PENDING_PRESENT.with(Signal::get) {
        if present_channel == channel.id {
            if let Some(http) = http {
                let key = format!("present:{}:{request}", channel.id);
                let (fetch_http, channel_id) = (http.clone(), channel.id);
                let latest: query::QueryResult<Vec<models::Message>> = queries.query(
                    &key,
                    Duration::from_secs(30),
                    move || async move {
                        messages::get_channel_messages(
                            &fetch_http,
                            channel_id,
                            messages::INITIAL_PAGE_SIZE,
                        )
                        .await
                    },
                );
                match latest.data {
                    Some(page) => {
                        state.set_history(channel_id, (*page).clone());
                        queries.remove(&key);
                        virtualization::PENDING_PRESENT.with(|p| p.set(None));
                        jump_to_present(message_scroll, channel_id);
                    }
                    None if latest.is_hard_error() => {
                        virtualization::PENDING_PRESENT.with(|p| p.set(None))
                    }
                    None => {
                        let _ = images_ready.get();
                    }
                }
            }
        }
    }
}

/// Estimates each row's position: group starts, day dividers, the unread marker and the heights
/// measured so far.
fn compute_row_metas(
    state: &AppState,
    messages: &[&models::Message],
    first_unread: Option<u64>,
) -> Vec<RowMeta> {
    let mut metas: Vec<RowMeta> = Vec::with_capacity(messages.len());
    let mut last_message: Option<&models::Message> = None;
    let mut last_day: Option<chrono::NaiveDate> = None;
    let _ = state.messages.exhausted.get();
    let mut running_y = 0.0f32;
    for message in messages {
        let local = message
            .timestamp_millis()
            .and_then(chrono::DateTime::from_timestamp_millis)
            .map(|t| t.with_timezone(&chrono::Local));
        let this_day = local.map(|t| t.date_naive());
        let new_day = this_day.is_some()
            && match last_day {
                Some(previous) => this_day != Some(previous),

                None => true,
            };
        let day = new_day
            .then(|| local.map(|t| utils_time::date_long(&t)))
            .flatten();
        if this_day.is_some() {
            last_day = this_day;
        }

        let unread_marker = first_unread == Some(message.id.get());

        let starts_group = day.is_some()
            || message.is_reply()
            || last_message.is_none_or(|last| message.starts_group_after(last));
        last_message = Some(message);

        let estimate = if starts_group {
            virtualization::ROW_HEIGHT_START
        } else {
            virtualization::ROW_HEIGHT_CONTINUATION
        };
        let height = virtualization::MEASURED
            .with(|m| m.borrow().get(&message.id.get()).copied())
            .unwrap_or(estimate);

        let divider = if day.is_some() {
            DAY_DIVIDER_HEIGHT_PX
        } else if unread_marker && !starts_group {
            UNREAD_LINE_INLINE_PX
        } else {
            0.0
        };
        let margin = if starts_group {
            virtualization::GROUP_SPACING
        } else {
            0.0
        } + divider;
        let y = running_y + margin;
        metas.push(RowMeta {
            starts_group,
            height,
            y,
            margin,
            day,
            unread_marker,
        });
        running_y = y + height;
    }
    metas
}

/// Near the top of the loaded history: fetch the page before the oldest loaded message.
fn fetch_older_history(
    state: &AppState,
    channel: &Channel,
    messages: &[&models::Message],
    queries: &QueryClient,
    http: Option<&HttpClient>,
    images_ready: &Signal<u64>,
) {
    if let (Some(oldest), Some(http)) = (messages.first().map(|m| m.id), http) {
        let key = format!("history_before:{}:{oldest}", channel.id);
        let (http, channel_id) = (http.clone(), channel.id);
        let page: query::QueryResult<Vec<models::Message>> = queries.query(
            &key,
            Duration::from_secs(30),
            move || async move {
                messages::get_channel_messages_before(
                    &http,
                    channel_id,
                    oldest,
                    messages::OLDER_PAGE_SIZE,
                )
                .await
            },
        );
        match page.data {
            Some(older) => {
                state.prepend_history(channel_id, (*older).clone());
                queries.remove(&key);
            }

            None => {
                let _ = images_ready.get();
            }
        }
    }
}

/// Above the first loaded message: the loading skeleton, or the channel's beginning.
fn mount_history_header(
    tree: &mut WidgetTree,
    list_id: WidgetId,
    state: &AppState,
    channel: &Channel,
    has_more_before: bool,
    colors: &ChatColors,
    profile_popout: &ProfilePopoutState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) {
    if has_more_before {
        let skeleton = skeleton::HistorySkeleton {
            spec: skeleton::spec_for(channel.id.get()),
            color: colors.text_default,
        };
        tree.mount(
            list_id,
            Node::new("div", skeleton).class(message_list::HISTORY_SKELETON),
        );
    } else {
        tree.mount(
            list_id,
            div().class(message_list::MESSAGE_LIST_TOP_FILL),
        );

        let start = channel_start::build(state, channel, colors).or_else(|| {
            let dm = state.dms.find(channel.id)?;
            Some(dm_start::build(
                state,
                &dm,
                colors,
                profile_popout,
                fonts,
                images,
                backend,
                queries,
                http,
            ))
        });
        if let Some(start) = start {
            tree.mount(list_id, start);
        }
    }
}

/// Rows that were deferred to keep this rebuild short get built over the next few frames.
fn schedule_follow_up(deferred_rows: usize) {
    if deferred_rows > 0 {
        let follow_ups = virtualization::FOLLOW_UPS.with(|n| {
            n.set(n.get() + 1);
            n.get()
        });
        if follow_ups <= virtualization::MAX_FOLLOW_UPS {
            timer::request_animation_frame(|| {
                virtualization::WINDOW_TICK.with(|tick| tick.update(|n| *n += 1))
            });
        }
    } else {
        virtualization::FOLLOW_UPS.with(|n| n.set(0));
    }
}

/// The pill shown while the list is detached from the newest messages.
fn mount_jump_to_present(
    tree: &mut WidgetTree,
    main_id: WidgetId,
    channel_id: Id<ChannelMarker>,
    colors: &ChatColors,
) {
    let jump = ButtonProps::default()
        .variant(ButtonVariant::Primary)
        .size(ButtonSize::Small)
        .label(requiem_i18n::t!("message-list-jump-to-present").to_string())
        .on_click(move || {
            let request = virtualization::PRESENT_REQUESTS.with(|n| {
                n.set(n.get() + 1);
                n.get()
            });
            virtualization::PENDING_PRESENT.with(|p| p.set(Some((channel_id, request))));
        })
        .build();
    let label = TextProps::styled(
        requiem_i18n::t!("message-list-you-re-viewing-older-messages").to_string(),
        TextVariant::TEXT_SM_MEDIUM,
    )
    .color(colors.text_default)
    .build()
    .class(message_list::JUMP_PILL_LABEL);
    tree.mount(
        main_id,
        div()
            .class(message_list::JUMP_PILL)
            .child(label)
            .child(jump),
    );
}

/// The bar above the list saying how many messages arrived while away, with jump and mark-read.
fn mount_new_messages_bar(
    tree: &mut WidgetTree,
    main_id: WidgetId,
    state: &AppState,
    channel_id: Id<ChannelMarker>,
    marker: &NewMarker,
) {
    let first = Id::<marker::MessageMarker>::new(
        marker.first_unread.unwrap_or(marker.acked_at_open),
    );
    let text = new_messages_text(
        marker.count,
        marker.estimated,
        first.created_at_millis(),
        chrono::Local::now(),
    );
    let white = Color::WHITE;
    let jump = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::NEW_BAR_MAIN.to_string()])
        .on_click(move || request_jump(channel_id, first))
        .build()
        .child(
            TextProps::styled(text, TextVariant::TEXT_SM_MEDIUM)
                .color(white)
                .line_clamp(1_u32)
                .build(),
        );
    let read_state = state.clone();
    let mark = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::NEW_BAR_READ.to_string()])
        .on_click(move || read_state.read_states.mark_channel_read(channel_id))
        .build()
        .child(
            TextProps::styled(
                requiem_i18n::t!("message-list-mark-as-read"),
                TextVariant::TEXT_SM_SEMIBOLD,
            )
            .color(white)
            .line_clamp(1_u32)
            .build(),
        )
        .child(icon(CHAT_CHECK, white).class(cls::NEW_BAR_ICON));
    let bar = div().class(cls::NEW_BAR).child(jump).child(mark);
    tree.mount(main_id, bar);
}

/// The profile popout opened from a message's avatar or name.
fn mount_profile_popout(
    tree: &mut WidgetTree,
    main_id: WidgetId,
    state: &AppState,
    channel: &Channel,
    author_id: Id<UserMarker>,
    webhook_author: Option<&models::User>,
    current_user_id: Option<u64>,
    profile_popout: &ProfilePopoutState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) {
    let anchor =
        Anchor::new(profile_popout.overlays.popout_anchor.peek());

    let popout_state = profile_popout.clone();
    tree.mount(
        main_id,
        popover_backdrop(move || profile_popout::close_popouts(&popout_state)),
    );

    let status = channel
        .guild_id
        .and_then(|guild_id| state.members.presence_of(guild_id, author_id))
        .unwrap_or(if current_user_id == Some(author_id.get()) {
            PresenceStatus::Online
        } else {
            PresenceStatus::Offline
        });
    let viewer = state
        .users
        .current
        .with_untracked(|u| u.as_ref().map(|u| u.id));
    let custom_status = if viewer == Some(author_id) {
        state.presence.custom_status.get()
    } else {
        channel
            .guild_id
            .and_then(|guild_id| state.members.custom_status_of(guild_id, author_id))
    };
    let popout = if let Some(webhook) = webhook_author {
        profile_popout::build_webhook(webhook, anchor, fonts, images, backend, profile_popout)
            .focus_scope()
    } else {
        profile_popout::build(
            author_id,
            channel.guild_id,
            viewer,
            status,
            custom_status.as_ref(),
            anchor,
            fonts,
            images,
            backend,
            queries,
            http,
            state,
            profile_popout,
        )
        .focus_scope()
    };
    tree.mount(main_id, popout);
}

/// Which rows are in the window, from the scroll position and the estimated row heights.
struct ListWindow {
    total_estimated_height: f32,
    viewport_px: f32,
    viewport_top: f32,
    visible_start: usize,
    visible_end: usize,
    has_more_before: bool,
    real_view_height: f32,
    view_top_from_oldest: f32,
}

fn compute_window(
    state: &AppState,
    channel_id: Id<ChannelMarker>,
    messages: &[&models::Message],
    metas: &[RowMeta],
    scroll_state: &MessageListScroll,
) -> ListWindow {
    let total_estimated_height = metas.last().map_or(0.0, |m| m.y + m.height);
    let index_of = |key: &str| {
        key.parse::<u64>()
            .ok()
            .and_then(|id| messages.iter().position(|m| m.id.get() == id))
    };

    let viewport_px = {
        let (_, window_height) = app::current_window_size();
        let real = window_height - CHANNEL_HEADER_HEIGHT_PX - COMPOSER_ROW_HEIGHT_PX;
        if real > 200.0 {
            real
        } else {
            virtualization::ASSUMED_VIEWPORT_PX
        }
    };
    let pinned_top = (total_estimated_height - viewport_px).max(0.0);
    let viewport_top = match &scroll_state.position {
        ScrollPosition::Pinned => pinned_top,
        ScrollPosition::Anchored(anchor) => index_of(&anchor.key).map_or(pinned_top, |i| {
            (metas[i].y - anchor.offset_from_top).max(0.0)
        }),
    };
    let window_start = (viewport_top - virtualization::OVERSCAN_PX).max(0.0);
    let window_end = viewport_top + viewport_px + virtualization::OVERSCAN_PX;

    let visible_start = metas
        .iter()
        .position(|m| m.y + m.height >= window_start && m.y <= window_end)
        .unwrap_or(metas.len());
    let visible_end = metas
        .iter()
        .rposition(|m| m.y + m.height >= window_start && m.y <= window_end)
        .map_or(visible_start, |i| i + 1);

    let has_more_before = !state.messages.exhausted.get().contains(&channel_id);
    let (_, window_height) = app::current_window_size();
    let real_view_height =
        (window_height - CHANNEL_HEADER_HEIGHT_PX - COMPOSER_ROW_HEIGHT_PX).max(0.0);
    let view_top_from_oldest = match &scroll_state.position {
        ScrollPosition::Pinned => total_estimated_height - real_view_height,
        ScrollPosition::Anchored(anchor) => {
            index_of(&anchor.key).map_or(f32::MAX, |i| metas[i].y - anchor.offset_from_top)
        }
    };
    ListWindow {
        total_estimated_height,
        viewport_px,
        viewport_top,
        visible_start,
        visible_end,
        has_more_before,
        real_view_height,
        view_top_from_oldest,
    }
}

/// The scroll view for the window: pinning, load-more, the measured row heights and which
/// scroll positions need a rebuild.
fn list_scroll_widget(
    state: &AppState,
    message_scroll: &Signal<HashMap<Id<ChannelMarker>, MessageListScroll>>,
    channel_id: Id<ChannelMarker>,
    scroll_state: &MessageListScroll,
    messages: &[&models::Message],
    metas: &[RowMeta],
    window: &ListWindow,
) -> ScrollViewWidget {
    let ListWindow {
        total_estimated_height,
        visible_start,
        visible_end,
        has_more_before,
        real_view_height,
        ..
    } = *window;
    let message_count = messages.len();
    let message_ids: Rc<Vec<u64>> = Rc::new(messages.iter().map(|m| m.id.get()).collect());
    let row_tops: Rc<Vec<f32>> = Rc::new(metas.iter().map(|m| m.y).collect());
    let pinned_view_top = total_estimated_height - real_view_height;
    let scroll_write = message_scroll.clone();
    let scroll_channel_id = channel_id;

    let wheel_carry = virtualization::WHEEL.with(|(carry, channel)| {
        if channel.replace(scroll_channel_id.get()) != scroll_channel_id.get() {
            *carry.borrow_mut() = None;
        }
        carry.clone()
    });

    let window_top_y = metas.get(visible_start).map_or(0.0, |m| m.y);
    let window_bottom_y = visible_end
        .checked_sub(1)
        .and_then(|i| metas.get(i))
        .map_or(0.0, |m| m.y + m.height);

    state
        .read_states
        .set_at_bottom(matches!(scroll_state.position, ScrollPosition::Pinned));
    let read_states = state.read_states.clone();
    ScrollViewWidget::new()
        .with_wheel_carry(wheel_carry)
        .with_chat_position(
            scroll_state.position.clone(),
            virtualization::PIN_THRESHOLD_PX,
        )
        .with_on_position(move |position| {
            read_states.set_at_bottom(matches!(position, ScrollPosition::Pinned));
            scroll_write.update(|m| {
                m.insert(
                    scroll_channel_id,
                    MessageListScroll {
                        position: position.clone(),
                    },
                );
            });

            let (near_edge, view_top) = match position {
                ScrollPosition::Pinned => (visible_end < message_count, pinned_view_top),
                ScrollPosition::Anchored(anchor) => match anchor
                    .key
                    .parse::<u64>()
                    .ok()
                    .and_then(|id| message_ids.iter().position(|&m| m == id))
                {
                    Some(i) => {
                        let top = row_tops[i] - anchor.offset_from_top;
                        (
                            (visible_start > 0 && top - window_top_y < virtualization::EDGE_PX)
                                || (visible_end < message_count
                                    && window_bottom_y - (top + real_view_height)
                                        < virtualization::EDGE_PX),
                            top,
                        )
                    }
                    None => (true, f32::MAX),
                },
            };

            let loading_more = has_more_before && view_top < virtualization::LOAD_MORE_BUFFER_PX;
            let ask_for_more = virtualization::LOAD_MORE_ASKED.with(|asked| {
                let channel = Some(scroll_channel_id.get());
                let ask = loading_more && asked.get() != channel;
                asked.set(if loading_more { channel } else { None });
                ask
            });
            if near_edge || ask_for_more {
                virtualization::WINDOW_TICK.with(|tick| tick.update(|n| *n += 1));
            }
        })
        .with_on_measure(|rows| {
            virtualization::MEASURED.with(|m| {
                let mut m = m.borrow_mut();
                for (key, height) in rows {
                    if let Ok(id) = key.parse::<u64>() {
                        m.insert(id, *height);
                    }
                }
            });
        })
}

fn message_menu_env(
    state: &AppState,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    images: &ImageCache,
    media_viewer: &media_viewer::MediaViewerState,
) -> context_menus::MenuEnv {
    context_menus::MenuEnv {
        backend: backend.clone(),
        queries: queries.clone(),
        http: http.cloned(),
        images: images.clone(),
        on_save: media_viewer.on_save.clone(),
        own_user_id: state
            .users
            .current
            .with_untracked(|u| u.as_ref().map(|u| u.id)),
        state: state.clone(),
    }
}

/// The day divider and unread line above a row.
fn day_divider(meta: &RowMeta, colors: &ChatColors) -> Node {
        let mut line = match &meta.day {
            Some(_) => div().class(cls::DAY_DIVIDER),
            None if meta.starts_group => div()
                .class(cls::UNREAD_LINE)
                .class(cls::UNREAD_LINE_BEFORE_GROUP),
            None => div().class(cls::UNREAD_LINE).class(cls::UNREAD_LINE_INLINE),
        };
        if meta.unread_marker {
            line = line.class(cls::DAY_DIVIDER_UNREAD);
        }
        if let Some(day) = &meta.day {
            let color = if meta.unread_marker {
                colors::BADGE_NOTIFICATION
            } else {
                colors.text_muted
            };
            let label = TextProps::styled(day.clone(), TextVariant::TEXT_XS_SEMIBOLD)
                .line_height(13.0_f32)
                .color(color)
                .build();
            line = line.child(div().class(cls::DAY_DIVIDER_LABEL_BOX).child(label));
        }
        if meta.unread_marker {
            line = line.child(new_pill());
        }
        line
}

/// What the row loop reports back.
struct RowsOutcome {
    /// The message whose author's profile popout is open, if it is among the rows.
    popout_target: Option<(Id<UserMarker>, Option<models::User>)>,
    /// Rows replaced by placeholders to keep this rebuild short.
    deferred_rows: usize,
}

/// Mounts the rows of the window (reusing unchanged ones), with their dividers and pending-upload
/// notices.
fn mount_rows(
    tree: &mut WidgetTree,
    list_id: WidgetId,
    env: &RowEnv<'_>,
    metas: &[RowMeta],
    visible: Range<usize>,
    must_build: (f32, f32),
    profile_popout_open: Option<Id<marker::MessageMarker>>,
) -> RowsOutcome {
    let RowEnv {
        state,
        channel,
        profile_popout,
        guild_roles,
        colors,
        attachments,
        messages,
        ..
    } = *env;
    let (visible_start, visible_end) = (visible.start, visible.end);
    let (must_build_from, must_build_to) = must_build;
    let (mut built_ahead, mut deferred_rows) = (0usize, 0usize);
    let mut popout_target: Option<(Id<UserMarker>, Option<models::User>)> = None;
    for (message, meta) in messages[visible_start..visible_end]
        .iter()
        .copied()
        .zip(metas[visible_start..visible_end].iter())
    {
        let starts_group = meta.starts_group;
        if profile_popout_open == Some(message.id) {
            let webhook_author = (message.webhook_id.is_some()
                && profile_popout.overlays.popout_user.peek().is_none())
            .then(|| state.users.resolve(&message.author));
            popout_target = Some((
                profile_popout
                    .overlays
                    .popout_user
                    .peek()
                    .unwrap_or(message.author.id),
                webhook_author,
            ));
        }

        let author = state.users.resolve(&message.author);
        let name = author_display_name(state, channel, &author);
        let style = author_style::resolve(state, guild_roles, channel.guild_id, &author);
        let author = RowAuthor {
            user: author,
            name,
            style,
        };
        let row_key = message.id.get();
        let row_version = compute_row_version(env, message, starts_group, &author);
        let input = RowInput {
            message,
            starts_group,
            row_key,
            row_version,
            author,
        };
        let reusable =
            rows_are_reused() && control::retained_available(row_key, row_version);

        if rows_are_reused()
            && !reusable
            && (meta.y + meta.height < must_build_from || meta.y > must_build_to)
        {
            if built_ahead >= virtualization::NEW_ROWS_PER_REBUILD {
                deferred_rows += 1;
                tree.mount(
                    list_id,
                    Node::new(
                        "div",
                        MessageRowPlaceholder {
                            height: meta.margin + meta.height,
                        },
                    )
                    .class(message_list::MESSAGE_LIST_SPACER),
                );
                continue;
            }
            built_ahead += 1;
        }
        let row = if reusable {
            Node::reuse(row_key, row_version)
        } else if message.kind == message_type::CHANNEL_PINNED_MESSAGE {
            pinned_row(env, &input)
        } else if message.kind == message_type::USER_JOIN {
            join_row(env, &input)
        } else if message.kind == message_type::THREAD_CREATED {
            thread_created_row(env, &input)
        } else {
            regular_row(env, &input)
        };

        if meta.day.is_some() || meta.unread_marker {
            tree.mount(list_id, day_divider(meta, colors));
        }
        tree.mount(list_id, row);
        for notice in pending_uploads::notices_after(attachments, channel.id, message.id)
        {
            tree.mount(list_id, notice);
        }
    }
    RowsOutcome {
        popout_target,
        deferred_rows,
    }
}

pub fn build(
    tree: &mut WidgetTree,
    parent: WidgetId,
    state: &AppState,
    channel: Option<&Channel>,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    compose_tx: &UnboundedSender<(Id<ChannelMarker>, String)>,
    composer_focused: &Signal<bool>,
    composer_has_text: &Signal<bool>,
    composer_send_toggle: &Signal<ToggleState>,
    message_scroll: &Signal<HashMap<Id<ChannelMarker>, MessageListScroll>>,
    measure_cache: &Rc<MeasureCache>,
    colors: &ChatColors,
    emoji_picker: &EmojiPickerState,
    attachments: &AttachmentsState,
    media_viewer: &media_viewer::MediaViewerState,
    images_ready: &Signal<u64>,
    profile_popout: &ProfilePopoutState,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    #[cfg(feature = "video")] video_ctx: Option<message_embed::VideoContext>,
) {
    let mut main = div().class(message_list::MAIN_CONTENT);
    if let Some(channel) = channel.filter(|_| tsx_list::enabled()) {
        main = main.on_mount(tsx_list::navigation(channel.id));
    }
    let main_id = tree.mount(parent, main);

    let Some(channel) = channel else {
        tree.mount(
            main_id,
            scroll_view().class(message_list::MESSAGE_LIST),
        );
        return;
    };

    let _rebuild_timer = RebuildTimer(Instant::now());

    if tsx_list::enabled() {
        tsx_list::install(tsx_list::ListCtx {
            state: state.clone(),
            fonts: fonts.clone(),
            images: images.clone(),
            backend: backend.clone(),
            colors: colors.clone(),
            profile_popout: profile_popout.clone(),
            queries: queries.clone(),
            attachments: attachments.clone(),
            media_viewer: media_viewer.clone(),
            measure_cache: measure_cache.clone(),
            images_ready: images_ready.clone(),
            message_scroll: message_scroll.clone(),
            http: http.cloned(),
            global_tick: images_ready.clone(),
            #[cfg(feature = "video")]
            videos: video_ctx.map(|ctx| (ctx.videos.clone(), ctx.fullscreen.clone())),
        });
        let key = channel.id.get() ^ 0x5453_5800_0000_0000;
        let list = if control::retained_available(key, TSX_LIST_VERSION) {
            Some(Node::reuse(key, TSX_LIST_VERSION))
        } else {
            tsx_list::list_node(channel.id).map(|node| node.retained(key, TSX_LIST_VERSION))
        };
        if let Some(list) = list {
            tree.mount(main_id, list);
            let menu_env = message_menu_env(state, backend, queries, http, images, media_viewer);
            message_keys::set_env(menu_env.clone());
            if state.messages.detached.get().contains(&channel.id) {
                mount_jump_to_present(tree, main_id, channel.id, colors);
            }
            if let Some(marker) = state
                .read_states
                .new_marker(channel.id)
                .filter(|marker| marker.count > 0)
            {
                mount_new_messages_bar(tree, main_id, state, channel.id, &marker);
            }
            tree.mount(main_id, reactions::picker_overlay(&menu_env, images_ready));
            tree.mount(main_id, hover_tooltip::hover_tooltip_layer());
            build_composer(
                tree,
                main_id,
                state,
                &menu_env,
                channel,
                images,
                images_ready,
                compose_tx,
                composer_focused,
                composer_has_text,
                composer_send_toggle,
                colors,
                emoji_picker,
                attachments,
                message_scroll,
                media_viewer,
                fonts,
            );
            if let Some(opened) = profile_popout.open.get() {
                let _ = profile_popout.overlays.popout_user.get();
                let _ = images_ready.get();
                let _ = profile_popout.window_resize_tick.get();
                let snapshot = state.messages.in_channel(channel.id);
                if let Some(message) = snapshot.iter().find(|message| message.id == opened) {
                    let popout_user = profile_popout.overlays.popout_user.peek();
                    let webhook_author = (message.webhook_id.is_some() && popout_user.is_none())
                        .then(|| state.users.resolve(&message.author));
                    mount_profile_popout(
                        tree,
                        main_id,
                        state,
                        channel,
                        popout_user.unwrap_or(message.author.id),
                        webhook_author.as_ref(),
                        state.users.current.get().as_ref().map(|user| user.id.get()),
                        profile_popout,
                        fonts,
                        images,
                        backend,
                        queries,
                        http,
                    );
                }
            }
            return;
        }
    }

    let _ = virtualization::WINDOW_TICK.with(Signal::get);

    let _ = state.read_states.marker_revision(channel.id).get();
    let first_unread = state
        .read_states
        .new_marker(channel.id)
        .and_then(|marker| marker.first_unread);

    let editing = message_edit::target()
        .filter(|(edit_channel, _)| *edit_channel == channel.id)
        .map(|(_, id)| id);

    let replying = reply::target_in(channel.id).map(|t| t.message_id);

    let channel_messages = state.messages.in_channel(channel.id);

    let plugin_overlays = state.plugins.overlays.get();
    let show_original = state.plugins.show_original.get();
    let messages: Vec<&models::Message> = channel_messages
        .iter()
        .filter(|m| {
            !plugin_overlays
                .get(&m.id)
                .is_some_and(|o| o.hidden.is_some())
        })
        .collect();
    let current_user_id = state.users.current.get().as_ref().map(|u| u.id.get());

    let mut scroll_state = message_scroll
        .with_untracked(|m| m.get(&channel.id).cloned())
        .unwrap_or_default();

    follow_own_message(channel, &messages, current_user_id, message_scroll, &mut scroll_state);

    resolve_pending_jump(
        state,
        channel,
        &messages,
        &mut scroll_state,
        message_scroll,
        queries,
        http,
        images_ready,
    );

    resolve_pending_present(state, channel, message_scroll, queries, http, images_ready);

    let mentions = Rc::new(mention_names(state, channel));
    let channels = Rc::new(channel_names(state, channel));

    let emoji_click: primitives::EmojiClickHandler = {
        let guild_id = channel.guild_id;
        Rc::new(move |click| emoji_popout::open(click, guild_id))
    };

    let profile_popout_open = profile_popout.open.get();
    if profile_popout_open.is_some() {
        let _ = profile_popout.overlays.popout_user.get();
        let _ = images_ready.get();

        let _ = profile_popout.window_resize_tick.get();
    }

    let metas = compute_row_metas(state, &messages, first_unread);
    let ListWindow {
        total_estimated_height,
        viewport_px,
        viewport_top,
        visible_start,
        visible_end,
        has_more_before,
        real_view_height,
        view_top_from_oldest,
    } = compute_window(state, channel.id, &messages, &metas, &scroll_state);

    if has_more_before && view_top_from_oldest < virtualization::LOAD_MORE_BUFFER_PX {
        fetch_older_history(state, channel, &messages, queries, http, images_ready);
    }

    let list_widget = list_scroll_widget(
        state,
        message_scroll,
        channel.id,
        &scroll_state,
        &messages,
        &metas,
        &ListWindow {
            total_estimated_height,
            viewport_px,
            viewport_top,
            visible_start,
            visible_end,
            has_more_before,
            real_view_height,
            view_top_from_oldest,
        },
    );
    let list_id = tree.mount(
        main_id,
        Node::new("div", list_widget).class(message_list::MESSAGE_LIST),
    );

    mount_history_header(
        tree,
        list_id,
        state,
        channel,
        has_more_before,
        colors,
        profile_popout,
        fonts,
        images,
        backend,
        queries,
        http,
    );

    if visible_start > 0 {
        let leading_height = metas
            .get(visible_start)
            .map_or(total_estimated_height, |m| m.y - m.margin);
        let spacer = Node::new(
            "div",
            MessageRowPlaceholder {
                height: leading_height,
            },
        )
        .class(message_list::MESSAGE_LIST_SPACER);
        tree.mount(list_id, spacer);
    }

    let menu_env = message_menu_env(state, backend, queries, http, images, media_viewer);

    message_keys::set_env(menu_env.clone());

    let from_store = channel
        .guild_id
        .map(|guild_id| author_style::guild_roles(state, guild_id));
    let guild_roles: &[models::Role] = from_store.as_deref().unwrap_or(&[]);

    let role_mentions: Rc<HashMap<u64, (String, Option<Color>)>> =
        Rc::new(author_style::role_mention_map(guild_roles));

    let rows_started = Instant::now();
    let can_add_reactions = state.can(channel, Permissions::ADD_REACTIONS);
    let (must_build_from, must_build_to) = (
        viewport_top - virtualization::MUST_BUILD_MARGIN_PX,
        viewport_top + viewport_px + virtualization::MUST_BUILD_MARGIN_PX,
    );
    let row_env = RowEnv {
        state,
        channel,
        fonts,
        images,
        backend,
        colors,
        profile_popout,
        queries,
        attachments,
        media_viewer,
        measure_cache,
        images_ready,
        menu_env: &menu_env,
        guild_roles,
        mentions: &mentions,
        channels: &channels,
        role_mentions: &role_mentions,
        emoji_click: &emoji_click,
        messages: &messages,
        plugin_overlays: &plugin_overlays,
        show_original: &show_original,
        editing,
        replying,
        current_user_id,
        can_add_reactions,
        #[cfg(feature = "video")]
        video_ctx,
    };
    let RowsOutcome {
        popout_target,
        deferred_rows,
    } = mount_rows(
        tree,
        list_id,
        &row_env,
        &metas,
        visible_start..visible_end,
        (must_build_from, must_build_to),
        profile_popout_open,
    );
    REBUILD_PARTS_MS
        .with(|parts| parts.set((rows_started.elapsed().as_secs_f64() * 1000.0, parts.get().1)));
    schedule_follow_up(deferred_rows);

    if visible_end < metas.len() {
        let trailing_start = metas
            .get(visible_end)
            .map_or(total_estimated_height, |m| m.y - m.margin);
        let trailing_height = (total_estimated_height - trailing_start).max(0.0);
        let spacer = Node::new(
            "div",
            MessageRowPlaceholder {
                height: trailing_height,
            },
        )
        .class(message_list::MESSAGE_LIST_SPACER);
        tree.mount(list_id, spacer);
    }

    let pending_ctx = pending_uploads::PendingContext {
        user: state.users.current.get(),
        last: messages
            .last()
            .and_then(|m| m.timestamp_millis().map(|at| (m.author.id, at))),
        images: images.clone(),
        backend: backend.clone(),
    };
    tree.mount(
        list_id,
        pending_uploads::build_bottom(attachments, channel.id, pending_ctx, fonts),
    );

    // The typing strip overlays the list's bottom edge; leave it room so the last message stays clear.
    tree.mount(
        list_id,
        div()
            .class(message_list::MESSAGE_LIST_SPACER)
            .style(format!("height: {TYPING_CLEARANCE_PX}px")),
    );

    if state.messages.detached.get().contains(&channel.id) {
        mount_jump_to_present(tree, main_id, channel.id, colors);
    }

    if let Some(marker) = state
        .read_states
        .new_marker(channel.id)
        .filter(|marker| marker.count > 0)
    {
        mount_new_messages_bar(tree, main_id, state, channel.id, &marker);
    }

    tree.mount(
        main_id,
        reactions::picker_overlay(&menu_env, images_ready),
    );

    tree.mount(
        main_id,
        hover_tooltip::hover_tooltip_layer(),
    );

    let composer_started = Instant::now();
    build_composer(
        tree,
        main_id,
        state,
        &menu_env,
        channel,
        images,
        images_ready,
        compose_tx,
        composer_focused,
        composer_has_text,
        composer_send_toggle,
        colors,
        emoji_picker,
        attachments,
        message_scroll,
        media_viewer,
        fonts,
    );
    REBUILD_PARTS_MS.with(|parts| {
        parts.set((
            parts.get().0,
            composer_started.elapsed().as_secs_f64() * 1000.0,
        ))
    });

    if let Some((author_id, webhook_author)) = popout_target {
        mount_profile_popout(
            tree,
            main_id,
            state,
            channel,
            author_id,
            webhook_author.as_ref(),
            current_user_id,
            profile_popout,
            fonts,
            images,
            backend,
            queries,
            http,
        );
    }
}

fn plugin_note(
    state: &AppState,
    message_id: Id<marker::MessageMarker>,
    overlay: &plugins::IncomingOutcome,
    showing_original: bool,
    colors: &ChatColors,
) -> Option<Node> {
    let mut parts: Vec<Node> = overlay
        .annotations
        .iter()
        .map(|a| {
            TextProps::styled(a.value.clone(), TextVariant::TEXT_XS_NORMAL)
                .color(colors.text_muted)
                .build()
        })
        .collect();
    if let Some(rewrite) = &overlay.display_content {
        parts.push(
            TextProps::styled(
                requiem_i18n::t!(
                    "plugins-modified-by",
                    plugin = state.plugins.name_of(&rewrite.plugin)
                ),
                TextVariant::TEXT_XS_NORMAL,
            )
            .color(colors.text_muted)
            .build(),
        );
        let label = if showing_original {
            requiem_i18n::t!("plugins-show-modified")
        } else {
            requiem_i18n::t!("plugins-show-original")
        };
        let plugins = state.plugins.clone();
        parts.push(
            TextLinkProps::new(label, move || {
                plugins.toggle_original(message_id)
            })
            .small(true)
            .build(),
        );
    }
    (!parts.is_empty()).then(|| {
        div()
            .class(message_list::PLUGIN_NOTE)
            .children(parts)
    })
}

fn new_messages_text(
    count: u32,
    estimated: bool,
    since_ms: i64,
    now: chrono::DateTime<chrono::Local>,
) -> String {
    let Some(since) =
        chrono::DateTime::from_timestamp_millis(since_ms).map(|t| t.with_timezone(&chrono::Local))
    else {
        return requiem_i18n::t!("message-list-new-messages-since", count = count, time = "");
    };
    let time = utils_time::clock(&since);
    let date = utils_time::date_long(&since);
    match (estimated, since.date_naive() == now.date_naive()) {
        (false, true) => requiem_i18n::t!(
            "message-list-new-messages-since",
            count = count,
            time = time
        ),
        (false, false) => requiem_i18n::t!(
            "message-list-new-messages-since-on",
            count = count,
            time = time,
            date = date
        ),
        (true, true) => requiem_i18n::t!(
            "message-list-new-messages-estimated-since",
            count = count,
            time = time
        ),
        (true, false) => requiem_i18n::t!(
            "message-list-new-messages-estimated-since-on",
            count = count,
            time = time,
            date = date
        ),
    }
}

const DAY_DIVIDER_HEIGHT_PX: f32 = 25.0;

const UNREAD_LINE_INLINE_PX: f32 = 13.0;

fn new_pill() -> Node {
    let cap = IconSpec::Svg(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="13" viewBox="0 0 8 13"><path d="{}" stroke="currentColor" fill="currentColor"/></svg>"#,
        Icon::Named("shape/unread_pill_cap").path()
    ));
    div()
        .class(cls::UNREAD_PILL)
        .child(icon(cap, colors::BADGE_NOTIFICATION).class(cls::UNREAD_PILL_CAP))
        .child(
            TextProps::new(requiem_i18n::t!("message-list-new"), 10.0_f32)
                .line_height(9.0_f32)
                .weight(Weight::BOLD)
                .color(Color::WHITE)
                .uppercase(true)
                .build(),
        )
}

const JUMP_FLASH: Duration = Duration::from_secs(2);

const CHANNEL_HEADER_HEIGHT_PX: f32 = 48.0;

const COMPOSER_ROW_HEIGHT_PX: f32 = 66.0;

/// Tail space after the last row. The typing strip is 24px and overlays the list's bottom
/// edge; the list's own 16px bottom padding covers most of it.
const TYPING_CLEARANCE_PX: f32 = 8.0;

pub fn message_target(
    message: &models::Message,
    channel: &Channel,
) -> context_menus::MessageTarget {
    context_menus::MessageTarget {
        id: message.id,
        channel_id: message.channel_id,
        guild_id: channel.guild_id,
        author_id: message.author.id,
        author_name: message.author.display_name(None),
        content: message.content.clone(),
        message: message.clone(),
        my_reactions: message
            .reactions
            .iter()
            .filter(|r| r.me)
            .map(|r| r.emoji.clone())
            .collect(),
    }
}

fn reply_preview(
    state: &AppState,
    channel: &Channel,
    roles: &[models::Role],
    message: &models::Message,
    images: &ImageCache,
    backend: &BackendConfig,
    fonts: &Fonts,
    profile_popout: &ProfilePopoutState,
    mentions: &Rc<HashMap<u64, String>>,
    channels: &Rc<HashMap<u64, String>>,
) -> Node {
    let target = message.referenced_message.as_deref();
    let (avatar, name, style) = match target {
        Some(target) => {
            let author = state.users.resolve(&target.author);
            let style = author_style::resolve(state, roles, channel.guild_id, &author);
            let image = author
                .avatar
                .as_deref()
                .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(author.id, hash)));
            let mut props = ui::user_avatar_props(author.id, &author.discriminator, fonts);
            if let Some(image) = image {
                props = props.image(image);
            }
            // The reply pings its author only when they are among its mentions.
            let pinged = message.mentions.iter().any(|user| user.id == author.id);
            let shown = author_display_name(state, channel, &author);
            (
                Some(props.build()),
                Some(if pinged { format!("@{shown}") } else { shown }),
                style,
            )
        }
        None => (None, None, Default::default()),
    };
    let jump = message
        .message_reference
        .as_ref()
        .and_then(|r| r.message_id)
        .filter(|_| target.is_some())
        .map(|id| {
            let channel_id = channel.id;
            Rc::new(move || request_jump(channel_id, id)) as Rc<dyn Fn()>
        });

    let target_author = target.map(|t| t.author.id);
    let open_profile = |node: Node, part: AuthorPart| match target_author {
        Some(user_id) => {
            let class = match part {
                AuthorPart::Avatar => reply_cls::REPLY_AVATAR_BTN,
                AuthorPart::Name => reply_cls::REPLY_AUTHOR_BTN,
            };
            profile_popout::clickable_user(node, message.id, user_id, profile_popout, Some(class))
        }
        None => node,
    };

    let mut names = (**mentions).clone();
    for user in target.into_iter().flat_map(|t| t.mentions.iter()) {
        names
            .entry(user.id.get())
            .or_insert_with(|| state.users.display_name(user));
    }
    let tag = style.tag.as_ref().map(|tag| {
        author_style::tag_chip(
            tag,
            TagPlace::Reply,
            images,
            backend,
        )
    });
    reply::preview(
        avatar,
        name.as_deref().map(|name| (name, style.fill.as_ref())),
        tag,
        Snippet::of(target),
        jump,
        emoji_text::resolver(images, backend),
        Rc::new(names),
        channels.clone(),
        Rc::new(author_style::role_mention_map(roles)),
        &open_profile,
    )
}

fn ephemeral_footer(state: &AppState, message: &models::Message) -> Node {
    let (state, id, channel_id) = (state.clone(), message.id, message.channel_id);
    let dismiss = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .on_click(move || {
            state.apply_event(&GatewayEvent::new(
                "MESSAGE_DELETE",
                serde_json::json!({ "id": id.to_string(), "channel_id": channel_id.to_string() }),
            ))
        })
        .build()
        .child(
            TextProps::styled(
                requiem_i18n::t!("reply-dismiss").to_string(),
                TextVariant::TEXT_XS_NORMAL,
            )
            .color(colors::TEXT_ANCHOR)
            .build(),
        );
    div().class(reply_cls::EPHEMERAL_NOTE).children([
        TextProps::styled(
            format!("{} \u{b7}", requiem_i18n::t!("reply-only-you")),
            TextVariant::TEXT_XS_NORMAL,
        )
        .color(TEXT_SUBTLE)
        .build(),
        dismiss,
    ])
}

fn message_menu_handler(
    env: &context_menus::MenuEnv,
    message: &models::Message,
    channel: &Channel,
) -> impl Fn(f32, f32) + 'static {
    let (env, target) = (env.clone(), Rc::new(message_target(message, channel)));
    move |x, y| {
        let (env, target) = (env.clone(), target.clone());
        context_menu::open(x, y, move || {
            context_menus::message_items(&env, &target)
        });
    }
}

pub struct PreviewMessage<'a> {
    pub avatar: Option<skia_safe::Image>,
    pub name: Node,
    pub role_icon: Option<skia_safe::Image>,
    pub time: String,
    pub timestamp_color: Color,
    pub text: &'a str,
    pub text_color: Color,
}

pub fn preview_message(fonts: &Fonts, message: PreviewMessage<'_>) -> Node {
    let mut avatar = ui::user_avatar_props(Id::new(1), "0", fonts);
    if let Some(image) = message.avatar {
        avatar = avatar.image(image);
    }
    let avatar = div()
        .class(cls::MESSAGE_AVATAR_BTN)
        .child(avatar.build().class(cls::MESSAGE_AVATAR));
    let mut header = div()
        .class(cls::MESSAGE_HEADER)
        .class(cls::MESSAGE_HEADER_PREVIEW)
        .child(message.name.class(cls::MESSAGE_NOWRAP));
    if let Some(image) = message.role_icon {
        header = header.child(
            Node::new(
                "div",
                ImageWidget::new(Some(image)),
            )
            .class(cls::MESSAGE_ROLE_ICON),
        );
    }
    header = header.child(
        TextProps::styled(message.time, TextVariant::TEXT_XS_MEDIUM)
            .line_height(MESSAGE_LINE_HEIGHT)
            .color(message.timestamp_color)
            .build()
            .class(cls::MESSAGE_NOWRAP),
    );
    let text = TextProps::styled(message.text.to_string(), TextVariant::TEXT_MD_NORMAL)
        .line_height(MESSAGE_LINE_HEIGHT)
        .color(message.text_color)
        .build()
        .class(cls::MESSAGE_NOWRAP);
    let body = div().class(cls::MESSAGE_BODY).children([header, text]);
    div()
        .class(cls::MESSAGE_ROW)
        .class(cls::MESSAGE_ROW_STATIC)
        .children([avatar, body])
}

pub fn author_display_name(
    state: &AppState,
    channel: &Channel,
    author: &models::User,
) -> String {
    author.display_name(
        channel
            .guild_id
            .and_then(|guild_id| state.members.nick_of(guild_id, author.id))
            .as_deref(),
    )
}

fn user_menu_handler(
    env: &context_menus::MenuEnv,
    message: &models::Message,
    channel: &Channel,
    display_name: String,
) -> impl Fn(f32, f32) + 'static {
    let env = env.clone();
    let target = Rc::new(context_menus::UserTarget {
        user_id: message.author.id,
        guild_id: channel.guild_id,
        display_name,
        webhook: message.webhook_id.is_some(),
    });
    move |x, y| {
        if target.webhook && !prefs::developer_mode() {
            return;
        }
        let (env, target) = (env.clone(), target.clone());
        context_menu::open(x, y, move || {
            context_menus::user_items(&env, &target)
        });
    }
}

fn attachment_menu_handler(
    env: &context_menus::MenuEnv,
    message: &models::Message,
    channel: &Channel,
) -> message_embed::AttachmentMenu {
    let (env, target) = (env.clone(), Rc::new(message_target(message, channel)));
    Rc::new(
        move |attachment: &models::Attachment, x: f32, y: f32| {
            let (env, target, attachment) = (env.clone(), target.clone(), attachment.clone());
            context_menu::open(x, y, move || {
                context_menus::attachment_items(&env, &attachment, &target)
            });
        },
    )
}

fn mention_names(state: &AppState, channel: &Channel) -> HashMap<u64, String> {
    let mut names = HashMap::new();
    if let Some(guild_id) = channel.guild_id {
        state.members.by_guild.with_untracked(|by_guild| {
            if let Some(members) = by_guild.get(&guild_id) {
                for member in members.values() {
                    let display = member.user.display_name(member.nick.as_deref());
                    names.insert(member.user.id.get(), display);
                }
            }
        });
    }
    state.messages.by_channel.with_untracked(|by_channel| {
        for message in by_channel.get(&channel.id).into_iter().flatten() {
            names
                .entry(message.author.id.get())
                .or_insert_with(|| state.users.display_name(&message.author));

            for user in &message.mentions {
                names
                    .entry(user.id.get())
                    .or_insert_with(|| state.users.display_name(user));
            }
        }
    });
    names
}

fn channel_names(state: &AppState, channel: &Channel) -> HashMap<u64, String> {
    let Some(guild_id) = channel.guild_id else {
        return HashMap::new();
    };
    state.channels.by_guild.watch(&guild_id);
    state.channels.by_guild.with_untracked(|by_guild| {
        by_guild
            .get(&guild_id)
            .into_iter()
            .flatten()
            .filter(|c| state.can(c, Permissions::VIEW_CHANNEL))
            .filter_map(|c| Some((c.id.get(), c.name.clone()?)))
            .collect()
    })
}

fn mentions_current_user(message: &models::Message, current_user_id: Option<u64>) -> bool {
    let Some(current_user_id) = current_user_id else {
        return false;
    };
    message.mention_everyone
        || message
            .mentions
            .iter()
            .any(|u| u.id.get() == current_user_id)
}

pub fn chat_input(props: TextInputProps) -> TextInputProps {
    chat_input_with(props, None)
}

pub fn chat_input_with(
    props: TextInputProps,
    custom: Option<&emoji_picker::CustomEmojiSource>,
) -> TextInputProps {
    let props = props
        .markdown(MarkdownFeatures::ALL)
        .code_highlighter(code_block::composer_highlighter())
        .emoji_provider(twemoji::tile as primitives::EmojiProvider)
        .dynamic_autocomplete(vec![emoji_picker::emoji_lookup(
            custom.map(|c| c.lookup_sections()),
        )]);
    match custom {
        Some(source) => props.custom_emoji(emoji_text::resolver(
            &source.images,
            &source.backend,
        )),
        None => props,
    }
}

fn custom_emoji_source(
    env: &context_menus::MenuEnv,
    channel: &Channel,
    tick: &Signal<u64>,
) -> emoji_picker::CustomEmojiSource {
    let nitro = env.state.users.current.with_untracked(|u| {
        u.as_ref()
            .and_then(|u| u.premium_type)
            .is_some_and(|t| t > 0)
    });
    emoji_picker::CustomEmojiSource::new(env, channel.guild_id, nitro, tick)
}

pub fn member_candidates(
    state: &AppState,
    channel: &Channel,
    env: &context_menus::MenuEnv,
) -> Vec<AutocompleteCandidate> {
    let members_by_guild = state.members.by_guild.peek();
    let channel_messages = state.messages.in_channel(channel.id);

    let roster = channel
        .guild_id
        .and_then(|guild_id| members_by_guild.get(&guild_id))
        .filter(|members| !members.is_empty());

    let avatar_of = |user: &models::User| {
        let url = user
            .avatar
            .as_deref()
            .map(|hash| env.backend.avatar_url_sized(user.id, hash, 48))?;
        let images = env.images.clone();
        Some(Rc::new(move || images.get_or_decode_image(&url))
            as Rc<dyn Fn() -> Option<skia_safe::Image>>)
    };
    let mentions: Vec<AutocompleteCandidate> = match roster {
        Some(members) => members
            .values()
            .map(|member| {
                let display = member.user.display_name(member.nick.as_deref());
                let subtitle =
                    (display != member.user.username).then(|| member.user.username.clone());
                AutocompleteCandidate {
                    display,
                    id: Some(member.user.id.get()),
                    subtitle,
                    avatar_color: Some(ui::default_avatar_color(
                        member.user.id,
                        &member.user.discriminator,
                    )),
                    avatar: avatar_of(&member.user),
                    fallback_icon: Some(ui::DEFAULT_AVATAR_ICON),
                    status: channel.guild_id.map(|guild_id| {
                        state
                            .members
                            .presence_of(guild_id, member.user.id)
                            .unwrap_or_default()
                    }),
                    ..Default::default()
                }
            })
            .collect(),
        None => {
            let mut seen_authors = HashSet::new();
            channel_messages
                .iter()
                .filter(|message| seen_authors.insert(message.author.id))
                .map(|message| {
                    let author = state.users.resolve(&message.author);
                    let subtitle = author
                        .global_name
                        .is_some()
                        .then(|| author.username.clone());
                    let display = author
                        .global_name
                        .clone()
                        .unwrap_or_else(|| author.username.clone());
                    AutocompleteCandidate {
                        display,
                        id: Some(message.author.id.get()),
                        subtitle,
                        avatar_color: Some(ui::default_avatar_color(
                            author.id,
                            &author.discriminator,
                        )),
                        avatar: avatar_of(&author),
                        fallback_icon: Some(ui::DEFAULT_AVATAR_ICON),
                        ..Default::default()
                    }
                })
                .collect()
        }
    };
    mentions
}

pub fn autocomplete_triggers(
    state: &AppState,
    channel: &Channel,
    colors: &ChatColors,
    env: &context_menus::MenuEnv,
) -> Vec<AutocompleteTrigger> {
    if let Some(guild_id) = channel.guild_id {
        state.channels.by_guild.watch(&guild_id);
    }
    let channels_by_guild: HashMap<_, _> = channel
        .guild_id
        .and_then(|id| {
            state
                .channels
                .by_guild
                .with_untracked(|all| all.get(&id).cloned().map(|channels| (id, channels)))
        })
        .into_iter()
        .collect();
    let mut mentions = member_candidates(state, channel, env);

    let channels: Vec<AutocompleteCandidate> = channel
        .guild_id
        .and_then(|guild_id| channels_by_guild.get(&guild_id))
        .into_iter()
        .flatten()
        .filter(|c| state.can(c, Permissions::VIEW_CHANNEL))
        .filter_map(|c| Some((c.id, c.name.clone()?)))
        .map(|(id, name)| AutocompleteCandidate {
            display: name,
            id: Some(id.get()),
            ..Default::default()
        })
        .collect();

    {
        let (time_env, channel_id) = (env.clone(), channel.id);
        mentions.push(AutocompleteCandidate {
            display: "time".to_string(),
            subtitle: Some(requiem_i18n::t!("time-panel-mention-description").to_string()),
            section: 1,
            on_select: Some(Rc::new(move || {
                time_panel::begin(&time_env, channel_id)
            })),
            ..Default::default()
        });
    }
    let may_ping_everyone = state.can(channel, Permissions::MENTION_EVERYONE);
    if may_ping_everyone && channel.guild_id.is_some() {
        for (name, description) in [
            (
                "everyone",
                requiem_i18n::t!("message-list-mention-everyone"),
            ),
            ("here", requiem_i18n::t!("message-list-mention-here")),
        ] {
            mentions.push(AutocompleteCandidate {
                display: name.to_string(),
                subtitle: Some(description.to_string()),
                section: 1,
                ..Default::default()
            });
        }
    }
    if let Some(guild_id) = channel.guild_id {
        let stored = author_style::guild_roles(&env.state, guild_id);
        let mut roles: Vec<&models::Role> = stored
            .iter()
            .filter(|role| {
                role.id.get() != guild_id.get() && (role.mentionable || may_ping_everyone)
            })
            .collect();
        roles.sort_by_key(|role| cmp::Reverse(role.position));
        for role in roles {
            let fill = author_style::role_fill(role);
            let color = match fill {
                Some(NameFill::Solid(color)) => Some(color),
                Some(NameFill::Gradient(colors)) => {
                    colors.first().copied()
                }
                None => None,
            };
            mentions.push(AutocompleteCandidate {
                display: role.name.clone(),
                subtitle: Some(requiem_i18n::t!("message-list-mention-role").to_string()),
                section: 2,
                name_color: color,
                token: Some(format!("<@&{}>", role.id)),
                ..Default::default()
            });
        }
    }

    vec![
        AutocompleteTrigger {
            trigger_char: primitives::TIMESTAMP_TRIGGER_CHAR,
            candidates: Vec::new(),
            only_at_message_start: false,
            chip_background: Some(colors::BACKGROUND_MOD_NORMAL),
            chip_text_color: colors.text_default,
            popup_label: String::new(),
            multi_word: false,
            whole_text: false,
        },
        AutocompleteTrigger {
            trigger_char: '@',
            candidates: mentions,
            only_at_message_start: false,
            chip_background: Some(colors.mention_chip_background),
            chip_text_color: colors.mention_chip_text,
            popup_label: requiem_i18n::t!("message-list-members").to_string(),
            multi_word: false,
            whole_text: false,
        },
        AutocompleteTrigger {
            trigger_char: '#',
            candidates: channels,
            only_at_message_start: false,
            chip_background: Some(colors.mention_chip_background),
            chip_text_color: colors.mention_chip_text,
            popup_label: requiem_i18n::t!("message-list-channels").to_string(),
            multi_word: false,
            whole_text: false,
        },
    ]
}

#[allow(clippy::too_many_arguments)]
fn build_composer(
    tree: &mut WidgetTree,
    parent: WidgetId,
    state: &AppState,
    menu_env: &context_menus::MenuEnv,
    channel: &Channel,
    images: &ImageCache,
    images_ready: &Signal<u64>,
    compose_tx: &UnboundedSender<(Id<ChannelMarker>, String)>,
    composer_focused: &Signal<bool>,
    composer_has_text: &Signal<bool>,
    composer_send_toggle: &Signal<ToggleState>,
    colors: &ChatColors,
    emoji_picker: &EmojiPickerState,
    attachments: &AttachmentsState,
    message_scroll: &Signal<HashMap<Id<ChannelMarker>, MessageListScroll>>,
    media_viewer: &media_viewer::MediaViewerState,
    fonts: &Fonts,
) {
    let row_class = message_list::COMPOSER_ROW;
    let box_class = message_list::COMPOSER_BOX;
    let box_focused_class = message_list::COMPOSER_BOX_FOCUSED;
    let box_unfocused_class = message_list::COMPOSER_BOX_UNFOCUSED;
    let attach_btn_class = message_list::COMPOSER_ATTACH_BTN;
    let attach_icon_class = message_list::COMPOSER_ATTACH_ICON;
    let composer_class = message_list::COMPOSER;
    let popup_outset = (48.0, 8.0);

    let channel_id = channel.id;

    let _ = command_composer::reseed_tick();

    let permissions = state.channel_permissions(channel);
    if !permissions.contains(Permissions::SEND_MESSAGES) {
        let reason = TextProps::styled(
            requiem_i18n::t!("message-list-you-do-not-have-permission-to").to_string(),
            TextVariant::TEXT_MD_NORMAL,
        )
        .line_height(22.0_f32)
        .color(colors.text_placeholder)
        .line_clamp(1_u32)
        .build();
        let pill = div()
            .class(box_class)
            .class(box_unfocused_class)
            .class(message_list::COMPOSER_BOX_DISABLED)
            .child(reason);
        tree.mount(
            parent,
            div()
                .class(row_class)
                .child(typing_bar::fade())
                .child(pill),
        );
        return;
    }
    let can_attach = permissions.contains(Permissions::ATTACH_FILES);

    let initial_text = state.drafts.text(channel_id);
    let initial_focused = composer_focused.peek();

    {
        let has_text = !initial_text.trim().is_empty();
        let (has_text_flag, send_toggle) =
            (composer_has_text.clone(), composer_send_toggle.clone());
        timer::set_timeout(Duration::ZERO, move || {
            if has_text_flag.peek() != has_text {
                has_text_flag.set(has_text);
                send_toggle.set(send_toggle.peek().set(has_text));
            }
        });
    }

    let change_draft = state.drafts.clone();

    let first_change = Cell::new(true);
    let focus_draft = composer_focused.clone();
    let submit_draft = state.drafts.clone();
    let submit_tx = compose_tx.clone();
    let submit_attachments = attachments.clone();
    let (submit_env, submit_channel) = (menu_env.clone(), channel.clone());
    let submit_scroll = message_scroll.clone();
    let has_text_draft = composer_has_text.clone();
    let picker_open_for_panel = emoji_picker.open.clone();
    let send_toggle_draft = composer_send_toggle.clone();

    let menu_open = attachments.menu_open.clone();
    let attach_props = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![attach_btn_class.to_string()])
        .on_click(move || menu_open.update(|open| *open = !*open));
    let attach_btn = attach_props
        .build()
        .dom_id(attachments::MENU_BUTTON_DOM_ID)
        .child(icon(PLUS_LARGE, ui::TEXT_SECONDARY).class(attach_icon_class));

    let placeholder = match (state.dms.find(channel.id), channel.name.as_deref()) {
        (Some(dm), _) if dm.is_group() => requiem_i18n::t!(
            "message-list-placeholder-group",
            name = home::dm_title(state, &dm)
        ),
        (Some(dm), _) => requiem_i18n::t!(
            "message-list-placeholder-dm",
            name = home::dm_title(state, &dm)
        ),
        (None, Some(name)) => requiem_i18n::t!("message-list-placeholder-channel", name = name),
        (None, None) => requiem_i18n::t!("message-list-message-2").to_string(),
    };

    let custom_source = custom_emoji_source(menu_env, channel, images_ready);
    let input: Node = chat_input_with(TextInputProps::default(), Some(&custom_source))
        .text(initial_text)
        .placeholder(placeholder)
        .font_size(16.0_f32)
        .line_height(22.0_f32)
        .vertical_padding(17.0_f32)
        .color(colors.text_default)
        .placeholder_color(colors.text_placeholder)
        .focused(initial_focused)
        .wrap(true)
        .key_hook({
            let (hook_env, hook_channel) = (menu_env.clone(), channel.clone());
            move |key: NamedKey, shift: bool, text: &str| match key {
                NamedKey::Undo
                    if command_composer::plain_undo(
                        &hook_env,
                        hook_channel.id,
                        text,
                    ) =>
                {
                    true
                }
                NamedKey::Redo
                    if command_composer::plain_redo(
                        &hook_env,
                        hook_channel.id,
                        text,
                    ) =>
                {
                    true
                }
                _ => commands_panel::handle_key(&hook_env, &hook_channel, key, shift),
            }
        })
        .on_change(move |text: &str| {
            commands_panel::set_text(channel_id, text);

            if text.starts_with('/') && picker_open_for_panel.peek() {
                picker_open_for_panel.set(false);
            }

            typing_local::composer_changed(
                channel_id.get(),
                &change_draft.text(channel_id),
                text,
                first_change.replace(false),
            );

            change_draft.set(channel_id, text);

            let has_text = !text.trim().is_empty();
            if has_text_draft.peek() != has_text {
                has_text_draft.set(has_text);
                let current_toggle = send_toggle_draft.peek();
                send_toggle_draft.set(current_toggle.set(has_text));
            }
        })
        .on_focus_change(move |focused| {
            focus_draft.set(focused);
        })
        .on_submit(move |content: String| {
            let content = emoji_skin::strip_tones(&content).into_owned();

            submit_draft.clear(channel_id);
            typing_local::stopped(channel_id.get());

            if slash_commands::try_send(&submit_env, &submit_channel, &content) {
                return;
            }

            reply::take_for_send(channel_id);

            match submit_attachments.begin_send(channel_id, content.clone()) {
                Some((nonce, files)) => {
                    (submit_attachments.backend.send)(channel_id, nonce, content, files);
                    jump_to_present(&submit_scroll, channel_id);
                }
                None => {
                    let _ = submit_tx.send((channel_id, content));
                }
            }
        })
        .paste_override({
            let attachments = attachments.clone();
            Rc::new(move || {
                can_attach && attachments::paste_from_clipboard(&attachments, channel_id)
            }) as Rc<dyn Fn() -> bool>
        })
        .submit_when_empty({
            let attachments = attachments.clone();
            Rc::new(move || attachments.count(channel_id) > 0) as Rc<dyn Fn() -> bool>
        })
        .autocomplete(autocomplete_triggers(state, channel, colors, menu_env))
        .popup_outset(popup_outset)
        .build()
        .dom_id("composer-input")
        .bind_image(images_ready.clone(), || None)
        .on_context_menu(|x, y| {
            context_menu::open(x, y, context_menus::composer_items)
        })
        .class(composer_class);

    let input = if initial_focused {
        input.wants_focus()
    } else {
        input
    };

    let box_focus_class = if initial_focused {
        box_focused_class
    } else {
        box_unfocused_class
    };

    let attach_btn = div()
        .class(message_list::COMPOSER_ATTACH_ANCHOR)
        .child(attach_btn)
        .child(attach_menu(attachments, state, channel_id));

    let time_ctx = time_panel::Ctx {
        env: menu_env.clone(),
        channel: channel.clone(),
        composer_focused: composer_focused.clone(),
    };
    let time_input = time_panel::active(channel_id);
    let input = match &time_input {
        Some(pill) => time_panel::content(&time_ctx, pill),
        None => input,
    };
    let mut composer_box = if can_attach {
        div().child(attach_btn).child(input)
    } else {
        div().child(input)
    };
    composer_box = composer_box.class(message_list::COMPOSER_LINE);
    if !can_attach {
        composer_box =
            composer_box.class(message_list::COMPOSER_LINE_SANS_ATTACH);
    }

    let command_frame: Signal<AbsoluteRect> = signal::signal(AbsoluteRect::default());
    let mut command_bar: Option<Node> = None;
    if let Some(active) = command_composer::active(channel_id) {
        let ctx = command_composer::Ctx {
            env: menu_env,
            channel,
            fonts,
            colors,
            custom_emoji: &custom_source,
            frame: command_frame.clone(),
        };
        command_bar = Some(command_composer::bar(&ctx, &active));
        composer_box = command_composer::line(&ctx, &active);
    }
    let in_command = command_bar.is_some();
    if time_input.is_some() && !in_command {
        command_bar = Some(time_panel::bar());
    }

    let has_picker;
    {
        let open_for_gif_click = emoji_picker.open.clone();
        let tab_for_gif_click = emoji_picker.tab.clone();
        let signals_for_gif_click = emoji_picker.signals.clone();
        let on_open_gif_tab = emoji_picker.gif.on_open.clone();
        let gif_picker_btn = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![
                message_list::COMPOSER_GIF_BTN.to_string(),
            ])
            .on_click(move || {
                tab_for_gif_click.set(emoji_picker::PickerTab::Gif);
                if !open_for_gif_click.peek() {
                    signals_for_gif_click.reset_for_open();
                }
                open_for_gif_click.set(true);
                on_open_gif_tab();
            })
            .build()
            .child(
                Node::new("span", GifButtonIconWidget::new())
                    .class(message_list::COMPOSER_GIF_BTN_ICON),
            );
        if !in_command {
            composer_box = composer_box.child(gif_picker_btn);
        }

        let open_for_click = emoji_picker.open.clone();
        let tab_for_click = emoji_picker.tab.clone();
        let signals_for_click = emoji_picker.signals.clone();
        let emoji_picker_btn = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![
                message_list::COMPOSER_EMOJI_BTN.to_string(),
            ])
            .on_click(move || {
                tab_for_click.set(emoji_picker::PickerTab::Emoji);
                if !open_for_click.peek() {
                    signals_for_click.reset_for_open();
                }
                open_for_click.update(|open| *open = !*open);
            })
            .build()
            .child(
                Node::new(
                    "span",
                    EmojiButtonIconWidget::new(),
                )
                .class(message_list::COMPOSER_EMOJI_BTN_ICON),
            );
        composer_box = composer_box.child(emoji_picker_btn);
        has_picker = true;
    }

    let pill = div()
        .class(box_class)
        .class(box_focus_class)
        .bind_class(
            composer_focused.clone(),
            box_focused_class,
            box_unfocused_class,
        )
        .observe_rect(command_frame.clone())
        .children(command_bar)
        .child(reply::bar(
            channel_id,
            state
                .users
                .current
                .with_untracked(|u| u.as_ref().map(|u| u.id)),
        ))
        .child(attachments::build_tray(
            attachments,
            channel_id,
            Some(attachments::TrayViewer {
                viewer: media_viewer.clone(),
                images: images.clone(),
            }),
        ))
        .child(composer_box);
    // The fade is first so the composer box paints over it, as on Discord.
    let mut row = div()
        .class(row_class)
        .child(typing_bar::fade())
        .child(pill);
    if !in_command {
        // Top-layer like the emoji picker and attach menu, so the message list can't paint over it.
        row = row.child(
            Node::new("div", primitives::TopLayerWidget)
                .class(classes_commands_panel::CMD_PANEL_HOLDER)
                .child(commands_panel::panel(
                    menu_env,
                    channel,
                    fonts,
                    command_frame.clone(),
                )),
        );
        row = row.child(
            Node::new("div", primitives::TopLayerWidget)
                .class(classes_commands_panel::CMD_PANEL_HOLDER)
                .child(time_panel::panel(
                    time_ctx.clone(),
                    command_frame.clone(),
                )),
        );
    }

    if has_picker {
        let on_insert = emoji_picker.on_insert.clone();
        let open_after_emoji_pick = emoji_picker.open.clone();
        let on_pick_emoji: Rc<dyn Fn(String, bool)> = Rc::new(move |text: String, shift: bool| {
            on_insert(format!("{text} "));
            if !shift {
                open_after_emoji_pick.set(false);
            }
        });

        let send_tx = compose_tx.clone();
        let open_after_gif_pick = emoji_picker.open.clone();
        let on_pick_gif: Rc<dyn Fn(String)> = Rc::new(move |gif_url: String| {
            let _ = send_tx.send((channel_id, gif_url));
            open_after_gif_pick.set(false);
        });

        let picker_state = emoji_picker.clone();
        let images = images.clone();
        let (picker_env, picker_guild, picker_tick) =
            (menu_env.clone(), channel.guild_id, images_ready.clone());
        let picker = Node::reactive(move || {
            if !picker_state.open.get() {
                return div();
            }

            let nitro = picker_env.state.users.current.with_untracked(|u| {
                u.as_ref()
                    .and_then(|u| u.premium_type)
                    .is_some_and(|t| t > 0)
            });
            let custom = emoji_picker::CustomEmojiSource::new(
                &picker_env,
                picker_guild,
                nitro,
                &picker_tick,
            );
            emoji_picker::build(
                &picker_state,
                &images,
                on_pick_emoji.clone(),
                on_pick_gif.clone(),
                Some(custom),
            )
            .style("right: 0px; bottom: 0px")
        });
        row = row.child(
            Node::new("div", primitives::TopLayerWidget)
                .class(message_list::COMPOSER_EMOJI_PICKER)
                .child(picker),
        );

        let backdrop_open = emoji_picker.open.clone();
        let backdrop_state = emoji_picker.open.clone();
        row = row.child(Node::reactive(move || {
            if !backdrop_open.get() {
                return div();
            }
            let close = backdrop_state.clone();
            components::anchored_popover_backdrop(move || close.set(false))
        }));
    }

    tree.mount(parent, typing_bar::build(state, channel_id));
    tree.mount(parent, row);
    if let Some(banner) = threads::thread_banner(channel) {
        tree.mount(parent, banner);
    }
}

pub fn attach_menu(
    attachments: &AttachmentsState,
    app_state: &AppState,
    channel_id: Id<ChannelMarker>,
) -> Node {
    let (state, app_state) = (attachments.clone(), app_state.clone());
    let menu = Node::reactive(move || {
        if !state.menu_open.get() {
            return div();
        }
        let entry =
            |icon_path: Icon, label: &str, enabled: bool, on_click: Option<Box<dyn FnMut()>>| {
                let mut props = ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![cls::ATTACH_MENU_ITEM.to_string()]);
                if let Some(on_click) = on_click {
                    props = props.on_click(on_click);
                }
                let mut item = props
                    .build()
                    .child(icon(icon_path, ui::TEXT_SECONDARY).class(cls::ATTACH_MENU_ICON))
                    .child(
                        TextProps::styled(label.to_string(), TextVariant::TEXT_SM_MEDIUM)
                            .color(TEXT_STRONG)
                            .build(),
                    );
                if !enabled {
                    item = item.class(cls::ATTACH_MENU_ITEM_DISABLED);
                }
                item
            };
        let upload = {
            let state = state.clone();
            Box::new(move || {
                state.menu_open.set(false);
                (state.backend.pick_files)(channel_id);
            }) as Box<dyn FnMut()>
        };
        let app_state = app_state.clone();
        let create_thread = {
            let state = state.clone();
            Box::new(move || {
                state.menu_open.set(false);
                if let Some(guild_id) = ui::find_channel(&app_state, channel_id)
                    .and_then(|(_, channel)| channel.guild_id)
                {
                    threads::open_create(guild_id, channel_id, None);
                }
            }) as Box<dyn FnMut()>
        };
        let menu = div()
            .dom_id(attachments::MENU_DOM_ID)
            .class(cls::ATTACH_MENU)
            .child(entry(
                FILE_UP,
                &requiem_i18n::t!("message-list-upload-a-file"),
                true,
                Some(upload),
            ))
            .child(entry(
                THREAD_PLUS,
                &requiem_i18n::t!("message-list-create-thread"),
                true,
                Some(create_thread),
            ))
            .child(entry(
                POLLS,
                &requiem_i18n::t!("message-list-create-poll"),
                false,
                None,
            ))
            .child(entry(
                APPS,
                &requiem_i18n::t!("message-list-use-apps"),
                false,
                None,
            ));

        let close_state = state.clone();
        div()
            .child(components::anchored_popover_backdrop(
                move || close_state.menu_open.set(false),
            ))
            .child(menu)
    });

    Node::new("div", primitives::TopLayerWidget)
        .class(cls::COMPOSER_ATTACH_MENU)
        .child(menu)
}

#[cfg(test)]
mod new_messages_bar_tests {
    use super::*;
    use chrono::TimeZone;

    fn local(day: u32, hour: u32, minute: u32) -> chrono::DateTime<chrono::Local> {
        chrono::Local
            .with_ymd_and_hms(2026, 10, day, hour, minute, 0)
            .unwrap()
    }

    #[test]
    fn the_bar_counts_the_messages_and_says_since_when_with_the_date_only_when_it_was_not_today() {
        let since = local(8, 9, 48).timestamp_millis();
        let today = local(8, 15, 0);
        let text = |count, estimated, now| new_messages_text(count, estimated, since, now);

        assert!(
            text(4, false, today).starts_with("4 new messages since "),
            "{}",
            text(4, false, today)
        );
        assert!(
            text(1, false, today).starts_with("1 new message since "),
            "singular"
        );
        assert!(!text(4, false, today).contains(" on "), "same day: no date");
        assert!(
            text(4, false, local(9, 8, 0)).ends_with("on October 8, 2026"),
            "{}",
            text(4, false, local(9, 8, 0))
        );
        assert!(
            text(50, true, today).starts_with("50+ new messages since "),
            "more may be out of reach"
        );
        assert!(text(50, true, local(9, 8, 0)).contains(" on October 8, 2026"));
    }
}
