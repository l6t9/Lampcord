use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::sync::OnceLock;

use requiem_tsx::{HostComponents, HostProps, StateQueries, Value};
use requiem_ui::node::Node;
use requiem_ui::signal::{self, Signal, create_effect};
use requiem_ui::widget::primitives::{BoxWidget, ScrollAnchor, ScrollPosition};
use requiem_ui::widget::WidgetId;
use serde::Serialize;
use specta::Type;

use super::*;
use crate::ui_state::id_arg;
use crate::ui_tsx::{self, Tsx, UiAction};

thread_local! {
    static CTX: RefCell<Option<Rc<ListCtx>>> = const { RefCell::new(None) };
    static CACHE: RefCell<Option<(Id<ChannelMarker>, Rc<ChannelCache>)>> = const { RefCell::new(None) };
    static ENTRY_HEIGHTS: RefCell<HashMap<String, f32>> = RefCell::new(HashMap::new());
    static ROWS_TICK: Signal<u64> = signal::signal(0);
    static OLDER_ASKED: Cell<Option<(u64, u64)>> = const { Cell::new(None) };
}

const HISTORY_ESTIMATE: f32 = 160.0;

pub(super) struct ListCtx {
    pub state: AppState,
    pub fonts: Fonts,
    pub images: ImageCache,
    pub backend: BackendConfig,
    pub colors: ChatColors,
    pub profile_popout: ProfilePopoutState,
    pub queries: QueryClient,
    pub attachments: AttachmentsState,
    pub media_viewer: media_viewer::MediaViewerState,
    pub measure_cache: Rc<MeasureCache>,
    pub images_ready: Signal<u64>,
    pub message_scroll: Signal<HashMap<Id<ChannelMarker>, MessageListScroll>>,
    pub http: Option<HttpClient>,
    pub global_tick: Signal<u64>,
    #[cfg(feature = "video")]
    pub videos: Option<(
        crate::video::VideoCache,
        Signal<Option<ui::video_player::FullscreenVideoState>>,
    )>,
}

struct ChannelCache {
    mentions: Rc<HashMap<u64, String>>,
    channels: Rc<HashMap<u64, String>>,
    role_mentions: Rc<HashMap<u64, (String, Option<Color>)>>,
    guild_roles: Vec<models::Role>,
    menu_env: context_menus::MenuEnv,
    emoji_click: primitives::EmojiClickHandler,
    can_add_reactions: bool,
}

pub(super) fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| env::var_os("REQUIEM_TSX_MESSAGES").is_none_or(|value| value != "0"))
}

pub(super) fn install(ctx: ListCtx) {
    CTX.with(|slot| *slot.borrow_mut() = Some(Rc::new(ctx)));
}

fn ctx() -> Option<Rc<ListCtx>> {
    CTX.with(|slot| slot.borrow().clone())
}

pub(super) fn tsx() -> Option<Rc<Tsx>> {
    let ctx = ctx()?;
    ui_tsx::shared(
        &ctx.state,
        &ctx.fonts,
        &ctx.images,
        &ctx.backend,
        &ctx.global_tick,
    )
}

pub(super) fn list_node(channel_id: Id<ChannelMarker>) -> Option<Node> {
    Some(
        tsx()?
            .node("MessageList", vec![("channelId", Value::str(channel_id.to_string()))])
            .class(message_list::MESSAGE_LIST_HOST),
    )
}

pub(super) fn navigation(channel_id: Id<ChannelMarker>) -> impl Fn(&mut WidgetTree, WidgetId) + 'static {
    move |tree, id| {
        let effect = create_effect(
            move |_| {
                let Some(ctx) = ctx() else { return };
                let Some(channel) = ui::displayed_channel(&ctx.state, Some(channel_id)) else {
                    return;
                };
                let snapshot = ctx.state.messages.in_channel(channel.id);
                let messages = visible_messages(&ctx, &snapshot);
                let current_user_id = ctx.state.users.current.get().as_ref().map(|user| user.id.get());
                let mut scroll_state = ctx
                    .message_scroll
                    .with_untracked(|map| map.get(&channel.id).cloned())
                    .unwrap_or_default();
                follow_own_message(
                    &channel,
                    &messages,
                    current_user_id,
                    &ctx.message_scroll,
                    &mut scroll_state,
                );
                resolve_pending_jump(
                    &ctx.state,
                    &channel,
                    &messages,
                    &mut scroll_state,
                    &ctx.message_scroll,
                    &ctx.queries,
                    ctx.http.as_ref(),
                    &ctx.images_ready,
                );
                resolve_pending_present(
                    &ctx.state,
                    &channel,
                    &ctx.message_scroll,
                    &ctx.queries,
                    ctx.http.as_ref(),
                    &ctx.images_ready,
                );
            },
            tree,
        );
        tree.attach_effect(id, effect);
    }
}

