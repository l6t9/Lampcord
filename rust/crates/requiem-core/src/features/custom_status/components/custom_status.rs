use crate::models;
use crate::ui::classes::custom_status as cls;
use crate::ui::emoji_button_icon::EmojiButtonIconWidget;
use crate::ui::emoji_button_icon;
use crate::ui::profile_popout;
use crate::ui::twemoji;
use crate::ui;
use crate::utils::time;
use requiem_ui::colors::TEXT_MUTED;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Align, Padding, StackProps};
use requiem_ui::components::{AvatarStatusSize, avatar_with_status};
use requiem_ui::components::{ButtonVariant, ModalSize};
use requiem_ui::icons::IconPath;
use requiem_ui::widget::primitives::MessageTextWidget;
use requiem_ui::widget::primitives::PresenceStatus;
use requiem_ui::widget::primitives;
use skia_safe::surfaces;
use std::rc::Rc;
use std::sync::OnceLock;

use chrono::{DateTime, Duration as ChronoDuration, Local, TimeZone};
use requiem_ui::components::{
    ButtonProps, MessageTextProps, ModalProps, SelectOption, SelectProps, TextInputProps,
    TextProps, TooltipProps, div, icon, message_text,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::primitives::{ImageWidget, MarkdownFeatures, TopLayerWidget};
use skia_safe::font_style::Weight;
use skia_safe::{Color, Image};

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::presence::{CustomStatus, StatusEmoji};
use crate::ui::emoji_picker::{self, EmojiPickerSignals};
use crate::ui::{Fonts, TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::icons::{CIRCLE_PLUS, PENCIL, TRASH};

pub const PROMPTS: [&str; 32] = [
    requiem_i18n::key!("custom-status-can-t-wait-for"),
    requiem_i18n::key!("custom-status-choose-your-character-class"),
    requiem_i18n::key!("custom-status-what-are-you-currently-obsessed-with"),
    requiem_i18n::key!("custom-status-best-dad-joke"),
    requiem_i18n::key!("custom-status-best-emoji-to-describe-your-day"),
    requiem_i18n::key!("custom-status-favorite-anime-lately"),
    requiem_i18n::key!("custom-status-favorite-collectible"),
    requiem_i18n::key!("custom-status-what-fictional-world-would-you-live"),
    requiem_i18n::key!("custom-status-just-finished-playing"),
    requiem_i18n::key!("custom-status-just-finished-reading"),
    requiem_i18n::key!("custom-status-food-you-re-craving-now"),
    requiem_i18n::key!("custom-status-most-satisfying-game-mechanic"),
    requiem_i18n::key!("custom-status-hit-a-high-score-in"),
    requiem_i18n::key!("custom-status-what-s-your-hot-take"),
    requiem_i18n::key!("custom-status-latest-hobby"),
    requiem_i18n::key!("custom-status-what-have-you-been-listening-to"),
    requiem_i18n::key!("custom-status-what-are-you-looking-forward-to"),
    requiem_i18n::key!("custom-status-most-used-emoji-lately"),
    requiem_i18n::key!("custom-status-pick-a-mythical-creature-as-a"),
    requiem_i18n::key!("custom-status-favorite-npc-companion"),
    requiem_i18n::key!("custom-status-what-s-one-of-your-hot"),
    requiem_i18n::key!("custom-status-what-have-you-been-playing"),
    requiem_i18n::key!("custom-status-what-do-you-want-to-ask"),
    requiem_i18n::key!("custom-status-just-ranked-up-in"),
    requiem_i18n::key!("custom-status-what-have-you-been-reading"),
    requiem_i18n::key!("custom-status-what-would-you-like-recommendations-for"),
    requiem_i18n::key!("custom-status-shower-thought"),
    requiem_i18n::key!("custom-status-last-song-stuck-in-your-head"),
    requiem_i18n::key!("custom-status-if-your-life-had-a-theme"),
    requiem_i18n::key!("custom-status-today-i-learned"),
    requiem_i18n::key!("custom-status-what-s-the-most-useless-talent"),
    requiem_i18n::key!("custom-status-what-video-game-item-would-you"),
];

pub fn random_prompt() -> &'static str {
    PROMPTS[emoji_button_icon::random_u32() as usize % PROMPTS.len()]
}

const MAX_STATUS_LENGTH: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearAfter {
    Hours24,
    Hours4,
    Hour1,
    Minutes30,
    Never,
}

impl ClearAfter {
    pub const ALL: [ClearAfter; 5] = [
        ClearAfter::Hours24,
        ClearAfter::Hours4,
        ClearAfter::Hour1,
        ClearAfter::Minutes30,
        ClearAfter::Never,
    ];

    fn key(self) -> &'static str {
        match self {
            ClearAfter::Hours24 => "24h",
            ClearAfter::Hours4 => "4h",
            ClearAfter::Hour1 => "1h",
            ClearAfter::Minutes30 => "30m",
            ClearAfter::Never => "never",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.key() == key)
    }

    fn duration(self) -> Option<ChronoDuration> {
        match self {
            ClearAfter::Hours24 => Some(ChronoDuration::hours(24)),
            ClearAfter::Hours4 => Some(ChronoDuration::hours(4)),
            ClearAfter::Hour1 => Some(ChronoDuration::hours(1)),
            ClearAfter::Minutes30 => Some(ChronoDuration::minutes(30)),
            ClearAfter::Never => None,
        }
    }

    fn lands_at(self, now: DateTime<Local>) -> Option<DateTime<Local>> {
        self.duration().map(|d| now + d)
    }

    pub fn expires_at_ms(self, now: DateTime<Local>) -> Option<u64> {
        self.lands_at(now)
            .map(|at| at.timestamp_millis().max(0) as u64)
    }

    fn option_label(self, now: DateTime<Local>) -> String {
        let span = match self {
            ClearAfter::Hours24 => requiem_i18n::t!("custom-status-24-hours"),
            ClearAfter::Hours4 => requiem_i18n::t!("custom-status-4-hours"),
            ClearAfter::Hour1 => requiem_i18n::t!("custom-status-1-hour"),
            ClearAfter::Minutes30 => requiem_i18n::t!("custom-status-30-minutes"),
            ClearAfter::Never => return requiem_i18n::t!("custom-status-don-t-clear").to_string(),
        };
        let at = self.lands_at(now).expect("timed options land somewhere");
        if at.date_naive() == now.date_naive() {
            requiem_i18n::t!(
                "custom-status-clear-option",
                span = span,
                time = time::clock(&at)
            )
        } else {
            requiem_i18n::t!(
                "custom-status-clear-option-tomorrow",
                span = span,
                time = time::clock(&at)
            )
        }
    }

    fn trigger_label(self, now: DateTime<Local>) -> String {
        match self.lands_at(now) {
            None => requiem_i18n::t!("custom-status-don-t-clear").to_string(),
            Some(at) if at.date_naive() == now.date_naive() => {
                requiem_i18n::t!(
                    "custom-status-clear-at",
                    time = time::clock(&at)
                )
            }
            Some(at) => requiem_i18n::t!(
                "custom-status-clear-tomorrow-at",
                time = time::clock(&at)
            ),
        }
    }
}

