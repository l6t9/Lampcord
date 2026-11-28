pub use crate::features::account_popout;
pub use crate::features::channel_sidebar;
pub use crate::features::channel_start::{channel_start, dm_start};
pub use crate::features::typing::typing_bar;
pub use crate::features::link_warning::link_warning;
pub use crate::features::composer::{command_composer, commands_panel, time_panel};
pub use crate::features::context_menus::{context_menu, context_menus};
pub use crate::features::create_channel;
pub use crate::features::create_guild;
pub use crate::features::custom_status;
pub use crate::features::devtools;
pub use crate::features::emoji::{
    emoji_picker, emoji_popout, emoji_skin, emoji_style_menu, emoji_usage,
};
pub use crate::features::gif_picker;
pub use crate::features::guild_rail;
pub use crate::features::guild_settings;
pub use crate::features::home;
pub use crate::features::inbox;
pub use crate::features::invite::{invite_embed, invite_modal};
pub use crate::features::loading_screen;
pub use crate::features::media_viewer;
pub use crate::features::member_list;
pub use crate::features::messages::{
    attachments, join_message, message_components, message_confirm, message_edit, message_embed, message_keys,
    message_list, message_toolbar, pending_uploads, pins, reactions, reply,
};
pub use crate::features::forward::components::forward;
pub use crate::features::new_message;
pub use crate::features::playground;
pub use crate::features::plugins::{plugin_install, plugin_modal, plugin_parts, plugin_view};
pub use crate::features::profile::{
    dm_profile, full_profile, profile_card, profile_decorations, profile_image, profile_popout,
    profile_theme,
};
pub use crate::features::quick_switcher;
pub use crate::features::search::{search, search_art, search_filters, search_state};
pub use crate::features::shortcuts::shortcuts_dialog;
pub use crate::features::threads::{thread_browser, threads};
pub use crate::features::user_panel;
pub use crate::features::user_settings::{
    language_settings, notification_settings, plugins_settings,
};
#[cfg(feature = "video")]
pub use crate::features::video_player;
pub use crate::features::voice::{audio_menu, call_view, voice_panel, voice_users};
pub mod author_style;
pub mod avatar_color;
pub mod built_in_commands;
pub mod code_block;
pub mod dialog;
pub mod dim;
pub mod emoji_anim;
pub mod emoji_button_icon;
pub mod emoji_names;
mod emoji_names_data;
mod emoji_pack;
pub mod requiem_emoji {
    pub use super::emoji_pack::*;
}
pub mod emoji_text;
pub mod logo;
pub mod lottie_icon;
pub mod modal_stack;
pub mod navigation;
pub mod permission_spec;
pub mod popouts;
pub mod read_state;
pub mod role_popout;
pub mod sidebar_resize;
pub mod slash_commands;
pub mod timestamps;
pub mod twemoji;
mod twemoji_atlas_data;
use crate::models::id::marker;
use crate::state;
#[cfg(feature = "video")]
use crate::video;
use requiem_ui::font_system;
use requiem_ui::icons as i;
use requiem_ui::node;
use requiem_ui::signal;
use requiem_ui::text_cache;
use requiem_ui::widget::primitives::AvatarShape;
use requiem_ui::widget::primitives::ScrollPosition;
use requiem_ui::widget::primitives;
use resvg::tiny_skia::Pixmap;
use resvg::tiny_skia::Transform;
use resvg::usvg::Tree;
use skia_safe::images;
use std::cell;
use std::fmt as std_fmt;
use std::mem;
use std::sync;

use requiem_ui::components::AvatarProps;
use requiem_ui::components::toast::ToastState;
use requiem_ui::icons::Icon;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use requiem_ui::signal::Signal;
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::primitives::BoxWidget;
use requiem_ui::widget::{WidgetId, WidgetTree};
use skia_safe::Typeface;
use tokio::sync::mpsc::UnboundedSender;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::models::{Channel, Id, Permissions};
use crate::query::QueryClient;
use crate::rest::HttpClient;
use crate::state::AppState;
use requiem_ui::icons::CLYDE;