fn channel_cache(ctx: &ListCtx, channel: &Channel, refresh: bool) -> Rc<ChannelCache> {
    if !refresh {
        if let Some((id, cache)) = CACHE.with(|slot| slot.borrow().clone()) {
            if id == channel.id {
                return cache;
            }
        }
    }
    let guild_roles = channel
        .guild_id
        .map(|guild_id| author_style::guild_roles(&ctx.state, guild_id))
        .unwrap_or_default();
    let role_mentions = Rc::new(author_style::role_mention_map(&guild_roles));
    let guild_id = channel.guild_id;
    let emoji_click: primitives::EmojiClickHandler =
        Rc::new(move |click| emoji_popout::open(click, guild_id));
    let cache = Rc::new(ChannelCache {
        mentions: Rc::new(mention_names(&ctx.state, channel)),
        channels: Rc::new(channel_names(&ctx.state, channel)),
        role_mentions,
        guild_roles,
        menu_env: message_menu_env(
            &ctx.state,
            &ctx.backend,
            &ctx.queries,
            ctx.http.as_ref(),
            &ctx.images,
            &ctx.media_viewer,
        ),
        emoji_click,
        can_add_reactions: ctx.state.can(channel, Permissions::ADD_REACTIONS),
    });
    message_keys::set_env(cache.menu_env.clone());
    CACHE.with(|slot| *slot.borrow_mut() = Some((channel.id, cache.clone())));
    cache
}

fn with_row_env<R>(
    ctx: &ListCtx,
    channel: &Channel,
    messages: &[&models::Message],
    refresh: bool,
    f: impl FnOnce(&RowEnv<'_>) -> R,
) -> R {
    let cache = channel_cache(ctx, channel, refresh);
    let plugin_overlays = ctx.state.plugins.overlays.get();
    let show_original = ctx.state.plugins.show_original.get();
    let editing = message_edit::target()
        .filter(|(edit_channel, _)| *edit_channel == channel.id)
        .map(|(_, id)| id);
    let replying = reply::target_in(channel.id).map(|target| target.message_id);
    let current_user_id = ctx.state.users.current.get().as_ref().map(|user| user.id.get());
    #[cfg(feature = "video")]
    let video_ctx = ctx
        .videos
        .as_ref()
        .map(|(videos, fullscreen)| message_embed::VideoContext { videos, fullscreen });
    let env = RowEnv {
        state: &ctx.state,
        channel,
        fonts: &ctx.fonts,
        images: &ctx.images,
        backend: &ctx.backend,
        colors: &ctx.colors,
        profile_popout: &ctx.profile_popout,
        queries: &ctx.queries,
        attachments: &ctx.attachments,
        media_viewer: &ctx.media_viewer,
        measure_cache: &ctx.measure_cache,
        images_ready: &ctx.images_ready,
        menu_env: &cache.menu_env,
        guild_roles: &cache.guild_roles,
        mentions: &cache.mentions,
        channels: &cache.channels,
        role_mentions: &cache.role_mentions,
        emoji_click: &cache.emoji_click,
        messages,
        plugin_overlays: &plugin_overlays,
        show_original: &show_original,
        editing,
        replying,
        current_user_id,
        can_add_reactions: cache.can_add_reactions,
        #[cfg(feature = "video")]
        video_ctx,
    };
    f(&env)
}

fn visible_messages<'a>(
    ctx: &ListCtx,
    snapshot: &'a [models::Message],
) -> Vec<&'a models::Message> {
    let overlays = ctx.state.plugins.overlays.get();
    snapshot
        .iter()
        .filter(|message| {
            !overlays
                .get(&message.id)
                .is_some_and(|overlay| overlay.hidden.is_some())
        })
        .collect()
}