#[derive(Clone)]
pub struct CustomStatusModalState {
    pub open: Signal<ToggleState>,
    pub draft_text: Signal<String>,

    pub draft_emoji: Signal<Option<StatusEmoji>>,
    pub clear_after: Signal<ClearAfter>,
    pub select_open: Signal<bool>,
    pub emoji_open: Signal<bool>,
    pub emoji: EmojiPickerSignals,
    pub prompt: Signal<&'static str>,
    preview_text: Signal<String>,
    show_prompt: Signal<bool>,
    show_text: Signal<bool>,
    show_emoji_only: Signal<bool>,
}

impl Default for CustomStatusModalState {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomStatusModalState {
    pub fn new() -> Self {
        Self {
            open: signal(ToggleState::new(false)),
            draft_text: signal(String::new()),
            draft_emoji: signal(None),
            clear_after: signal(ClearAfter::Hours24),
            select_open: signal(false),
            emoji_open: signal(false),
            emoji: EmojiPickerSignals {
                search: signal(String::new()),
                scroll_offset: signal(0.0),
                category_jump: signal(None),
                hovered: signal(None),
                collapsed: signal(Vec::new()),
            },
            prompt: signal(PROMPTS[0]),
            preview_text: signal(String::new()),
            show_prompt: signal(true),
            show_text: signal(false),
            show_emoji_only: signal(false),
        }
    }