pub use requiem_ui::colors::{
    TEXT_DEFAULT, TEXT_MUTED, TEXT_PLACEHOLDER, TEXT_SECONDARY, TEXT_STRONG, TEXT_SUBTLE,
};

#[derive(Clone, Debug)]
pub struct MessageListScroll {
    pub position: ScrollPosition,
}

#[derive(Clone, Debug)]
pub struct SettingsUiState {
    pub category: String,

    pub email_revealed: bool,

    pub sub_page: Option<SettingsSubPage>,

    pub logout_confirm: ToggleState,

    pub profile_draft: Signal<ProfileDraft>,
    pub profile_dialog: Signal<ProfileDialogState>,

    pub plugin_install: Signal<plugin_install::PluginInstallState>,
    pub profile_tab: ProfileTab,

    pub bio_emoji_open: Signal<bool>,

    pub color_popout: Signal<Option<ColorTarget>>,

    pub profile_server_select_open: bool,

    pub profile_epoch: u32,
}

#[derive(Clone, Debug, Default)]
pub struct ProfileDraft {
    pub seeded: bool,
    pub display_name: String,
    pub pronouns: String,
    pub bio: String,
    pub accent_color: Option<i64>,

    pub theme_colors: Option<(i64, i64)>,

    pub preview_theme: Option<(i64, i64)>,

    pub preview_preset: usize,

    pub avatar: PendingImage,
    pub banner: PendingImage,
    pub saved_display_name: String,
    pub saved_pronouns: String,
    pub saved_bio: String,
    pub saved_accent_color: Option<i64>,
    pub saved_theme_colors: Option<(i64, i64)>,

    pub server: ServerDraft,
}

#[derive(Clone)]
pub struct PendingUpload {
    pub data_uri: Rc<str>,

    pub preview: skia_safe::Image,
}

impl std_fmt::Debug for PendingUpload {
    fn fmt(&self, f: &mut std_fmt::Formatter<'_>) -> std_fmt::Result {
        write!(
            f,
            "PendingUpload({} bytes of data URI)",
            self.data_uri.len()
        )
    }
}

#[derive(Clone, Debug, Default)]
pub enum PendingImage {
    #[default]
    Unchanged,

    Removed,
    Set(PendingUpload),

    Saved(PendingUpload),
}

impl PendingImage {
    pub fn is_changed(&self) -> bool {
        matches!(self, PendingImage::Set(_) | PendingImage::Removed)
    }