fn newest<'a>(ctx: &ListCtx, snapshot: &'a [models::Message]) -> Option<&'a models::Message> {
    let overlays = ctx.state.plugins.overlays.get();
    snapshot.iter().rev().find(|message| {
        !overlays
            .get(&message.id)
            .is_some_and(|overlay| overlay.hidden.is_some())
    })
}

fn row_author(env: &RowEnv<'_>, message: &models::Message) -> RowAuthor {
    let user = env.state.users.resolve(&message.author);
    let name = author_display_name(env.state, env.channel, &user);
    let style = author_style::resolve(env.state, env.guild_roles, env.channel.guild_id, &user);
    RowAuthor { user, name, style }
}

fn row_version_of(
    ctx: &ListCtx,
    channel_id: Id<ChannelMarker>,
    message_id: u64,
    starts_group: bool,
) -> Option<u64> {
    let channel = ui::displayed_channel(&ctx.state, Some(channel_id))?;
    let snapshot = ctx.state.messages.in_channel(channel.id);
    let message = snapshot.iter().find(|message| message.id.get() == message_id)?;
    let last = newest(ctx, &snapshot)?;
    with_row_env(ctx, &channel, &[last], false, |env| {
        let author = row_author(env, message);
        Some(compute_row_version(env, message, starts_group, &author))
    })
}

fn row_node_of(
    ctx: &ListCtx,
    channel_id: Id<ChannelMarker>,
    message_id: u64,
    starts_group: bool,
    version: u64,
) -> Option<Node> {
    let channel = ui::displayed_channel(&ctx.state, Some(channel_id))?;
    let snapshot = ctx.state.messages.in_channel(channel.id);
    let message = snapshot.iter().find(|message| message.id.get() == message_id)?;
    let last = newest(ctx, &snapshot)?;
    Some(with_row_env(ctx, &channel, &[last], false, |env| {
        let author = row_author(env, message);
        let input = RowInput {
            message,
            starts_group,
            row_key: message_id,
            row_version: version,
            author,
        };
        if message.kind == message_type::CHANNEL_PINNED_MESSAGE {
            pinned_row(env, &input)
        } else if message.kind == message_type::USER_JOIN {
            join_row(env, &input)
        } else if message.kind == message_type::THREAD_CREATED {
            thread_created_row(env, &input)
        } else {
            regular_row(env, &input)
        }
    }))
}

fn rebuilding(
    props: &HostProps,
    version: impl Fn(&HostProps) -> Option<u64> + 'static,
    build: impl Fn(&HostProps, u64) -> Option<Node> + 'static,
) -> Node {
    let props = props.clone();
    let (version, build) = (Rc::new(version), Rc::new(build));
    Node::new("div", BoxWidget)
        .style("display: flex; flex-direction: column")
        .on_mount(move |tree, id| {
            let current: Rc<Cell<Option<WidgetId>>> = Rc::default();
            let last: Rc<Cell<Option<u64>>> = Rc::default();
            let (props, version, build) = (props.clone(), version.clone(), build.clone());
            let effect = create_effect(
                move |tree| {
                    let next = version(&props);
                    if last.get() == next && current.get().is_some() == next.is_some() {
                        return;
                    }
                    last.set(next);
                    if let Some(old) = current.take() {
                        tree.unmount(old);
                    }
                    let Some(version) = next else { return };
                    if let Some(node) = signal::untrack(|| build(&props, version)) {
                        current.set(Some(tree.mount(id, node)));
                    }
                    tree.invalidate_styles();
                },
                tree,
            );
            tree.attach_effect(id, effect);
        })
}

fn text_of(props: &HostProps, name: &str) -> String {
    props.get(name).to_child_text()
}

fn flag(props: &HostProps, name: &str) -> bool {
    props.get(name).truthy()
}

fn channel_of(props: &HostProps) -> Option<Id<ChannelMarker>> {
    id_arg(&[props.get("channelId")], 0)
}