    pub fn open_with(&self, current: Option<&CustomStatus>) {
        self.draft_text
            .set(current.map(|c| c.text.clone()).unwrap_or_default());
        self.draft_emoji.set(current.and_then(|c| c.emoji.clone()));

        self.clear_after
            .set(if current.is_some_and(|c| c.expires_at_ms.is_none()) {
                ClearAfter::Never
            } else {
                ClearAfter::Hours24
            });
        self.select_open.set(false);
        self.emoji_open.set(false);
        self.prompt.set(random_prompt());
        self.refresh_preview();
        self.open.update(|t| *t = t.set(true));
    }

    pub fn close(&self) {
        if self.select_open.peek() {
            self.select_open.set(false);
        }
        if self.emoji_open.peek() {
            self.emoji_open.set(false);
        }
        if self.open.peek().on {
            self.open.update(|t| *t = t.set(false));
        }
    }

    fn refresh_preview(&self) {
        let text = self.draft_text.peek();
        let emoji = self.draft_emoji.peek();
        let has_text = !text.trim().is_empty();
        let set = |s: &Signal<bool>, v: bool| {
            if s.peek() != v {
                s.set(v);
            }
        };
        set(&self.show_prompt, !has_text && emoji.is_none());
        set(&self.show_text, has_text);
        set(&self.show_emoji_only, !has_text && emoji.is_some());
        let inline = inline_text(&text, emoji.as_ref());
        if self.preview_text.peek() != inline {
            self.preview_text.set(inline);
        }
    }

