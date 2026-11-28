use requiem_core::channel_lookup;
use requiem_core::config;
use requiem_core::gateway::guild_subscriptions;
use requiem_core::images;
use requiem_core::models;
use requiem_core::plugins;
use requiem_core::state::FriendsTab;
use requiem_core::ui::Destination;
use requiem_core::ui::ProfileTab;
use requiem_core::ui::account_popout;
use requiem_core::ui::attachments;
use requiem_core::ui::call_view;
use requiem_core::ui::channel_sidebar;
use requiem_core::ui::classes::message_list;
use requiem_core::ui::context_menu;
use requiem_core::ui::context_menus;
use requiem_core::ui::create_channel;
use requiem_core::ui::create_guild;
use requiem_core::ui::custom_status;
use requiem_core::ui::devtools;
use requiem_core::ui::dm_profile;
use requiem_core::ui::emoji_popout;
use requiem_core::ui::full_profile;
use requiem_core::ui::guild_rail;
use requiem_core::ui::guild_settings as ui_guild_settings;
use requiem_core::ui::home;
use requiem_core::ui::inbox;
use requiem_core::ui::invite_modal;
use requiem_core::ui::loading_screen;
use requiem_core::ui::media_viewer;
use requiem_core::ui::member_list;
use requiem_core::ui::message_confirm;
use requiem_core::ui::forward;
use requiem_core::ui::message_list as ui_message_list;
use requiem_core::ui::message_list::ChatColors;
use requiem_core::ui::message_list::header_icon_button;
use requiem_core::ui::modal_stack::register;
use requiem_core::ui::modal_stack;
use requiem_core::ui::navigation;
use requiem_core::ui::new_message;
use requiem_core::ui::notification_settings;
use requiem_core::ui::pins;
use requiem_core::ui::playground;
use requiem_core::ui::plugin_install;
use requiem_core::ui::plugin_modal;
use requiem_core::ui::plugin_view;
use requiem_core::ui::profile_popout;
use requiem_core::ui::quick_switcher;
use requiem_core::ui::read_state;
use requiem_core::ui::role_popout;
use requiem_core::ui::search;
use requiem_core::ui::search_filters;
use requiem_core::ui::shortcuts_dialog;
use requiem_core::ui::sidebar_resize;
use requiem_core::ui::thread_browser;
use requiem_core::ui::threads;
use requiem_core::ui::user_panel;
use requiem_core::ui::video_player;
use requiem_core::ui;
use requiem_core::utils::prefs;
use requiem_ui::components::{div, toast};
use requiem_ui::signal;
use requiem_ui::timer;
use requiem_ui::widget::anim::ToggleState;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use requiem_core::state::{AppState, DmChannel};
use requiem_ui::node::Node;
use requiem_core::ui::{
    RootContext, RootSignals, borrow_images, borrow_queries, detach_only_child,
    resolve_selected_guild,
};
use requiem_ui::control::section;
use requiem_ui::icons::{
    CLYDE, FRIENDS, GROUP, GROUP_PLUS, PHONE_CALL, PIN, USER_CIRCLE_STATUS, VIDEO,
};
use requiem_ui::style::ComputedStyle;
use requiem_ui::style::cascade;
use requiem_ui::style::parser;
use requiem_ui::widget::WidgetId;
use requiem_ui::widget::WidgetTree;
use requiem_ui::widget::primitives::BoxWidget;

use crate::features::guild_settings;
use crate::features::title_bar;
use crate::features::user_settings::{profile_dialogs, profile_settings, settings_modal};
use crate::features::voice::voice_settings;

fn own_avatar_pending(
    state: &AppState,
    images: &images::ImageCache,
    backend: &config::BackendConfig,
) -> bool {
    let Some(user) = state.users.current.get() else {
        return false;
    };
    user.avatar.as_deref().is_some_and(|hash| {
        images
            .get_or_decode_image(&backend.avatar_url(user.id, hash))
            .is_none()
    })
}

static NATIVE_DIALOG_CLOSED_AT: Mutex<Option<Instant>> =
    Mutex::new(None);

pub fn note_native_dialog_closed() {
    *NATIVE_DIALOG_CLOSED_AT
        .lock()
        .expect("dialog mutex poisoned") = Some(Instant::now());
}

pub fn native_dialog_just_closed() -> bool {
    NATIVE_DIALOG_CLOSED_AT
        .lock()
        .expect("dialog mutex poisoned")
        .is_some_and(|at| at.elapsed() < Duration::from_millis(600))
}

const SETTINGS_MODAL_CLOSE_GRACE: Duration = Duration::from_millis(350);

const THEME_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/bundled.css"));

pub mod classes {
    include!(concat!(env!("OUT_DIR"), "/css_classes.rs"));
}

pub fn load_theme() -> Vec<parser::StyleRule> {
    requiem_ui::set_theme_name(match prefs::theme().as_str() {
        "light" => "light",
        "ash" => "ash",
        "onyx" => "onyx",
        _ => "dark",
    });
    let plugins = plugins::host()
        .map(|host| host.stylesheet())
        .unwrap_or_default();
    let combined = format!(
        "{}\n{}\n{}\n{}",
        requiem_ui::BUNDLED_CSS,
        ui::THEME_CSS,
        THEME_CSS,
        plugins
    );
    parser::parse_stylesheet(&combined)
}

fn home_env(
    state: &AppState,
    ctx: &RootContext,
    signals: &RootSignals,
) -> home::HomeEnv {
    home::HomeEnv {
        state: state.clone(),
        fonts: ctx.fonts.clone(),
        images: borrow_images(&ctx.images).clone(),
        backend: ctx.backend.clone(),
        queries: borrow_queries(&ctx.queries).clone(),
        http: ctx.http.clone(),
        tick: signals.global_tick.clone(),
        on_select_channel: signals.on_select_channel.clone(),
        on_open_friends: open_home(state, signals),
    }
}

pub fn open_home(state: &AppState, signals: &RootSignals) -> Rc<dyn Fn()> {
    let (home, selected) = (state.nav.home.clone(), signals.selected_channel.clone());
    Rc::new(move || {
        navigation::note_home();
        home.set(true);
        selected.set(None);
    })
}