pub(super) fn components(components: HostComponents) -> HostComponents {
    components
        .component(
            "MessageRowHost",
            &[
                ("channelId", "ChannelId", true),
                ("messageId", "string", true),
                ("startsGroup", "boolean", true),
            ],
            |props| {
                rebuilding(
                    props,
                    |props| {
                        let ctx = ctx()?;
                        let message_id = text_of(props, "messageId").parse().ok()?;
                        row_version_of(&ctx, channel_of(props)?, message_id, flag(props, "startsGroup"))
                    },
                    |props, version| {
                        let ctx = ctx()?;
                        let message_id = text_of(props, "messageId").parse().ok()?;
                        row_node_of(
                            &ctx,
                            channel_of(props)?,
                            message_id,
                            flag(props, "startsGroup"),
                            version,
                        )
                    },
                )
            },
        )
        .component(
            "MessageDayDivider",
            &[
                ("day", "string | null", false),
                ("unread", "boolean", true),
                ("startsGroup", "boolean", true),
            ],
            |props| {
                rebuilding(
                    props,
                    |props| {
                        let key = format!(
                            "{}|{}|{}",
                            text_of(props, "day"),
                            flag(props, "unread"),
                            flag(props, "startsGroup")
                        );
                        Some(text_hash(&key))
                    },
                    |props, _| {
                        let ctx = ctx()?;
                        let day = match props.get("day") {
                            Value::Str(day) => Some(day.to_string()),
                            _ => None,
                        };
                        let meta = RowMeta {
                            starts_group: flag(props, "startsGroup"),
                            height: 0.0,
                            y: 0.0,
                            margin: 0.0,
                            day,
                            unread_marker: flag(props, "unread"),
                        };
                        Some(day_divider(&meta, &ctx.colors))
                    },
                )
            },
        )
        .component(
            "MessagePendingNotices",
            &[("channelId", "ChannelId", true), ("messageId", "string", true)],
            |props| {
                rebuilding(
                    props,
                    |props| {
                        let ctx = ctx()?;
                        let channel_id = channel_of(props)?;
                        let message_id: u64 = text_of(props, "messageId").parse().ok()?;
                        let ids: Vec<u64> = ctx.attachments.notices.with(|list| {
                            list.iter()
                                .filter(|notice| {
                                    notice.channel == channel_id
                                        && notice.after.is_some_and(|after| after.get() == message_id)
                                })
                                .map(|notice| notice.id)
                                .collect()
                        });
                        Some(text_hash(&format!("{ids:?}")))
                    },
                    |props, _| {
                        let ctx = ctx()?;
                        let message_id = text_of(props, "messageId").parse().ok()?;
                        let notices = pending_uploads::notices_after(
                            &ctx.attachments,
                            channel_of(props)?,
                            Id::new(message_id),
                        );
                        Some(div().style("display: flex; flex-direction: column").children(notices))
                    },
                )
            },
        )
        .component(
            "MessagePendingBottom",
            &[("channelId", "ChannelId", true)],
            |props| {
                rebuilding(
                    props,
                    |props| {
                        let ctx = ctx()?;
                        let channel_id = channel_of(props)?;
                        let snapshot = ctx.state.messages.in_channel(channel_id);
                        let last = newest(&ctx, &snapshot).map(|message| message.id.get());
                        let user = ctx.state.users.current.get().map(|user| user.id.get());
                        Some(text_hash(&format!("{last:?}|{user:?}")))
                    },
                    |props, _| {
                        let ctx = ctx()?;
                        let channel_id = channel_of(props)?;
                        let snapshot = ctx.state.messages.in_channel(channel_id);
                        let last = newest(&ctx, &snapshot)
                            .and_then(|m| m.timestamp_millis().map(|at| (m.author.id, at)));
                        let pending = pending_uploads::PendingContext {
                            user: ctx.state.users.current.with_untracked(Clone::clone),
                            last,
                            images: ctx.images.clone(),
                            backend: ctx.backend.clone(),
                        };
                        Some(pending_uploads::build_bottom(
                            &ctx.attachments,
                            channel_id,
                            pending,
                            &ctx.fonts,
                        ))
                    },
                )
            },
        )
        .component(
            "MessageHistoryHeader",
            &[("channelId", "ChannelId", true), ("loading", "boolean", true)],
            |props| {
                rebuilding(
                    props,
                    |props| {
                        let ctx = ctx()?;
                        let channel_id = channel_of(props)?;
                        let _ = ctx.state.messages.exhausted.get();
                        Some(channel_id.get() ^ u64::from(flag(props, "loading")))
                    },
                    |props, _| {
                        let ctx = ctx()?;
                        let channel = ui::displayed_channel(&ctx.state, channel_of(props))?;
                        Some(history_node(&ctx, &channel, flag(props, "loading")))
                    },
                )
            },
        )
}