    fn draft_status(&self) -> Option<CustomStatus> {
        let status = CustomStatus {
            text: self.draft_text.peek().trim().to_string(),
            emoji: self.draft_emoji.peek(),
            expires_at_ms: self.clear_after.peek().expires_at_ms(Local::now()),
        };
        (!status.is_empty()).then_some(status)
    }
}

const ADD_ICON_GLYPH: &str = "\u{E000}";

fn raster_icon(icon: impl IconPath, color: Color) -> Option<Image> {
    let art = icon.art()?;
    let mut surface = surfaces::raster_n32_premul((48, 48))?;
    let canvas = surface.canvas();
    canvas.clear(Color::TRANSPARENT);
    art.draw_fitted(canvas, (0.0, 0.0, 48.0, 48.0), color, 1.0);
    Some(surface.image_snapshot())
}

static ADD_ICON_MUTED: OnceLock<Option<Image>> = OnceLock::new();
static ADD_ICON_DEFAULT: OnceLock<Option<Image>> = OnceLock::new();

fn add_icon_tile(
    image: &'static OnceLock<Option<Image>>,
    color: Color,
) -> Option<(Image, (f32, f32, f32, f32))> {
    image
        .get_or_init(|| raster_icon(CIRCLE_PLUS, color))
        .as_ref()
        .map(|i| (i.clone(), (0.0, 0.0, 48.0, 48.0)))
}

fn prompt_provider_muted(grapheme: &str) -> Option<(Image, (f32, f32, f32, f32))> {
    if grapheme == ADD_ICON_GLYPH {
        return add_icon_tile(&ADD_ICON_MUTED, TEXT_MUTED);
    }
    twemoji::tile(grapheme)
}

fn prompt_provider_default(grapheme: &str) -> Option<(Image, (f32, f32, f32, f32))> {
    if grapheme == ADD_ICON_GLYPH {
        return add_icon_tile(&ADD_ICON_DEFAULT, TEXT_DEFAULT);
    }
    twemoji::tile(grapheme)
}

const INLINE_EMOJI_SCALE: f32 = 16.0 / 14.0;

fn inline_text(text: &str, emoji: Option<&StatusEmoji>) -> String {
    let text = text.trim();
    match emoji.filter(|e| e.id.is_none()) {
        Some(e) if !text.is_empty() => format!("{} {text}", e.name),
        _ => text.to_string(),
    }
}

fn status_text(
    content: String,
    provider: primitives::EmojiProvider,
    color: Color,
    markdown: MarkdownFeatures,
) -> Node {
    message_text(MessageTextProps {
        content,
        size: 14.0,
        color,
        emoji_provider: Some(provider),
        emoji_scale: Some(INLINE_EMOJI_SCALE),
        markdown: Some(markdown),
        ..Default::default()
    })
}

pub fn inline_status_widget(
    status: &CustomStatus,
    size: f32,
    line_height: f32,
    emoji_px: f32,
    weight: Weight,
    color: Color,
    fonts: &Fonts,
) -> Option<MessageTextWidget> {
    let content = match status.emoji.as_ref().filter(|e| e.id.is_none()) {
        Some(e) if status.text.trim().is_empty() => e.name.clone(),
        emoji => inline_text(&status.text, emoji),
    };
    if content.is_empty() {
        return None;
    }
    let widget = MessageTextWidget::new(
        content,
        size,
        color,
        fonts.text_fonts.font_system.clone(),
        fonts.text_fonts.glyph_cache.clone(),
        Default::default(),
        Default::default(),
    )
    .with_emoji_provider(twemoji::tile)
    .with_emoji_scale(emoji_px / size)
    .with_markdown(MarkdownFeatures::NONE)
    .with_weight(weight)
    .with_line_height(line_height);
    Some(widget)
}

fn emoji_image(emoji: &StatusEmoji, images: &ImageCache, backend: &BackendConfig) -> Option<Node> {
    match emoji.id {
        None => twemoji::tile(&emoji.name).map(|(image, (x, y, w, h))| {
            Node::new(
                "div",
                ImageWidget::new(Some(image.clone())).with_src_rect(x, y, w, h),
            )
        }),
        Some(id) => images
            .get_or_decode_image(&backend.emoji_url(id))
            .map(|image| Node::new("div", ImageWidget::new(Some(image)))),
    }
}

pub enum BubbleContent<'a> {
    Prompt(&'a str),
    Status(&'a CustomStatus),
}

fn bubble_content(
    content: &BubbleContent<'_>,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    match content {
        BubbleContent::Prompt(prompt) => {
            let text = format!("{ADD_ICON_GLYPH} *{}*", requiem_i18n::text(prompt));
            div()
                .class(cls::CONTENT)
                .child(
                    status_text(
                        text.clone(),
                        prompt_provider_muted,
                        TEXT_MUTED,
                        MarkdownFeatures::ALL,
                    )
                    .class(cls::PROMPT_IDLE)
                    .class(cls::CLAMP),
                )
                .child(
                    status_text(
                        text,
                        prompt_provider_default,
                        TEXT_DEFAULT,
                        MarkdownFeatures::ALL,
                    )
                    .class(cls::PROMPT_HOVER)
                    .class(cls::CLAMP),
                )
        }
        BubbleContent::Status(status) => {
            let emoji = status.emoji.as_ref();
            if status.text.trim().is_empty() {
                let image = emoji
                    .and_then(|e| emoji_image(e, images, backend))
                    .map(|n| n.class(cls::EMOJI_ALONE))
                    .unwrap_or_default();
                return div().class(cls::CONTENT).child(image);
            }
            let text = status_text(
                inline_text(&status.text, emoji),
                twemoji::tile,
                TEXT_DEFAULT,
                MarkdownFeatures::NONE,
            )
            .class(cls::CLAMP);
            match emoji
                .filter(|e| e.id.is_some())
                .and_then(|e| emoji_image(e, images, backend))
            {
                Some(image) => div()
                    .class(cls::CONTENT)
                    .class(cls::CONTENT_ROW)
                    .child(image.class(cls::EMOJI_INLINE))
                    .child(text),
                None => div().class(cls::CONTENT).child(text),
            }
        }
    }
}

#[derive(Clone)]
pub struct BubbleActions {
    pub on_open: Rc<dyn Fn()>,
    pub on_clear: Rc<dyn Fn()>,
}

pub fn bubble(
    content: BubbleContent<'_>,
    actions: Option<&BubbleActions>,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    let is_prompt = matches!(content, BubbleContent::Prompt(_));
    let inner = div()
        .class(cls::INNER)
        .child(bubble_content(&content, images, backend));
    let inner = match (actions, is_prompt) {
        (Some(actions), true) => {
            let on_open = actions.on_open.clone();
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![
                    cls::INNER.to_string(),
                    cls::INNER_CLICKABLE.to_string(),
                ])
                .on_click(move || on_open())
                .build()
                .child(bubble_content(&content, images, backend))
        }
        _ => inner,
    };