/// What every section of the root layout is built from.
struct MountEnv<'a> {
    state: &'a AppState,
    ctx: &'a RootContext,
    signals: &'a RootSignals,
}

const SETTLE_WINDOW: Duration = Duration::from_millis(250);

fn settled<T: Clone + PartialEq + 'static>(
    tree: &mut WidgetTree,
    tick: signal::Signal<u64>,
    probe: impl Fn() -> (T, bool) + 'static,
) -> (signal::Signal<T>, signal::Effect) {
    let probe = Rc::new(probe);
    let current = signal::Signal::new(probe().0);
    let write = current.clone();
    let last_write = Rc::new(Cell::new(Instant::now()));
    let trailing = Rc::new(Cell::new(false));
    let effect = signal::create_effect(
        move |_| {
            let (value, loading) = probe();
            if loading {
                let _ = tick.get();
            }
            if write.peek() == value {
                return;
            }
            let since = last_write.get().elapsed();
            if !loading || since >= SETTLE_WINDOW {
                last_write.set(Instant::now());
                write.set(value);
            } else if !trailing.replace(true) {
                let (probe, write, last_write, trailing) =
                    (probe.clone(), write.clone(), last_write.clone(), trailing.clone());
                timer::set_timeout(SETTLE_WINDOW - since, move || {
                    trailing.set(false);
                    let (value, _) = signal::untrack(|| probe());
                    if write.peek() != value {
                        last_write.set(Instant::now());
                        write.set(value);
                    }
                });
            }
        },
        tree,
    );
    (current, effect)
}