    pub fn shown(&self) -> Option<&PendingUpload> {
        match self {
            PendingImage::Set(upload) | PendingImage::Saved(upload) => Some(upload),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ServerDraft {
    pub guild_id: Option<Id<GuildMarker>>,

    pub seeded: bool,
    pub nick: String,
    pub saved_nick: String,
}

impl ProfileDraft {
    pub const DEFAULT_THEME: (i64, i64) = (0x747c8c, 0xe3e5e8);

    pub fn preview_colors(&self) -> (i64, i64) {
        self.preview_theme.unwrap_or_else(|| {
            let preset = &NITRO_PRESETS[self.preview_preset % NITRO_PRESETS.len()];
            (preset.primary, preset.accent)
        })
    }

    pub fn dirty(&self) -> bool {
        (self.seeded
            && (self.display_name != self.saved_display_name
                || self.pronouns != self.saved_pronouns
                || self.bio != self.saved_bio
                || self.accent_color != self.saved_accent_color
                || self.theme_colors != self.saved_theme_colors
                || self.avatar.is_changed()
                || self.banner.is_changed()))
            || (self.server.seeded && self.server.nick != self.server.saved_nick)
    }

    pub fn seed(
        &mut self,
        display_name: String,
        pronouns: String,
        bio: String,
        accent_color: Option<i64>,
        theme_colors: Option<(i64, i64)>,
    ) {
        let server = mem::take(&mut self.server);
        *self = ProfileDraft {
            seeded: true,
            saved_display_name: display_name.clone(),
            saved_pronouns: pronouns.clone(),
            saved_bio: bio.clone(),
            saved_accent_color: accent_color,
            saved_theme_colors: theme_colors,
            display_name,
            pronouns,
            bio,
            accent_color,
            theme_colors,
            preview_theme: None,
            preview_preset: 0,
            avatar: PendingImage::Unchanged,
            banner: PendingImage::Unchanged,
            server,
        };
    }

    pub fn reset(&mut self) {
        self.display_name = self.saved_display_name.clone();
        self.pronouns = self.saved_pronouns.clone();
        self.bio = self.saved_bio.clone();
        self.accent_color = self.saved_accent_color;
        self.theme_colors = self.saved_theme_colors;
        self.avatar = PendingImage::Unchanged;
        self.banner = PendingImage::Unchanged;
        self.server.nick = self.server.saved_nick.clone();
    }
}

pub struct NitroPreset {
    pub name: &'static str,
    pub primary: i64,
    pub accent: i64,
    pub name_color: Option<i64>,
}

pub const NITRO_PRESETS: [NitroPreset; 5] = [
    NitroPreset {
        name: requiem_i18n::key!("ui-neon-space"),
        primary: 0x5d0068,
        accent: 0x0f603e,
        name_color: Some(0x8cf5be),
    },
    NitroPreset {
        name: requiem_i18n::key!("ui-starlit-dream"),
        primary: 0x4e2ea7,
        accent: 0x090321,
        name_color: Some(0xc9b8ff),
    },
    NitroPreset {
        name: requiem_i18n::key!("ui-shadow-realm"),
        primary: 0x000000,
        accent: 0x440808,
        name_color: Some(0xff5252),
    },
    NitroPreset {
        name: requiem_i18n::key!("ui-above-the-clouds"),
        primary: 0x0b7a98,
        accent: 0x8c9e66,
        name_color: None,
    },
    NitroPreset {
        name: requiem_i18n::key!("ui-cyberpunk"),
        primary: 0x1e0777,
        accent: 0x0b5484,
        name_color: Some(0xb7a6ff),
    },
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProfileTab {
    #[default]
    Main,
    PerServer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorTarget {
    Banner,
    ThemePrimary,
    ThemeAccent,

    PreviewPrimary,
    PreviewAccent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageTarget {
    Avatar,
    Banner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogKind {
    AvatarDecoration,
    Nameplate,
    ProfileEffect,
    ProfileFrame,
    DisplayNameStyle,
    Widgets,
}

impl CatalogKind {
    pub fn title(self) -> String {
        match self {
            CatalogKind::AvatarDecoration => requiem_i18n::t!("ui-change-avatar-decoration"),
            CatalogKind::Nameplate => requiem_i18n::t!("ui-change-nameplate"),
            CatalogKind::ProfileEffect => requiem_i18n::t!("ui-change-profile-effect"),
            CatalogKind::ProfileFrame => requiem_i18n::t!("ui-change-profile-frame"),
            CatalogKind::DisplayNameStyle => requiem_i18n::t!("ui-change-display-name-style"),
            CatalogKind::Widgets => requiem_i18n::t!("ui-add-widgets"),
        }
    }

    pub fn collectible_type(self) -> Option<i32> {
        match self {
            CatalogKind::AvatarDecoration => Some(0),
            CatalogKind::ProfileEffect => Some(1),
            CatalogKind::Nameplate => Some(2),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProfileDialog {
    Image {
        target: ImageTarget,
        upload: PendingUpload,
    },
    Catalog(CatalogKind),
}

#[derive(Clone, Debug)]
pub struct ProfileDialogState {
    pub dialog: Option<ProfileDialog>,
    pub open: ToggleState,
}

impl Default for ProfileDialogState {
    fn default() -> Self {
        Self {
            dialog: None,
            open: ToggleState::new(false),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsSubPage {
    LoggedInDevices,
}

impl SettingsSubPage {
    pub fn title(self) -> String {
        match self {
            SettingsSubPage::LoggedInDevices => requiem_i18n::t!("ui-logged-in-devices"),
        }
    }
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self {
            category: requiem_i18n::key!("settings-modal-account").to_string(),
            email_revealed: false,
            sub_page: None,
            logout_confirm: ToggleState::new(false),
            profile_draft: signal::signal(ProfileDraft::default()),
            profile_dialog: signal::signal(ProfileDialogState::default()),
            plugin_install: signal::signal(
                plugin_install::PluginInstallState::default(),
            ),
            profile_tab: ProfileTab::Main,
            bio_emoji_open: signal::signal(false),
            color_popout: signal::signal(None),
            profile_server_select_open: false,
            profile_epoch: 0,
        }
    }
}

impl Default for MessageListScroll {
    fn default() -> Self {
        Self {
            position: ScrollPosition::Pinned,
        }
    }
}

pub const THEME_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/bundled.css"));

pub mod classes {
    include!(concat!(env!("OUT_DIR"), "/css_classes.rs"));
}

#[derive(Clone)]
pub struct Fonts {
    pub typeface: Typeface,
    pub text_fonts: font_system::TextFonts,
}

#[derive(Clone)]
pub struct RootContext {
    pub fonts: Fonts,
    pub backend: BackendConfig,
    pub images: Rc<RefCell<Option<ImageCache>>>,
    pub queries: Rc<RefCell<Option<QueryClient>>>,
    pub http: HttpClient,
    pub compose_tx: UnboundedSender<(Id<ChannelMarker>, String)>,

    pub measure_cache: Rc<text_cache::MeasureCache>,

    #[cfg(feature = "video")]
    pub videos: video::VideoCache,
}

#[derive(Clone)]
pub struct RootSignals {
    pub selected_channel: Signal<Option<Id<ChannelMarker>>>,
    pub on_select_channel: Rc<dyn Fn(Id<ChannelMarker>)>,

    pub composer_focused: Signal<bool>,

    pub composer_has_text: Signal<bool>,

    pub composer_send_toggle: Signal<ToggleState>,
    pub message_scroll: Signal<HashMap<Id<ChannelMarker>, MessageListScroll>>,
    pub mute_toggle: Signal<ToggleState>,
    pub on_toggle_mute: Rc<dyn Fn()>,
    pub deafen_toggle: Signal<ToggleState>,
    pub on_toggle_deafen: Rc<dyn Fn()>,
    pub settings_modal_open: Signal<ToggleState>,
    pub on_open_settings: Rc<dyn Fn()>,
    pub on_close_settings: Rc<dyn Fn()>,

    pub on_log_out: Rc<dyn Fn()>,
    pub settings_ui: Signal<SettingsUiState>,

    pub emoji_picker: emoji_picker::EmojiPickerState,

    pub attachments: attachments::AttachmentsState,

    pub media_viewer: media_viewer::MediaViewerState,

    pub toasts: ToastState,

    pub profile_popout: profile_popout::ProfilePopoutState,

    pub member_list: member_list::MemberListState,

    pub search: search_state::SearchState,

    pub account_popout: account_popout::AccountPopoutState,

    pub global_tick: Signal<u64>,

    #[cfg(feature = "video")]
    pub fullscreen_video: Signal<Option<video_player::FullscreenVideoState>>,
}

pub fn detach_only_child(
    mut section_tree: WidgetTree,
    inner_root: WidgetId,
) -> node::Node {
    section_tree.nodes[inner_root]
        .children
        .first()
        .copied()
        .map(|only_child| detach(&mut section_tree, only_child))
        .unwrap_or_default()
}

pub fn detach(tree: &mut WidgetTree, id: WidgetId) -> node::Node {
    let wants_focus = tree.focused == Some(id);
    let wants_focus_scope = tree.focus_scope == Some(id);
    let node = &mut tree.nodes[id];
    let tag = node.tag;
    let classes = mem::take(&mut node.classes);
    let dom_id = node.dom_id.take();

    let class_binding = node.class_binding.take();

    let image_binding = node.image_binding.take();
    let text_binding = node.text_binding.take();
    let inline_style = node.inline_style_source.take();

    let reactive = node.reactive.take();
    let context_menu = node.context_menu.take();
    let size_observer = node.size_observer.take();
    let rect_observer = node.rect_observer.take();
    let children_ids = if reactive.is_some() {
        Vec::new()
    } else {
        node.children.clone()
    };

    let widget = mem::replace(&mut tree.nodes[id].widget, Box::new(BoxWidget));
    let children = children_ids
        .into_iter()
        .map(|child_id| detach(tree, child_id))
        .collect();
    node::Node {
        tag,
        classes,
        dom_id,
        widget,
        children,
        wants_focus,
        wants_focus_scope,
        class_binding,
        image_binding,
        text_binding,
        inline_style,
        style_binding: None,
        size_observer,
        rect_observer,
        reveal: tree.nodes[id].reveal.clone(),
        reactive,
        context_menu,
        retain: tree.nodes[id].retain,
        focus_ring_host: tree.nodes[id].focus_ring_host,
        mount_hook: tree.nodes[id].mount_hook.take(),
    }
}

pub fn borrow_images(cell: &Rc<RefCell<Option<ImageCache>>>) -> cell::Ref<'_, ImageCache> {
    cell::Ref::map(cell.borrow(), |opt| {
        opt.as_ref().expect("ImageCache not yet installed")
    })
}

pub fn borrow_queries(cell: &Rc<RefCell<Option<QueryClient>>>) -> cell::Ref<'_, QueryClient> {
    cell::Ref::map(cell.borrow(), |opt| {
        opt.as_ref().expect("QueryClient not yet installed")
    })
}

pub fn decode_image(bytes: &[u8]) -> Option<skia_safe::Image> {
    skia_safe::Image::from_encoded(skia_safe::Data::new_copy(bytes))
}

pub fn decode_svg(svg: &[u8], width: u32, height: u32) -> Option<skia_safe::Image> {
    let tree = Tree::from_data(svg, &resvg::usvg::Options::default()).ok()?;
    let mut pixmap = Pixmap::new(width, height)?;
    let size = tree.size();
    let transform = Transform::from_scale(
        width as f32 / size.width(),
        height as f32 / size.height(),
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let info = skia_safe::ImageInfo::new(
        (width as i32, height as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Premul,
        None,
    );
    images::raster_from_data(
        &info,
        skia_safe::Data::new_copy(pixmap.data()),
        width as usize * 4,
    )
}

pub fn asset_file(name: &str) -> Option<sync::Arc<[u8]>> {
    requiem_asset_store::get()?.file_bytes(name)
}

pub fn asset_image(name: &str) -> Option<skia_safe::Image> {
    decode_image(&asset_file(name)?)
}

pub fn asset_svg(name: &str, width: u32, height: u32) -> Option<skia_safe::Image> {
    decode_svg(&asset_file(name)?, width, height)
}

#[derive(Clone, Debug)]
pub enum Destination {
    Friends,
    Dm(state::DmChannel),
    Guild,
}

pub fn destination(state: &AppState, selected_channel: Option<Id<ChannelMarker>>) -> Destination {
    let home = state.nav.home.get();
    match selected_channel {
        Some(id) => match state.dms.find(id) {
            Some(dm) => Destination::Dm(dm),
            None => Destination::Guild,
        },
        None if home => Destination::Friends,
        None => Destination::Guild,
    }
}

pub fn resolve_selected_guild(
    state: &AppState,
    selected_channel: Option<Id<ChannelMarker>>,
) -> Option<Id<GuildMarker>> {
    if !matches!(destination(state, selected_channel), Destination::Guild) {
        return None;
    }
    selected_channel
        .and_then(|id| find_channel(state, id))
        .map(|(guild_id, _)| guild_id)
        .or_else(|| {
            state
                .guilds
                .list
                .with(|guilds| guilds.first().map(|g| g.id))
        })
}

pub fn find_channel(
    state: &AppState,
    channel_id: Id<ChannelMarker>,
) -> Option<(Id<GuildMarker>, Channel)> {
    let found = state.channels.by_guild.with_untracked(|by_guild| {
        by_guild.iter().find_map(|(guild_id, channels)| {
            channels
                .iter()
                .find(|c| c.id == channel_id)
                .map(|c| (*guild_id, c.clone()))
        })
    });
    if let Some((guild_id, _)) = &found {
        state.channels.by_guild.watch(guild_id);
    }
    found
}

pub fn displayed_channel(
    state: &AppState,
    selected_channel: Option<Id<ChannelMarker>>,
) -> Option<Channel> {
    match destination(state, selected_channel) {
        Destination::Friends => None,
        Destination::Dm(dm) => Some(dm.as_channel()),
        Destination::Guild => selected_channel
            .and_then(|id| find_channel(state, id))
            .map(|(_, c)| c)
            .filter(|c| state.can(c, Permissions::VIEW_CHANNEL))
            .or_else(|| default_channel(state, resolve_selected_guild(state, selected_channel))),
    }
}

pub fn can_attach_files(state: &AppState, selected_channel: Option<Id<ChannelMarker>>) -> bool {
    displayed_channel(state, selected_channel)
        .is_none_or(|c| state.can(&c, Permissions::SEND_MESSAGES | Permissions::ATTACH_FILES))
}

pub fn channel_icon(state: &AppState, channel: &Channel) -> Icon {
    let limited = channel
        .guild_id
        .is_some_and(|guild| state.permissions.role_required(guild, channel));
    channel_icon_for(channel, limited)
}

pub fn channel_icon_for(channel: &Channel, limited: bool) -> Icon {
    let nsfw = channel.nsfw;
    match channel.kind {
        2 if nsfw => i::VOICE_WARNING,
        2 if limited => i::VOICE_LOCK,
        2 => i::VOICE_NORMAL,
        4 => i::FOLDER,
        5 if nsfw => i::ANNOUNCEMENTS_WARNING,
        5 if limited => i::ANNOUNCEMENTS_LOCK,
        5 => i::ANNOUNCEMENTS,
        10 | 11 if nsfw => i::THREAD_WARNING,
        10 | 11 => i::THREAD,
        12 => i::THREAD_LOCK,
        13 if limited => i::STAGE_LOCK,
        13 => i::STAGE,
        14 => i::HUB,
        15 if nsfw => i::FORUM_WARNING,
        15 if limited => i::FORUM_LOCK,
        15 => i::FORUM,
        16 if nsfw => i::IMAGE_WARNING,
        16 if limited => i::IMAGE_LOCK,
        16 => i::IMAGE,
        _ if nsfw => i::TEXT_WARNING,
        _ if limited => i::TEXT_LOCK,
        _ => i::TEXT,
    }
}

pub fn default_channel(state: &AppState, guild_id: Option<Id<GuildMarker>>) -> Option<Channel> {
    let guild_id = guild_id?;
    state.channels.by_guild.watch(&guild_id);
    state.channels.by_guild.with_untracked(|by_guild| {
        let mut channels: Vec<&Channel> = by_guild
            .get(&guild_id)?
            .iter()
            .filter(|c| c.kind == 0 && state.can(c, Permissions::VIEW_CHANNEL))
            .collect();
        channels.sort_by_key(|c| c.position.unwrap_or(0));
        channels.first().map(|c| (*c).clone())
    })
}

pub const DEFAULT_AVATAR_ICON: primitives::FallbackIcon =
    primitives::FallbackIcon {
        path: CLYDE,
        width_fraction: 152.0 / 256.0,
    };

pub const DEFAULT_AVATAR_COLORS: [skia_safe::Color; 6] = [
    skia_safe::Color::from_rgb(0x58, 0x65, 0xf2),
    skia_safe::Color::from_rgb(0x75, 0x7e, 0x8a),
    skia_safe::Color::from_rgb(0x3b, 0xa5, 0x5c),
    skia_safe::Color::from_rgb(0xfa, 0xa6, 0x1a),
    skia_safe::Color::from_rgb(0xed, 0x42, 0x45),
    skia_safe::Color::from_rgb(0xeb, 0x45, 0x9f),
];

pub fn default_avatar_color(
    user_id: Id<marker::UserMarker>,
    discriminator: &str,
) -> skia_safe::Color {
    let index = match discriminator.parse::<u64>() {
        Ok(tag) if tag != 0 => (tag % 5) as usize,
        _ => ((user_id.get() >> 22) % 6) as usize,
    };
    DEFAULT_AVATAR_COLORS[index]
}

pub fn user_avatar_props(
    user_id: Id<marker::UserMarker>,
    discriminator: &str,
    fonts: &Fonts,
) -> AvatarProps {
    AvatarProps::default()
        .fallback_background(default_avatar_color(user_id, discriminator))
        .fallback_icon(DEFAULT_AVATAR_ICON)
        .typeface(fonts.typeface.clone())
        .shape(AvatarShape::Circle)
}

pub fn with_avatar_decoration(
    mut props: AvatarProps,
    asset: Option<&str>,
    images: &ImageCache,
    backend: &BackendConfig,
    size: u32,
) -> AvatarProps {
    if let Some(asset) = asset {
        if let Some(image) = images.get_or_decode_image(&backend.avatar_decoration_url(asset, size))
        {
            props = props.decoration(image);
        }
    }
    props
}

#[cfg(test)]
mod asset_raster_tests {

    use crate::test_assets;
    use skia_safe::surfaces;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn composited_on_white(image: &skia_safe::Image) -> (i32, i32, Vec<u8>) {
        let (w, h) = (image.width(), image.height());
        let mut surface = surfaces::raster_n32_premul((w, h)).unwrap();
        surface.canvas().clear(skia_safe::Color::WHITE);
        surface.canvas().draw_image(image, (0, 0), None);
        let info = skia_safe::ImageInfo::new(
            (w, h),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );
        let mut pixels = vec![0u8; (w * h * 4) as usize];
        assert!(surface.read_pixels(&info, &mut pixels, (w * 4) as usize, (0, 0)));
        (w, h, pixels)
    }

    #[test]
    fn rasterized_svgs_match_the_formerly_bundled_images() {
        let Ok(reference) = env::var("ASSET_REFERENCE_ASSETS") else {
            return;
        };
        if !test_assets::install() {
            return;
        }
        let reference = PathBuf::from(reference);
        let core = reference.join("crates/requiem-core/assets");
        let auth = reference.join("crates/requiem/src/ui/auth");
        let mut cases: Vec<(String, PathBuf, u32, u32)> = Vec::new();
        for stem in [
            "acrobat",
            "ae",
            "ai",
            "archive",
            "audio",
            "code",
            "document",
            "drop_image",
            "photoshop",
            "sketch",
            "spreadsheet",
            "unknown",
            "video",
            "webcode",
        ] {
            cases.push((
                format!("file_icon/{stem}"),
                core.join(format!("file_icons/{stem}.png")),
                144,
                192,
            ));
        }
        for (stem, px) in [
            ("artists", 80),
            ("create_my_own", 80),
            ("discover", 160),
            ("friends", 71),
            ("gaming", 60),
            ("intent_community", 80),
            ("intent_friends", 71),
            ("local_community", 80),
            ("school_club", 80),
            ("study_group", 80),
        ] {
            cases.push((
                format!("create_guild/{stem}"),
                core.join(format!("create_guild/{stem}.png")),
                px,
                px,
            ));
        }
        cases.push((
            "image/no_results".into(),
            core.join("quick_switcher/no_results.png"),
            200,
            208,
        ));
        cases.push((
            "image/auth_wordmark".into(),
            auth.join("wordmark.png"),
            248,
            48,
        ));
        cases.push((
            "image/auth_background".into(),
            auth.join("background.jpg"),
            1920,
            1200,
        ));

        let mut worst = 0.0f64;
        for (name, old_path, w, h) in cases {
            let old = super::decode_image(
                &fs::read(&old_path).unwrap_or_else(|e| panic!("{}: {e}", old_path.display())),
            )
            .expect("old image decodes");
            let new =
                super::asset_svg(&name, w, h).unwrap_or_else(|| panic!("{name} did not rasterize"));
            assert_eq!(
                (old.width(), old.height()),
                (new.width(), new.height()),
                "{name}: size"
            );
            let (_, _, a) = composited_on_white(&old);
            let (_, _, b) = composited_on_white(&new);
            let diff = a
                .iter()
                .zip(&b)
                .map(|(x, y)| (*x as i32 - *y as i32).abs() as f64)
                .sum::<f64>()
                / a.len() as f64;
            eprintln!("{name:<26} mean diff {diff:.3}");
            worst = worst.max(diff);
        }
        assert!(
            worst < 4.0,
            "worst mean difference {worst:.3} (0-255 per channel)"
        );
    }
}