    let outer = div()
        .class(cls::OUTER)
        .child(div().class(cls::THOUGHT_SMALL))
        .child(div().class(cls::THOUGHT_LARGE))
        .child(inner);
    let mut wrap = div().class(cls::WRAP).child(outer);
    if let Some(actions) = actions {
        wrap = wrap.class(cls::EDITABLE);
        if !is_prompt {
            wrap = wrap.child(toolbar(actions));
        }
    }
    wrap
}

fn toolbar(actions: &BubbleActions) -> Node {
    let on_open = actions.on_open.clone();
    let edit = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::TOOL.to_string(), cls::TOOL_FIRST.to_string()])
        .on_click(move || on_open())
        .build()
        .child(icon(PENCIL, TEXT_SECONDARY).class(cls::TOOL_ICON));
    let on_clear = actions.on_clear.clone();
    let clear = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::TOOL.to_string(), cls::TOOL_LAST.to_string()])
        .on_click(move || on_clear())
        .build()
        .child(icon(TRASH, TEXT_SECONDARY).class(cls::TOOL_ICON));
    let tip = |label: &str, anchor: Node| {
        TooltipProps::default()
            .text(label.to_string())
            .build(anchor)
    };
    div()
        .class(cls::TOOLBAR)
        .child(tip(&requiem_i18n::t!("custom-status-edit"), edit))
        .child(tip(&requiem_i18n::t!("custom-status-clear"), clear))
}

fn preview_bubble(
    state: &CustomStatusModalState,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    let emoji = state.draft_emoji.get();
    let prompt = div()
        .class(cls::CONTENT)
        .bind_class(state.show_prompt.clone(), cls::SHOWN, cls::HIDDEN)
        .child(
            status_text(
                format!("*{}*", requiem_i18n::text(state.prompt.peek())),
                prompt_provider_muted,
                TEXT_MUTED,
                MarkdownFeatures::ALL,
            )
            .class(cls::CLAMP),
        );
    let typed = status_text(
        state.preview_text.peek(),
        twemoji::tile,
        TEXT_DEFAULT,
        MarkdownFeatures::NONE,
    )
    .class(cls::CLAMP)
    .bind_text(state.preview_text.clone());
    let mut text =
        div()
            .class(cls::CONTENT)
            .bind_class(state.show_text.clone(), cls::SHOWN, cls::HIDDEN);
    if let Some(image) = emoji
        .as_ref()
        .filter(|e| e.id.is_some())
        .and_then(|e| emoji_image(e, images, backend))
    {
        text = text
            .class(cls::CONTENT_ROW)
            .child(image.class(cls::EMOJI_INLINE));
    }
    let text = text.child(typed);
    let alone = div()
        .class(cls::CONTENT)
        .bind_class(state.show_emoji_only.clone(), cls::SHOWN, cls::HIDDEN)
        .child(
            emoji
                .as_ref()
                .and_then(|e| emoji_image(e, images, backend))
                .map(|n| n.class(cls::EMOJI_ALONE))
                .unwrap_or_default(),
        );
    let inner = div()
        .class(cls::INNER)
        .child(prompt)
        .child(text)
        .child(alone);
    let outer = div()
        .class(cls::OUTER)
        .child(div().class(cls::THOUGHT_SMALL))
        .child(div().class(cls::THOUGHT_LARGE))
        .child(inner);
    div().class(cls::WRAP).child(outer)
}