fn mount_guild_rail(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let guild_rail_state = state.clone();
    let guild_rail_ctx = ctx.clone();
    let guild_rail_signals = signals.clone();
    section(tree, parent, move || {
        let selected_channel = guild_rail_signals.selected_channel.get();
        let selected_guild_id = resolve_selected_guild(&guild_rail_state, selected_channel);
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        section_tree.defer_mount_hooks = true;
        let inner_root = section_tree.root;
        guild_rail::build(
            &mut section_tree,
            inner_root,
            &guild_rail_state,
            selected_guild_id,
            &guild_rail_ctx.fonts,
            &borrow_images(&guild_rail_ctx.images),
            &guild_rail_ctx.backend,
            &guild_rail_signals.on_select_channel,
            &open_home(&guild_rail_state, &guild_rail_signals),
            &guild_rail_signals.global_tick,
        );
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_channel_sidebar(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let channel_sidebar_state = state.clone();
    let channel_sidebar_ctx = ctx.clone();
    let channel_sidebar_signals = signals.clone();
    section(tree, parent, move || {
        let selected_channel_id = channel_sidebar_signals.selected_channel.get();
        let selected_guild_id = resolve_selected_guild(&channel_sidebar_state, selected_channel_id);

        if selected_guild_id.is_none() {
            return home::sidebar(
                &home_env(
                    &channel_sidebar_state,
                    &channel_sidebar_ctx,
                    &channel_sidebar_signals,
                ),
                selected_channel_id,
            );
        }
        let channel =
            ui::displayed_channel(&channel_sidebar_state, selected_channel_id);
        let displayed_channel_id = channel.as_ref().map(|c| c.id);
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        channel_sidebar::build(
            &mut section_tree,
            inner_root,
            &channel_sidebar_state,
            selected_guild_id,
            displayed_channel_id,
            &channel_sidebar_signals.on_select_channel,
            &channel_sidebar::SidebarEnv {
                images: borrow_images(&channel_sidebar_ctx.images).clone(),
                backend: channel_sidebar_ctx.backend.clone(),
                queries: borrow_queries(&channel_sidebar_ctx.queries).clone(),
                http: channel_sidebar_ctx.http.clone(),
                tick: channel_sidebar_signals.global_tick.clone(),
                fonts: channel_sidebar_ctx.fonts.clone(),
            },
        );

        detach_only_child(section_tree, inner_root)
            .bind_style(sidebar_resize::style())
    })
}

fn mount_user_panel(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let user_panel_state = state.clone();
    let user_panel_ctx = ctx.clone();
    let user_panel_signals = signals.clone();
    let (avatar, avatar_effect) = settled(tree, user_panel_signals.global_tick.clone(), {
        let (state, ctx) = (user_panel_state.clone(), user_panel_ctx.clone());
        move || {
            let pending = own_avatar_pending(&state, &borrow_images(&ctx.images), &ctx.backend);
            (!pending, pending)
        }
    });
    section(tree, parent, move || {
        let _ = &avatar_effect;
        let _ = avatar.get();
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        user_panel::build(
            &mut section_tree,
            inner_root,
            &user_panel_state,
            &user_panel_ctx.fonts,
            &borrow_images(&user_panel_ctx.images),
            &user_panel_ctx.backend,
            user_panel_signals.mute_toggle.get(),
            &user_panel_signals.on_toggle_mute,
            user_panel_signals.deafen_toggle.get(),
            &user_panel_signals.on_toggle_deafen,
            &user_panel_signals.on_open_settings,
            &user_panel_signals.account_popout,
            &user_panel_signals.on_select_channel,
        );
        detach_only_child(section_tree, inner_root)
            .bind_style(sidebar_resize::panel_style())
    })
}

fn mount_title_bar(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let title_bar_state = state.clone();
    let title_bar_ctx = ctx.clone();
    let title_bar_signals = signals.clone();
    section(tree, parent, move || {
        let selected_channel_id = title_bar_signals.selected_channel.get();
        let selected_guild_id = resolve_selected_guild(&title_bar_state, selected_channel_id);
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        let home_title = match ui::destination(&title_bar_state, selected_channel_id)
        {
            Destination::Friends => {
                Some((requiem_i18n::t!("ui-friends"), FRIENDS))
            }
            Destination::Dm(_) => {
                Some((requiem_i18n::t!("ui-direct-messages"), CLYDE))
            }
            Destination::Guild => None,
        };
        title_bar::build(
            &mut section_tree,
            inner_root,
            &title_bar_state,
            &title_bar_ctx.fonts,
            &borrow_images(&title_bar_ctx.images),
            &title_bar_ctx.backend,
            selected_guild_id,
            home_title,
            &title_bar_signals.on_select_channel,
            &devtools::DevtoolsContext {
                fonts: title_bar_ctx.fonts.clone(),
                state: title_bar_state.clone(),
            },
        );
        detach_only_child(section_tree, inner_root)
    })
}

/// The pins, thread and member-list buttons of a server channel's header.
fn guild_header_buttons(
    mut toolbar: Node,
    channel: Option<&models::Channel>,
    signals: &RootSignals,
) -> Node {
    let header_signals = signals;
    let pins_button = channel.map(|c| c.id).map(|channel_id| {
        pins::anchor(header_icon_button(
            PIN,
            requiem_i18n::t!("pins-title").to_string(),
            pins::is_open_for(channel_id),
            move || pins::toggle(channel_id),
        ))
    });
    if let Some(pins_button) =
        pins_button.filter(|_| channel.and_then(|c| c.guild_id).is_some())
    {
        toolbar = toolbar.child(pins_button);
    }
    if let Some(channel) = channel
        .as_ref()
        .filter(|channel| channel.guild_id.is_some())
    {
        if let Some(button) = thread_browser::header_button(channel) {
            toolbar = toolbar.child(button);
        }
    }
    if channel.and_then(|c| c.guild_id).is_some() {
        let open = header_signals.member_list.open.clone();
        let showing = open.get();
        let label = if showing {
            requiem_i18n::t!("member-list-hide")
        } else {
            requiem_i18n::t!("member-list-show")
        };
        toolbar = toolbar.child(header_icon_button(
            GROUP,
            label.to_string(),
            showing,
            move || {
                let show = !open.peek();
                open.set(show);

                guild_subscriptions::set_members_visible(show);
            },
        ));
    }
    toolbar
}

/// The call, pins, add-friends and profile buttons of a direct message's header.
fn dm_header_buttons(mut toolbar: Node, dm: &DmChannel, signals: &RootSignals) -> Node {
    let header_signals = signals;
        toolbar = toolbar.children([
            header_icon_button(
                PHONE_CALL,
                requiem_i18n::t!("home-start-voice-call").to_string(),
                false,
                || {},
            ),
            header_icon_button(
                VIDEO,
                requiem_i18n::t!("home-start-video-call").to_string(),
                false,
                || {},
            ),
            pins::anchor(header_icon_button(
                PIN,
                requiem_i18n::t!("dm-header-pinned-messages").to_string(),
                pins::is_open_for(dm.id),
                {
                    let channel_id = dm.id;
                    move || pins::toggle(channel_id)
                },
            )),
            header_icon_button(
                GROUP_PLUS,
                requiem_i18n::t!("dm-header-add-friends-to-dm").to_string(),
                false,
                || {},
            ),
        ]);

        let open = header_signals.member_list.dm_profile_open.clone();
        let showing = open.get();
        let (glyph, label) = match (dm.is_group(), showing) {
            (false, true) => (
                USER_CIRCLE_STATUS,
                requiem_i18n::t!("dm-header-hide-user-profile"),
            ),
            (false, false) => (
                USER_CIRCLE_STATUS,
                requiem_i18n::t!("dm-header-show-user-profile"),
            ),
            (true, true) => (GROUP, requiem_i18n::t!("member-list-hide")),
            (true, false) => (GROUP, requiem_i18n::t!("member-list-show")),
        };
        toolbar = toolbar.child(header_icon_button(
            glyph,
            label.to_string(),
            showing,
            move || open.set(!open.peek()),
        ));
    
    toolbar
}

fn mount_chat_header(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let header_state = state.clone();
    let header_ctx = ctx.clone();
    let header_signals = signals.clone();
    section(tree, parent, move || {
        let selected_channel_id = header_signals.selected_channel.get();
        let channel = ui::displayed_channel(&header_state, selected_channel_id);

        if matches!(
            ui::destination(&header_state, selected_channel_id),
            Destination::Friends
        ) {
            return div();
        }
        let mut toolbar = div().class(message_list::CHANNEL_HEADER_TOOLBAR);
        toolbar = guild_header_buttons(toolbar, channel.as_ref(), &header_signals);
        if let Destination::Dm(dm) = ui::destination(&header_state, selected_channel_id) {
            toolbar = dm_header_buttons(toolbar, &dm, &header_signals);
        }
        if channel.is_some() {
            toolbar = toolbar.child(search::build_bar(
                &header_state,
                channel.as_ref(),
                &header_signals.search,
                &header_ctx.fonts,
                &borrow_images(&header_ctx.images),
                &header_ctx.backend,
            ));
        }
        let toolbar = Some(toolbar);
        if let Destination::Dm(dm) =
            ui::destination(&header_state, selected_channel_id)
        {
            return home::dm_header(
                &home_env(&header_state, &header_ctx, &header_signals),
                &dm,
                toolbar,
            );
        }
        ui_message_list::build_channel_header(
            &header_state,
            channel.as_ref(),
            &ChatColors::desktop(),
            toolbar,
        )
    })
}

fn mount_message_list(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let message_list_state = state.clone();
    let message_list_ctx = ctx.clone();
    let message_list_signals = signals.clone();
    section(tree, parent, move || {
        let selected_channel_id = message_list_signals.selected_channel.get();
        let channel = ui::displayed_channel(&message_list_state, selected_channel_id);
        if matches!(
            ui::destination(&message_list_state, selected_channel_id),
            Destination::Friends
        ) {
            return home::friends_page(&home_env(
                &message_list_state,
                &message_list_ctx,
                &message_list_signals,
            ));
        }

        if let Some((channel, guild_id)) = channel
            .as_ref()
            .filter(|c| matches!(c.kind, 2 | 13))
            .and_then(|c| Some((c, c.guild_id?)))
        {
            let env = call_view::CallEnv {
                images: borrow_images(&message_list_ctx.images).clone(),
                backend: message_list_ctx.backend.clone(),
                fonts: message_list_ctx.fonts.clone(),
                tick: message_list_signals.global_tick.clone(),
                muted: message_list_signals.mute_toggle.get().on,
                on_toggle_mute: message_list_signals.on_toggle_mute.clone(),
                menu: context_menus::MenuEnv {
                    backend: message_list_ctx.backend.clone(),
                    queries: borrow_queries(&message_list_ctx.queries).clone(),
                    http: Some(message_list_ctx.http.clone()),
                    images: borrow_images(&message_list_ctx.images).clone(),
                    on_save: None,
                    own_user_id: message_list_state
                        .users
                        .current
                        .with_untracked(|u| u.as_ref().map(|u| u.id)),
                    state: message_list_state.clone(),
                },
            };
            return call_view::build(
                &message_list_state,
                guild_id,
                channel,
                &env,
            );
        }
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        section_tree.defer_mount_hooks = true;
        let inner_root = section_tree.root;
        ui_message_list::build(
            &mut section_tree,
            inner_root,
            &message_list_state,
            channel.as_ref(),
            &message_list_ctx.fonts,
            &borrow_images(&message_list_ctx.images),
            &message_list_ctx.backend,
            &message_list_ctx.compose_tx,
            &message_list_signals.composer_focused,
            &message_list_signals.composer_has_text,
            &message_list_signals.composer_send_toggle,
            &message_list_signals.message_scroll,
            &message_list_ctx.measure_cache,
            &ChatColors::desktop(),
            &message_list_signals.emoji_picker,
            &message_list_signals.attachments,
            &message_list_signals.media_viewer,
            &message_list_signals.global_tick,
            &message_list_signals.profile_popout,
            &borrow_queries(&message_list_ctx.queries),
            Some(&message_list_ctx.http),
            Some(ui::message_embed::VideoContext {
                videos: &message_list_ctx.videos,
                fullscreen: &message_list_signals.fullscreen_video,
            }),
        );
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_member_list(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let member_list_state = state.clone();
    let member_list_ctx = ctx.clone();
    let member_list_signals = signals.clone();
    section(tree, parent, move || {
        let selected_channel_id = member_list_signals.selected_channel.get();
        let channel = ui::displayed_channel(&member_list_state, selected_channel_id);

        if let (Some(channel), Some(submitted)) =
            (channel.as_ref(), member_list_signals.search.submitted.get())
        {
            if submitted.scope == search::scope_for(channel) {
                let images = borrow_images(&member_list_ctx.images);
                let queries = borrow_queries(&member_list_ctx.queries);
                return search::build_results(
                    &search::ResultsEnv {
                        state: &member_list_state,
                        search: &member_list_signals.search,
                        fonts: &member_list_ctx.fonts,
                        images: &images,
                        backend: &member_list_ctx.backend,
                        queries: &queries,
                        http: Some(&member_list_ctx.http),
                        media_viewer: &member_list_signals.media_viewer,
                        tick: &member_list_signals.global_tick,
                        video_ctx: Some(ui::message_embed::VideoContext {
                            videos: &member_list_ctx.videos,
                            fullscreen: &member_list_signals.fullscreen_video,
                        }),
                    },
                );
            }
        }
        if let Destination::Dm(dm) =
            ui::destination(&member_list_state, selected_channel_id)
        {
            if member_list_signals.member_list.dm_profile_open.get() {
                let _ = member_list_signals.global_tick.get();
                return dm_profile::build(
                    &member_list_state,
                    &dm,
                    &member_list_signals.member_list,
                    &member_list_signals.profile_popout,
                    &member_list_ctx.fonts,
                    &borrow_images(&member_list_ctx.images),
                    &member_list_ctx.backend,
                    &borrow_queries(&member_list_ctx.queries),
                    Some(&member_list_ctx.http),
                );
            }
        }
        if let Some(sidebar) = threads::sidebar() {
            return sidebar;
        }

        if channel.as_ref().is_some_and(|c| matches!(c.kind, 2 | 13)) {
            return div();
        }
        match channel.and_then(|c| c.guild_id) {
            Some(guild_id) if member_list_signals.member_list.open.get() => {
                member_list::build(
                    &member_list_state,
                    guild_id,
                    &member_list_signals.member_list,
                    &member_list_ctx.fonts,
                    &borrow_images(&member_list_ctx.images),
                    &member_list_ctx.backend,
                    &borrow_queries(&member_list_ctx.queries),
                    Some(&member_list_ctx.http),
                )
            }

            _ => div(),
        }
    })
}

fn mount_account_popout(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let account_popout_state = state.clone();
    let account_popout_ctx = ctx.clone();
    let account_popout_signals = signals.clone();
    section(tree, parent, move || {
        if account_popout_signals.account_popout.open.get() {
            let _ = account_popout_signals.global_tick.get();
        }
        account_popout::build(
            &account_popout_state,
            &account_popout_signals.account_popout,
            &account_popout_ctx.fonts,
            &borrow_images(&account_popout_ctx.images),
            &account_popout_ctx.backend,
            &borrow_queries(&account_popout_ctx.queries),
            Some(&account_popout_ctx.http),
        )
    })
}

fn mount_inbox(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let inbox_state = state.clone();
    let inbox_ctx = ctx.clone();
    let inbox_signals = signals.clone();
    section(tree, parent, move || {
        let images = borrow_images(&inbox_ctx.images);
        let queries = borrow_queries(&inbox_ctx.queries);

        let on_friend_requests: Rc<dyn Fn()> = {
            let (state, selected) = (inbox_state.clone(), inbox_signals.selected_channel.clone());
            Rc::new(move || {
                state
                    .nav
                    .friends_tab
                    .set(FriendsTab::Pending);
                state.nav.home.set(true);
                selected.set(None);
            })
        };
        inbox::build(
            &search::ResultsEnv {
                state: &inbox_state,
                search: &inbox_signals.search,
                fonts: &inbox_ctx.fonts,
                images: &images,
                backend: &inbox_ctx.backend,
                queries: &queries,
                http: Some(&inbox_ctx.http),
                media_viewer: &inbox_signals.media_viewer,
                tick: &inbox_signals.global_tick,
                video_ctx: Some(ui::message_embed::VideoContext {
                    videos: &inbox_ctx.videos,
                    fullscreen: &inbox_signals.fullscreen_video,
                }),
            },
            &on_friend_requests,
        )
    })
}

fn mount_pins(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let pins_state = state.clone();
    let pins_ctx = ctx.clone();
    let pins_signals = signals.clone();
    section(tree, parent, move || {
        let images = borrow_images(&pins_ctx.images);
        let queries = borrow_queries(&pins_ctx.queries);

        if pins::is_open() {
            let _ = pins_signals.global_tick.get();
        }
        pins::build(&search::ResultsEnv {
            state: &pins_state,
            search: &pins_signals.search,
            fonts: &pins_ctx.fonts,
            images: &images,
            backend: &pins_ctx.backend,
            queries: &queries,
            http: Some(&pins_ctx.http),
            media_viewer: &pins_signals.media_viewer,
            tick: &pins_signals.global_tick,
            video_ctx: Some(ui::message_embed::VideoContext {
                videos: &pins_ctx.videos,
                fullscreen: &pins_signals.fullscreen_video,
            }),
        })
    })
}

fn mount_emoji_popout(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let emoji_state = state.clone();
    let emoji_ctx = ctx.clone();
    let emoji_signals = signals.clone();
    section(tree, parent, move || {
        let images = borrow_images(&emoji_ctx.images);
        emoji_popout::build(
            &emoji_state,
            &images,
            &emoji_ctx.backend,
            &emoji_signals.global_tick,
            &emoji_signals.on_select_channel,
        )
    })
}

fn mount_role_popout(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let ctx = env.ctx;
    let role_ctx = ctx.clone();
    section(tree, parent, move || {
        let images = borrow_images(&role_ctx.images);
        role_popout::build(&images, &role_ctx.backend)
    })
}

fn mount_member_popout(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let member_popout_state = state.clone();
    let member_popout_ctx = ctx.clone();
    let member_popout_signals = signals.clone();
    section(tree, parent, move || {
        if member_popout_signals
            .profile_popout
            .overlays
            .member_target
            .get()
            .is_some()
        {
            let _ = member_popout_signals.global_tick.get();
            let _ = member_popout_signals
                .profile_popout
                .window_resize_tick
                .get();
        }
        profile_popout::build_member_layer(
            &member_popout_state,
            &member_popout_signals.profile_popout,
            &member_popout_ctx.fonts,
            &borrow_images(&member_popout_ctx.images),
            &member_popout_ctx.backend,
            &borrow_queries(&member_popout_ctx.queries),
            Some(&member_popout_ctx.http),
        )
    })
}

fn mount_plugin_modal(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let surface_signals = signals.clone();
    let surface_ctx = ctx.clone();
    let surface_state = state.clone();
    section(tree, parent, move || {
        let surface = plugin_modal::current(&surface_state);
        if surface.is_some() {
            let _ = surface_signals.global_tick.get();
        }
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        let surface = surface.map(|mut s| {
            s.open = modal_stack::effective("plugin", s.open);
            s
        });
        if let Some(surface) = surface
            .filter(|s| s.open.on || s.open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE)
        {
            let env = plugin_view::ViewEnv {
                images: borrow_images(&surface_ctx.images).clone(),
                tick: surface_signals.global_tick.clone(),
                fonts: surface_ctx.fonts.clone(),
                viewer: Some(surface_signals.media_viewer.clone()),
            };
            let id = section_tree.mount(
                inner_root,
                plugin_modal::build(&surface, &surface_state, &env),
            );
            if surface.open.on {
                section_tree.focus_scope = Some(id);
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_full_profile(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let full_profile_state = state.clone();
    let full_profile_ctx = ctx.clone();
    let full_profile_signals = signals.clone();
    section(tree, parent, move || {
        let overlays = &full_profile_signals.profile_popout.overlays;
        let open = modal_stack::effective(
            "full-profile",
            overlays.full_profile_ui.open.get(),
        );
        let closing = !open.on && open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE;
        let target = overlays.full_profile.get();
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if let Some(target) = target.filter(|_| open.on || closing) {
            let _ = full_profile_signals.global_tick.get();
            let node = full_profile::build(
                target,
                open,
                &full_profile_state,
                overlays,
                &full_profile_ctx.fonts,
                &borrow_images(&full_profile_ctx.images),
                &full_profile_ctx.backend,
                &borrow_queries(&full_profile_ctx.queries),
                Some(&full_profile_ctx.http),
                Some(&full_profile_signals.media_viewer),
            );
            let id = section_tree.mount(inner_root, node);
            if open.on {
                section_tree.focus_scope = Some(id);
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_block_confirm(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let ctx = env.ctx;
    let signals = env.signals;
    let block_ctx = ctx.clone();
    let block_signals = signals.clone();
    section(tree, parent, move || {
        let overlays = &block_signals.profile_popout.overlays;
        let open = modal_stack::effective("block", overlays.block_open.get());
        let closing = !open.on && open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE;
        let target = overlays.block_confirm.get();
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if let Some((user_id, name, location)) = target.filter(|_| open.on || closing) {
            let _ = block_signals.global_tick.get();
            let node = full_profile::build_block_modal(
                user_id,
                &name,
                location,
                open,
                overlays,
                &block_ctx.fonts,
                &borrow_images(&block_ctx.images),
                &block_ctx.backend,
                &borrow_queries(&block_ctx.queries),
                Some(&block_ctx.http),
            );
            let id = section_tree.mount(inner_root, node);
            if open.on {
                section_tree.focus_scope = Some(id);
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_drop_overlay(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let signals = env.signals;
    let drop_overlay_state = state.clone();
    let drop_overlay_signals = signals.clone();
    section(tree, parent, move || {
        let channel = ui::displayed_channel(
            &drop_overlay_state,
            drop_overlay_signals.selected_channel.get(),
        );
        attachments::build_drop_overlay(
            &drop_overlay_signals.attachments,
            channel.and_then(|c| c.name),
        )
    })
}

fn mount_modify_attachment(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let signals = env.signals;
    let modify_signals = signals.clone();
    section(tree, parent, move || {
        let attachments = &modify_signals.attachments;
        let open = modal_stack::effective(
            "modify-attachment",
            attachments.modify.open.get(),
        );
        let _ = attachments.modify.target.get();
        let _ = attachments.modify.sent.get();
        let closing = !open.on && open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE;
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if open.on || closing {
            if let Some(node) = attachments::build_modify_modal(attachments, open)
            {
                let id = section_tree.mount(inner_root, node);
                if open.on {
                    section_tree.focus_scope = Some(id);
                }
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_too_many_attachments(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let signals = env.signals;
    let too_many_signals = signals.clone();
    section(tree, parent, move || {
        let attachments = &too_many_signals.attachments;
        let open =
            modal_stack::effective("too-many", attachments.too_many_open.get());
        let closing = !open.on && open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE;
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if open.on || closing {
            let id = section_tree.mount(
                inner_root,
                attachments::build_too_many_modal(attachments, open),
            );
            if open.on {
                section_tree.focus_scope = Some(id);
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_search_filters(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let signals = env.signals;
    let filters_state = state.clone();
    let filters_signals = signals.clone();
    section(tree, parent, move || {
        let open = modal_stack::effective(
            "filters",
            filters_signals.search.filters_open.get(),
        );
        let closing = !open.on && open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE;
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if open.on || closing {
            if let Some(channel) = ui::displayed_channel(
                &filters_state,
                filters_signals.selected_channel.get(),
            ) {
                let choices = search_filters::choices(&filters_state, &channel);
                let (search, scope) = (
                    filters_signals.search.clone(),
                    search::scope_for(&channel),
                );
                let id = section_tree.mount(
                    inner_root,
                    search_filters::build_filters_modal(
                        &filters_signals.search,
                        &choices,
                        open,
                        move || search_filters::apply(&search, scope),
                    ),
                );
                if open.on {
                    section_tree.focus_scope = Some(id);
                }
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_settings_modal(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let settings_modal_state = state.clone();
    let settings_modal_ctx = ctx.clone();
    let settings_modal_signals = signals.clone();
    section(tree, parent, move || {
        let settings_modal_open = modal_stack::effective(
            "settings",
            settings_modal_signals.settings_modal_open.get(),
        );

        let settings_modal_closing = !settings_modal_open.on
            && settings_modal_open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE;

        if settings_modal_open.on {
            if own_avatar_pending(
                &settings_modal_state,
                &borrow_images(&settings_modal_ctx.images),
                &settings_modal_ctx.backend,
            ) {
                let _ = settings_modal_signals.global_tick.get();
            }
            let settings = settings_modal_signals.settings_ui.get();
            if settings.category == profile_settings::PROFILES_CATEGORY {
                let draft = settings.profile_draft.peek();
                let waiting_main = !draft.seeded;
                let waiting_server = settings.profile_tab
                    == ProfileTab::PerServer
                    && draft.server.guild_id.is_some()
                    && !draft.server.seeded;
                if waiting_main || waiting_server {
                    let _ = settings_modal_signals.global_tick.get();
                }
            }
        }

        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if settings_modal_open.on || settings_modal_closing {
            let on_close_settings = settings_modal_signals.on_close_settings.clone();
            let modal_node = settings_modal::build(
                &settings_modal_state,
                &settings_modal_ctx.fonts,
                &borrow_images(&settings_modal_ctx.images),
                &settings_modal_ctx.backend,
                settings_modal_open,
                Rc::new(move || on_close_settings()),
                &settings_modal_signals.settings_ui,
                &borrow_queries(&settings_modal_ctx.queries),
                Some(&settings_modal_ctx.http),
                &settings_modal_signals.account_popout,
                &settings_modal_signals.emoji_picker,
            );
            let modal_id = section_tree.mount(inner_root, modal_node);

            if settings_modal_open.on {
                section_tree.focus_scope = Some(modal_id);
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_guild_settings(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let gs_state = state.clone();
    let gs_ctx = ctx.clone();
    let gs_signals = signals.clone();
    section(tree, parent, move || {
        let gs = ui_guild_settings::state();
        let open = gs.open.get();
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;

        if open.on {
            let env = guild_settings::Env {
                state: gs_state.clone(),
                fonts: gs_ctx.fonts.clone(),
                images: borrow_images(&gs_ctx.images).clone(),
                backend: gs_ctx.backend.clone(),
                queries: borrow_queries(&gs_ctx.queries).clone(),
                http: gs_ctx.http.clone(),
                tick: gs_signals.global_tick.clone(),
            };
            let node =
                guild_settings::build(&env, open, Rc::new(ui_guild_settings::close));
            let id = section_tree.mount(inner_root, node);
            if open.on {
                section_tree.focus_scope = Some(id);
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_guild_settings_dialogs(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let gsd_state = state.clone();
    let gsd_ctx = ctx.clone();
    let gsd_signals = signals.clone();
    section(tree, parent, move || {
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        let env = guild_settings::Env {
            state: gsd_state.clone(),
            fonts: gsd_ctx.fonts.clone(),
            images: borrow_images(&gsd_ctx.images).clone(),
            backend: gsd_ctx.backend.clone(),
            queries: borrow_queries(&gsd_ctx.queries).clone(),
            http: gsd_ctx.http.clone(),
            tick: gsd_signals.global_tick.clone(),
        };
        if let Some(node) = guild_settings::open_dialogs(&env) {
            let id = section_tree.mount(inner_root, node);
            section_tree.focus_scope = Some(id);
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_logout_confirm(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let signals = env.signals;
    let logout_signals = signals.clone();
    section(tree, parent, move || {
        let open = modal_stack::effective(
            "logout",
            logout_signals.settings_ui.get().logout_confirm,
        );
        let closing = !open.on && open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE;
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if open.on || closing {
            let node = settings_modal::build_logout_confirm(
                open,
                &logout_signals.settings_ui,
                logout_signals.on_log_out.clone(),
            );
            let id = section_tree.mount(inner_root, node);
            if open.on {
                section_tree.focus_scope = Some(id);
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_profile_dialogs(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let dialog_signals = signals.clone();
    let dialog_ctx = ctx.clone();
    let dialog_state = state.clone();
    section(tree, parent, move || {
        let dialog_signal = dialog_signals.settings_ui.peek().profile_dialog;
        let mut dialog = dialog_signal.get();
        dialog.open = modal_stack::effective("profile-dialog", dialog.open);
        let closing =
            !dialog.open.on && dialog.open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE;
        if dialog.open.on {
            let _ = dialog_signals.global_tick.get();
        }
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if dialog.open.on || closing {
            let node = profile_dialogs::build(
                &dialog,
                &dialog_signal,
                &dialog_signals.settings_ui,
                &dialog_state,
                &dialog_ctx.fonts,
                &borrow_queries(&dialog_ctx.queries),
                Some(&dialog_ctx.http),
            );
            if let Some(node) = node {
                let id = section_tree.mount(inner_root, node);
                if dialog.open.on {
                    section_tree.focus_scope = Some(id);
                }
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_plugin_install(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let signals = env.signals;
    let install_signals = signals.clone();
    section(tree, parent, move || {
        let install_signal = install_signals.settings_ui.peek().plugin_install;
        let mut install = install_signal.get();
        install.open = modal_stack::effective("plugin-install", install.open);
        let closing =
            !install.open.on && install.open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE;
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if install.open.on || closing {
            if let Some(node) = plugin_install::build(&install, &install_signal) {
                let id = section_tree.mount(inner_root, node);
                if install.open.on {
                    section_tree.focus_scope = Some(id);
                }
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_custom_status(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let state = env.state;
    let ctx = env.ctx;
    let signals = env.signals;
    let custom_status_state = state.clone();
    let custom_status_ctx = ctx.clone();
    let custom_status_signals = signals.clone();
    section(tree, parent, move || {
        let modal = &custom_status_signals.account_popout.custom_status;
        let open = modal_stack::effective("custom-status", modal.open.get());
        let closing = !open.on && open.changed_at.elapsed() < SETTINGS_MODAL_CLOSE_GRACE;
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if open.on || closing {
            let _ = custom_status_signals.global_tick.get();
            if let Some(user) = custom_status_state.users.current.get() {
                let queries = borrow_queries(&custom_status_ctx.queries);
                let profile = queries.peek::<models::UserProfile>(&format!(
                    "user-profile:{}",
                    user.id
                ));
                let node = custom_status::build_modal(
                    modal,
                    open,
                    &user,
                    profile.as_deref(),
                    &custom_status_ctx.fonts,
                    &borrow_images(&custom_status_ctx.images),
                    &custom_status_ctx.backend,
                    custom_status_signals
                        .account_popout
                        .on_set_custom_status
                        .clone(),
                );
                let id = section_tree.mount(inner_root, node);
                if open.on {
                    section_tree.focus_scope = Some(id);
                }
            }
        }
        detach_only_child(section_tree, inner_root)
    })
}

fn mount_fullscreen_video(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: &MountEnv<'_>,
) -> signal::Effect {
    let signals = env.signals;
    let fullscreen_video_signals = signals.clone();
    section(tree, parent, move || {
        let fullscreen_state = fullscreen_video_signals.fullscreen_video.get();
        let mut section_tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let inner_root = section_tree.root;
        if let Some(state) = fullscreen_state {
            let overlay = video_player::fullscreen_overlay_node(
                &state,
                &fullscreen_video_signals.fullscreen_video,
            );
            section_tree.mount(inner_root, overlay);
        }
        detach_only_child(section_tree, inner_root)
    })
}

/// Tells the escape-key stack which modal dialogs are open.
fn register_modal_probes(state: &AppState, signals: &RootSignals) {
    let off = || ToggleState::new(false);
    let plugin_state = state.clone();
    register("plugin", move || {
        plugin_modal::current(&plugin_state)
            .map(|s| s.open)
            .unwrap_or_else(off)
    });
    let s = signals.clone();
    register("full-profile", move || {
        s.profile_popout.overlays.full_profile_ui.open.get()
    });
    let s = signals.clone();
    register("block", move || s.profile_popout.overlays.block_open.get());
    let s = signals.clone();
    register("modify-attachment", move || s.attachments.modify.open.get());
    let s = signals.clone();
    register("too-many", move || s.attachments.too_many_open.get());
    let s = signals.clone();
    register("filters", move || s.search.filters_open.get());
    let s = signals.clone();
    register("settings", move || s.settings_modal_open.get());
    let s = signals.clone();
    register("logout", move || s.settings_ui.peek().logout_confirm);
    let s = signals.clone();
    register("profile-dialog", move || {
        s.settings_ui.peek().profile_dialog.get().open
    });
    let s = signals.clone();
    register("plugin-install", move || {
        s.settings_ui.peek().plugin_install.get().open
    });
    let s = signals.clone();
    register("custom-status", move || {
        s.account_popout.custom_status.open.get()
    });
    register("forward", forward::toggle);
}

/// Mounts the feature overlays that share the quick switcher's view of the home screen.
fn mount_feature_overlays(
    tree: &mut WidgetTree,
    root: WidgetId,
    env: &MountEnv<'_>,
    effects: &mut Vec<signal::Effect>,
) {
    let MountEnv {
        state,
        ctx,
        signals,
    } = *env;
    effects.extend(media_viewer::mount(
        tree,
        root,
        &signals.media_viewer,
        &ctx.fonts,
        &borrow_images(&ctx.images),
        &ctx.backend,
    ));

    effects.extend(playground::mount(tree, root));

    let (switcher_state, switcher_ctx, switcher_signals) =
        (state.clone(), ctx.clone(), signals.clone());
    let home_factory: Rc<dyn Fn() -> home::HomeEnv> =
        Rc::new(move || home_env(&switcher_state, &switcher_ctx, &switcher_signals));
    effects.push(quick_switcher::mount(
        tree,
        root,
        home_factory.clone(),
    ));
    effects.push(new_message::mount(
        tree,
        root,
        home_factory.clone(),
    ));
    effects.push(forward::mount(
        tree,
        root,
        home_factory.clone(),
    ));
    effects.push(shortcuts_dialog::mount(tree, root));
    effects.extend(create_guild::mount(
        tree,
        root,
        home_factory.clone(),
    ));
    effects.extend(create_channel::mount(
        tree,
        root,
        home_factory.clone(),
        signals.emoji_picker.on_insert_at.clone(),
    ));
    message_confirm::install(home_factory.clone());
    channel_lookup::install(home_factory.clone());
    effects.push(notification_settings::mount(
        tree,
        root,
        home_factory.clone(),
    ));
    effects.push(voice_settings::mount(tree, root));
    effects.push(voice_settings::end_on_close(
        signals.settings_modal_open.clone(),
        tree,
    ));
    effects.push(threads::mount(
        tree,
        root,
        home_factory.clone(),
    ));
    effects.push(invite_modal::mount(
        tree,
        root,
        home_factory,
        signals.selected_channel.clone(),
    ));
}

pub fn mount_root(
    tree: &mut WidgetTree,
    state: &AppState,
    ctx: &RootContext,
    signals: &RootSignals,
) -> Vec<signal::Effect> {
    let mut effects: Vec<signal::Effect> = Vec::new();
    let env = MountEnv { state, ctx, signals };

    let root = tree.root;
    tree.nodes[root].classes.push("root".to_string());

    register_modal_probes(state, signals);

    effects.push(read_state::track_displayed_channel(
        tree,
        state,
        signals.selected_channel.clone(),
    ));

    effects.push(mount_guild_rail(tree, root, &env));

    let frame = tree.mount(root, div().class("left-nav-frame"));

    devtools::init_from_env();
    effects.extend(devtools::mount(
        tree,
        root,
        &devtools::DevtoolsContext {
            fonts: ctx.fonts.clone(),
            state: state.clone(),
        },
    ));

    effects.push(mount_channel_sidebar(tree, frame, &env));

    effects.push(mount_user_panel(tree, root, &env));

    effects.push(mount_title_bar(tree, root, &env));

    let chat_frame = tree.mount(
        frame,
        div().class(message_list::CHAT_FRAME),
    );

    effects.push(mount_chat_header(tree, chat_frame, &env));

    let chat_body = tree.mount(
        chat_frame,
        div().class(message_list::CHAT_BODY),
    );

    effects.push(mount_message_list(tree, chat_body, &env));

    effects.push(mount_member_list(tree, chat_body, &env));

    effects.push(mount_account_popout(tree, root, &env));

    effects.push(mount_inbox(tree, root, &env));

    effects.push(mount_pins(tree, root, &env));

    effects.push(mount_emoji_popout(tree, root, &env));

    effects.push(mount_role_popout(tree, root, &env));

    effects.push(mount_member_popout(tree, root, &env));

    effects.push(mount_plugin_modal(tree, root, &env));

    effects.push(mount_full_profile(tree, root, &env));
    effects.push(mount_block_confirm(tree, root, &env));

    effects.push(mount_drop_overlay(tree, root, &env));

    effects.push(mount_modify_attachment(tree, root, &env));
    effects.push(mount_too_many_attachments(tree, root, &env));

    effects.push(mount_search_filters(tree, root, &env));

    effects.push(mount_settings_modal(tree, root, &env));

    effects.push(mount_guild_settings(tree, root, &env));

    effects.push(mount_guild_settings_dialogs(tree, root, &env));

    effects.push(mount_logout_confirm(tree, root, &env));

    effects.push(mount_profile_dialogs(tree, root, &env));

    effects.push(mount_plugin_install(tree, root, &env));

    effects.push(mount_custom_status(tree, root, &env));

    effects.push(mount_fullscreen_video(tree, root, &env));

    mount_feature_overlays(tree, root, &env, &mut effects);

    effects.extend(context_menu::mount(tree, root));
    effects.push(toast::mount(tree, root, &signals.toasts));

    effects.extend(loading_screen::mount(
        tree,
        root,
        &state.connection.ready,
    ));

    cascade::apply_stylesheet(tree, &load_theme());

    effects
}

#[cfg(test)]
mod frame_layout_tests {
    use requiem_ui::app;
    use requiem_ui::font_system::TextFonts;
    use requiem_ui::layout;
    use requiem_ui::paint;
    use requiem_ui::style::cascade as style_cascade;
    use super::load_theme;
    use requiem_ui::components::{TextProps, TextVariant, div};
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::widget::WidgetTree;
    use requiem_ui::widget::primitives::BoxWidget;
    use taffy::{AvailableSpace, Size};

    #[test]
    fn a_long_unbroken_line_in_the_chat_does_not_push_the_devtools_off_screen() {
        TextFonts::install_new(1024 * 1024);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        let long_line = TextProps::styled("word ".repeat(300), TextVariant::TEXT_SM_NORMAL)
            .line_clamp(1_u32)
            .build();
        let chat = div()
            .style("flex-grow: 1; min-width: 0px; display: flex; flex-direction: column")
            .child(div().style("min-width: 0px").child(long_line));
        tree.mount(
            root,
            div().class("left-nav-frame").children([
                div().style("width: 240px; flex-shrink: 0; height: 100px"),
                chat,
            ]),
        );
        let devtools = tree.mount(
            root,
            div().style("width: 300px; flex-shrink: 0; height: 100px"),
        );
        let theme = load_theme();
        style_cascade::apply_stylesheet(&mut tree, &theme);
        app::set_window_size(1000.0, 600.0);
        layout::compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(1000.0),
                height: AvailableSpace::Definite(600.0),
            },
        );
        let rect = paint::all_absolute_rects(&tree)[&devtools];
        assert!(
            rect.x + rect.width <= 1000.5,
            "the devtools panel ends at {}px in a 1000px window",
            rect.x + rect.width
        );
    }
}