fn history_node(ctx: &ListCtx, channel: &Channel, has_more_before: bool) -> Node {
    let container = div().style("display: flex; flex-direction: column");
    if has_more_before {
        let skeleton = skeleton::HistorySkeleton {
            spec: skeleton::spec_for(channel.id.get()),
            color: ctx.colors.text_default,
        };
        return container.child(Node::new("div", skeleton).class(message_list::HISTORY_SKELETON));
    }
    let start = channel_start::build(&ctx.state, channel, &ctx.colors).or_else(|| {
        let dm = ctx.state.dms.find(channel.id)?;
        Some(dm_start::build(
            &ctx.state,
            &dm,
            &ctx.colors,
            &ctx.profile_popout,
            &ctx.fonts,
            &ctx.images,
            &ctx.backend,
            &ctx.queries,
            ctx.http.as_ref(),
        ))
    });
    match start {
        Some(start) => container.child(start),
        None => container,
    }
}

#[derive(Clone, Copy, Serialize, Type)]
#[serde(rename_all = "lowercase")]
enum RowKind {
    History,
    Message,
    Tail,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
struct RowView {
    key: String,
    kind: RowKind,
    height: f64,
    day: Option<String>,
    unread: bool,
    starts_group: bool,
    loading: bool,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
struct ScrollPositionView {
    pinned: bool,
    key: Option<String>,
    offset: f64,
    height: f64,
}

fn entry_height(key: &str, estimate: f32) -> f64 {
    f64::from(ENTRY_HEIGHTS.with(|heights| heights.borrow().get(key).copied().unwrap_or(estimate)))
}

fn rows(args: &[Value]) -> Vec<RowView> {
    let (Some(ctx), Some(channel_id)) = (ctx(), id_arg::<ChannelMarker>(args, 0)) else {
        return Vec::new();
    };
    let Some(channel) = ui::displayed_channel(&ctx.state, Some(channel_id)) else {
        return Vec::new();
    };
    let _ = ROWS_TICK.with(Signal::get);
    let _ = ctx.state.read_states.marker_revision(channel.id).get();
    let first_unread = ctx
        .state
        .read_states
        .new_marker(channel.id)
        .and_then(|marker| marker.first_unread);
    let snapshot = ctx.state.messages.in_channel(channel.id);
    let messages = visible_messages(&ctx, &snapshot);
    let metas = compute_row_metas(&ctx.state, &messages, first_unread);
    channel_cache(&ctx, &channel, true);
    let has_more_before = !ctx.state.messages.exhausted.get().contains(&channel.id);

    let mut out = Vec::with_capacity(messages.len() + 2);
    out.push(RowView {
        key: "history".into(),
        kind: RowKind::History,
        height: entry_height("history", HISTORY_ESTIMATE),
        day: None,
        unread: false,
        starts_group: false,
        loading: has_more_before,
    });
    for (message, meta) in messages.iter().zip(&metas) {
        let key = message.id.get().to_string();
        let height = entry_height(&key, meta.margin + meta.height);
        out.push(RowView {
            key,
            kind: RowKind::Message,
            height,
            day: meta.day.clone(),
            unread: meta.unread_marker,
            starts_group: meta.starts_group,
            loading: false,
        });
    }
    out.push(RowView {
        key: "tail".into(),
        kind: RowKind::Tail,
        height: entry_height("tail", TYPING_CLEARANCE_PX),
        day: None,
        unread: false,
        starts_group: false,
        loading: false,
    });
    out
}

fn scroll(args: &[Value]) -> ScrollPositionView {
    let pinned = ScrollPositionView {
        pinned: true,
        key: None,
        offset: 0.0,
        height: 0.0,
    };
    let (Some(ctx), Some(channel_id)) = (ctx(), id_arg::<ChannelMarker>(args, 0)) else {
        return pinned;
    };
    let state = ctx
        .message_scroll
        .get()
        .get(&channel_id)
        .map(|scroll| scroll.position.clone());
    match state {
        Some(ScrollPosition::Anchored(anchor)) => ScrollPositionView {
            pinned: false,
            key: Some(anchor.key),
            offset: f64::from(anchor.offset_from_top),
            height: f64::from(anchor.height),
        },
        _ => pinned,
    }
}

pub(super) fn queries(queries: StateQueries) -> StateQueries {
    queries
        .export::<ScrollPositionView>()
        .list("messages.rows", &[("channelId", "ChannelId")], rows)
        .scalar(
            "messages.scroll",
            &[("channelId", "ChannelId")],
            "ScrollPositionView",
            scroll,
        )
}

pub(super) fn handle(action: UiAction) {
    match action {
        UiAction::MessageListScrolled {
            channel_id,
            pinned,
            key,
            offset,
            height,
        } => scrolled(channel_id, pinned, key, offset as f32, height as f32),
        UiAction::MessageRowsMeasured { keys, heights } => {
            let changed = ENTRY_HEIGHTS.with(|map| {
                let mut map = map.borrow_mut();
                let mut changed = false;
                for (key, height) in keys.into_iter().zip(heights) {
                    let height = height as f32;
                    if map.get(&key).is_none_or(|known| (known - height).abs() > 0.5) {
                        map.insert(key, height);
                        changed = true;
                    }
                }
                changed
            });
            if changed {
                ROWS_TICK.with(|tick| tick.update(|n| *n += 1));
            }
        }
        _ => {}
    }
}

fn scrolled(channel_id: Id<ChannelMarker>, pinned: bool, key: Option<String>, offset: f32, height: f32) {
    let Some(ctx) = ctx() else { return };
    let anchor = key.filter(|_| !pinned);
    let position = match &anchor {
        Some(key) => ScrollPosition::Anchored(ScrollAnchor {
            key: key.clone(),
            offset_from_top: offset,
            height,
        }),
        None => ScrollPosition::Pinned,
    };
    ctx.state.read_states.set_at_bottom(anchor.is_none());
    ctx.message_scroll.update(|map| {
        map.insert(channel_id, MessageListScroll { position });
    });
    if let Some(key) = anchor {
        load_older_if_near_top(&ctx, channel_id, &key, offset);
    }
}

fn load_older_if_near_top(ctx: &ListCtx, channel_id: Id<ChannelMarker>, key: &str, offset: f32) {
    if ctx.state.messages.exhausted.with_untracked(|exhausted| exhausted.contains(&channel_id)) {
        return;
    }
    let Some(channel) = ui::displayed_channel(&ctx.state, Some(channel_id)) else {
        return;
    };
    let snapshot = ctx.state.messages.in_channel(channel.id);
    let messages = visible_messages(ctx, &snapshot);
    let Some(index) = key
        .parse::<u64>()
        .ok()
        .and_then(|id| messages.iter().position(|message| message.id.get() == id))
    else {
        return;
    };
    if index > 60 {
        return;
    }
    let metas = compute_row_metas(&ctx.state, &messages[..messages.len().min(80)], None);
    let view_top = metas[index].y - offset;
    if view_top >= virtualization::LOAD_MORE_BUFFER_PX {
        return;
    }
    let (Some(oldest), Some(http)) = (messages.first().map(|message| message.id.get()), ctx.http.clone())
    else {
        return;
    };
    if OLDER_ASKED.with(|asked| asked.replace(Some((channel_id.get(), oldest)))) == Some((channel_id.get(), oldest)) {
        return;
    }
    let state = ctx.state.clone();
    ui::guild_settings::run(
        &ctx.queries,
        move || async move {
            messages::get_channel_messages_before(
                &http,
                channel_id,
                Id::new(oldest),
                messages::OLDER_PAGE_SIZE,
            )
            .await
            .map_err(|error| error.to_string())
        },
        move |result| {
            OLDER_ASKED.with(|asked| asked.set(None));
            if let Ok(older) = result {
                state.prepend_history(channel_id, older);
            }
        },
    );
}

#[cfg(test)]
#[path = "tsx_list_tests.rs"]
mod tests;