#[allow(clippy::too_many_arguments)]
pub fn build_modal(
    state: &CustomStatusModalState,
    open: ToggleState,
    user: &models::User,
    profile: Option<&models::UserProfile>,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    on_save: Rc<dyn Fn(Option<CustomStatus>)>,
) -> Node {
    let dismiss = {
        let state = state.clone();
        move || state.close()
    };
    let save = {
        let (state, on_save) = (state.clone(), on_save.clone());
        Rc::new(move || {
            on_save(state.draft_status());
            state.close();
        })
    };

    let banner_image = profile
        .and_then(|p| p.user.banner.as_deref())
        .and_then(|hash| images.get_or_decode_image(&backend.banner_url(user.id, hash)));
    let banner = match banner_image {
        Some(image) => Node::new("div", ImageWidget::new(Some(image))).class(cls::PREVIEW_BANNER),
        None => {
            let color = profile_popout::banner_color(
                user.id,
                &user.discriminator,
                profile.and_then(|p| p.user.accent_color),
                user.avatar.as_deref(),
                images,
                backend,
            );
            Node::new("div", profile_popout::ColorBoxWidget { color })
                .class(cls::PREVIEW_BANNER)
        }
    };
    let mut avatar = ui::user_avatar_props(user.id, &user.discriminator, fonts);
    if let Some(image) = user
        .avatar
        .as_deref()
        .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(user.id, hash)))
    {
        avatar = avatar.image(image);
    }
    let avatar = avatar_with_status(
        avatar,
        AvatarStatusSize::Px80,
        Some(PresenceStatus::Online),
    )
    .class(cls::PREVIEW_AVATAR);
    let header = div()
        .class(cls::PREVIEW_HEADER)
        .child(banner)
        .child(div().class(cls::PREVIEW_AVATAR_RING))
        .child(avatar)
        .child(preview_bubble(state, images, backend).class(cls::PREVIEW_BUBBLE_SLOT));
    let display_name = user
        .global_name
        .clone()
        .unwrap_or_else(|| user.username.clone());
    let name = TextProps::styled(display_name, TextVariant::TEXT_LG_BOLD)
        .color(TEXT_DEFAULT)
        .line_clamp(1_u32)
        .build();
    let username = TextProps::styled(user.username.clone(), TextVariant::TEXT_SM_NORMAL)
        .color(TEXT_STRONG)
        .build();
    let mut username_row = div().class(cls::PREVIEW_USERNAME_ROW).child(username);
    if let Some(badges) =
        profile.and_then(|p| profile_popout::badges_row(p, images, backend))
    {
        username_row = username_row.child(badges);
    }
    let names = StackProps::vertical()
        .gap(0)
        .full_width(false)
        .padding(Padding::top(8).with_right(16).with_bottom(16).with_left(16))
        .build()
        .child(
            StackProps::horizontal()
                .align(Align::Center)
                .gap(0)
                .full_width(false)
                .build()
                .child(name),
        )
        .child(username_row);
    let card = div().class(cls::PREVIEW_CARD).child(header).child(names);
    let preview = div().class(cls::PREVIEW).child(card);

    let emoji = state.draft_emoji.get();
    let emoji_glyph = match emoji.as_ref().and_then(|e| emoji_image(e, images, backend)) {
        Some(image) => image.class(cls::FIELD_EMOJI),
        None => Node::new(
            "span",
            EmojiButtonIconWidget::new(),
        )
        .class(cls::FIELD_EMOJI),
    };
    let emoji_button = {
        let (open, hovered) = (state.emoji_open.clone(), state.emoji.clone());
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::FIELD_EMOJI_BTN.to_string()])
            .on_click(move || {
                let opening = !open.peek();
                if opening {
                    hovered.reset_for_open();
                }
                open.set(opening);
            })
            .build()
            .child(emoji_glyph)
    };
    let field = {
        let (draft, state_for_change, save) =
            (state.draft_text.clone(), state.clone(), save.clone());
        TextInputProps::new()
            .text(state.draft_text.peek())
            .placeholder(requiem_i18n::text(state.prompt.peek()))
            .max_length(MAX_STATUS_LENGTH)
            .clearable(true)
            .autofocus(true)
            .leading(emoji_button)
            .on_change(move |text: &str| {
                draft.set(text.to_string());
                state_for_change.refresh_preview();
            })
            .on_submit(move |_text: String| save())
            .build()
    };
    let label = TextProps::styled(
        requiem_i18n::t!("custom-status-status").to_string(),
        TextVariant::TEXT_MD_SEMIBOLD,
    )
    .color(TEXT_DEFAULT)
    .build();
    let mut field_wrap = div().class(cls::FIELD_WRAP).child(field);
    if state.emoji_open.get() {
        let on_pick = {
            let state = state.clone();
            Rc::new(move |glyph: String, _keep_open: bool| {
                state.draft_emoji.set(Some(StatusEmoji {
                    id: None,
                    name: glyph,
                    animated: false,
                }));
                state.emoji_open.set(false);
                state.refresh_preview();
            }) as Rc<dyn Fn(String, bool)>
        };
        let picker = emoji_picker::build_emoji_only(&state.emoji, on_pick, None);
        field_wrap = field_wrap.child(
            Node::new("div", TopLayerWidget)
                .class(cls::PICKER_LAYER)
                .child(picker),
        );
    }
    let form = div().class(cls::FORM).child(label).child(field_wrap);

    let now = Local::now();
    let options: Vec<SelectOption> = ClearAfter::ALL
        .iter()
        .map(|c| {
            SelectOption::new(c.key(), c.option_label(now))
                .with_selected_label(c.trigger_label(now))
                .with_group(requiem_i18n::t!("custom-status-clear-after"))
        })
        .collect();
    let clear_after = {
        let (open, select_open, clear_after) = (
            state.select_open.clone(),
            state.select_open.clone(),
            state.clear_after.clone(),
        );
        SelectProps::new(
            move || open.set(!open.peek()),
            move |value: String| {
                if let Some(choice) = ClearAfter::from_key(&value) {
                    clear_after.set(choice);
                }
                select_open.set(false);
            },
        )
        .options(options)
        .selected(state.clear_after.get().key().to_string())
        .open(state.select_open.get())
        .text_only(true)
        .panel_above(true)
        .build()
        .class(cls::CLEAR_AFTER)
    };
    let save_button = {
        let save = save.clone();
        ButtonProps::default()
            .label(requiem_i18n::t!("custom-status-save").to_string())
            .on_click(move || save())
            .build()
    };

    let title = TextProps::styled(
        requiem_i18n::t!("custom-status-set-your-status").to_string(),
        TextVariant::TEXT_LG_SEMIBOLD,
    )
    .color(TEXT_STRONG)
    .build();
    ModalProps::default()
        .size(ModalSize::Medium)
        .title(title)
        .dismissable(true)
        .on_dismiss(dismiss)
        .body(
            StackProps::vertical()
                .gap(0)
                .full_width(false)
                .build()
                .child(preview)
                .child(form),
        )
        .footer(vec![save_button])
        .footer_leading(clear_after)
        .open(open)
        .build()
}

pub fn local_time(ms: u64) -> Option<DateTime<Local>> {
    Local.timestamp_millis_opt(ms as i64).single()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn clear_after_labels_match_discord() {
        let now = Local.with_ymd_and_hms(2026, 9, 27, 22, 42, 0).unwrap();
        assert_eq!(
            ClearAfter::Hours24.option_label(now),
            "24 hours (tomorrow at 10:42 PM)"
        );
        assert_eq!(
            ClearAfter::Hours4.option_label(now),
            "4 hours (tomorrow at 2:42 AM)"
        );
        assert_eq!(ClearAfter::Hour1.option_label(now), "1 hour (11:42 PM)");
        assert_eq!(
            ClearAfter::Minutes30.option_label(now),
            "30 minutes (11:12 PM)"
        );
        assert_eq!(
            ClearAfter::Hours24.trigger_label(now),
            "Clear tomorrow at 10:42 PM"
        );
        assert_eq!(ClearAfter::Hour1.trigger_label(now), "Clear at 11:42 PM");
        assert_eq!(ClearAfter::Never.trigger_label(now), "Don\u{2019}t clear");
        assert_eq!(ClearAfter::Never.expires_at_ms(now), None);
    }

    #[test]
    fn inline_text_prefixes_unicode_emoji_only() {
        let melon = StatusEmoji {
            id: None,
            name: "\u{1f349}".into(),
            animated: false,
        };
        assert_eq!(inline_text(" hi ", Some(&melon)), "\u{1f349} hi");
        let custom = StatusEmoji {
            id: Some(1),
            name: "pepe".into(),
            animated: false,
        };
        assert_eq!(inline_text("hi", Some(&custom)), "hi");
        assert_eq!(inline_text("", Some(&melon)), "");
    }
}
