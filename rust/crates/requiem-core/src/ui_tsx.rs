use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use requiem_tsx::{
    Actions, CssModules, DeclarationFile, FileDiagnostic, Host, HostComponents, Module, Options, Source,
    StateQueries, Value, compile_project, declaration_files, mount, ui_actions,
};
use requiem_ui::colors;
use requiem_ui::components::SearchBarProps;
use requiem_ui::components::line_clamp_class;
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::typography::TextVariant;
use requiem_ui::widget::primitives::{BoxWidget, ImageWidget};

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use crate::state::AppState;
use crate::ui::{self, Fonts, author_style, message_list, thread_browser, user_avatar_props};
use crate::ui_state;

ui_actions! {
    #[derive(Clone, Debug, PartialEq)]
    pub enum UiAction {
        goToThread(thread_id: Id<ChannelMarker>, shift_key: bool) => GoToThread,
        openChannelMenu(
            channel_id: Id<ChannelMarker>,
            guild_id: Option<Id<GuildMarker>>,
            x: f64,
            y: f64
        ) => OpenChannelMenu,
        loadMoreThreads(channel_id: Id<ChannelMarker>) => LoadMoreThreads,
        rememberThreadScroll(offset: f64) => RememberThreadScroll,
        startThread(channel_id: Id<ChannelMarker>) => StartThread,
        threadSearchChanged(query: String) => ThreadSearchChanged,
        submitThreadSearch(channel_id: Id<ChannelMarker>, query: String) => SubmitThreadSearch,
        closeThreadBrowser() => CloseThreadBrowser,
        messageListScrolled(
            channel_id: Id<ChannelMarker>,
            pinned: bool,
            key: Option<String>,
            offset: f64,
            height: f64
        ) => MessageListScrolled,
        messageRowsMeasured(keys: Vec<String>, heights: Vec<f64>) => MessageRowsMeasured,
    }
}

const THREAD_SOURCES: [(&str, &str); 9] = [
    (
        "ThreadBrowserRow.tsx",
        include_str!("features/threads/components/ThreadBrowserRow.tsx"),
    ),
    (
        "ThreadBrowserRowSubtext.tsx",
        include_str!("features/threads/components/ThreadBrowserRowSubtext.tsx"),
    ),
    (
        "ThreadBrowserList.tsx",
        include_str!("features/threads/components/ThreadBrowserList.tsx"),
    ),
    (
        "ThreadBrowserEntry.tsx",
        include_str!("features/threads/components/ThreadBrowserEntry.tsx"),
    ),
    (
        "ThreadBrowserEmpty.tsx",
        include_str!("features/threads/components/ThreadBrowserEmpty.tsx"),
    ),
    (
        "ThreadBrowserHeader.tsx",
        include_str!("features/threads/components/ThreadBrowserHeader.tsx"),
    ),
    (
        "ThreadBrowser.tsx",
        include_str!("features/threads/components/ThreadBrowser.tsx"),
    ),
    (
        "MessageEntry.tsx",
        include_str!("features/messages/components/MessageEntry.tsx"),
    ),
    (
        "MessageList.tsx",
        include_str!("features/messages/components/MessageList.tsx"),
    ),
];

pub fn thread_sources() -> Vec<Source> {
    THREAD_SOURCES
        .iter()
        .map(|(path, text)| Source::new(path, text))
        .collect()
}

pub fn css_modules() -> CssModules {
    CssModules::from_json(include_str!(concat!(env!("OUT_DIR"), "/css_modules.json")))
        .expect("the build writes valid css_modules.json")
}

fn user_id(value: &Value) -> Option<Id<UserMarker>> {
    match value {
        Value::Str(text) => Id::from_str(text).ok(),
        _ => None,
    }
}

fn classes_of(node: Node, value: &Value) -> Node {
    match value {
        Value::Str(text) => node.classes(text.split_whitespace().map(str::to_string)),
        _ => node,
    }
}

pub fn components(
    state: &AppState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    tick: &Signal<u64>,
) -> HostComponents {
    let avatar = {
        let (users, fonts, images, backend, tick) = (
            state.users.clone(),
            fonts.clone(),
            images.clone(),
            backend.clone(),
            tick.clone(),
        );
        move |props: &requiem_tsx::HostProps| {
            let Some(id) = user_id(&props.peek("userId")) else {
                return Node::new("div", BoxWidget);
            };
            let discriminator = users
                .get(id)
                .map(|user| user.discriminator)
                .unwrap_or_else(|| "0".into());
            let (users, images, backend) = (users.clone(), images.clone(), backend.clone());
            let node = user_avatar_props(id, &discriminator, &fonts)
                .bind_image(tick.clone(), move || {
                    let hash = users.get_watched(id)?.avatar?;
                    images.get_or_decode_image(&backend.avatar_url(id, &hash))
                })
                .build();
            classes_of(node, &props.peek("className"))
        }
    };
    let author_name = {
        let state = state.clone();
        move |props: &requiem_tsx::HostProps| {
            let Some(user) = user_id(&props.peek("userId")).and_then(|id| state.users.get(id))
            else {
                return Node::new("div", BoxWidget);
            };
            let guild_id = match props.peek("guildId") {
                Value::Str(text) => Id::<GuildMarker>::from_str(&text).ok(),
                _ => None,
            };
            let roles = guild_id
                .map(|guild_id| author_style::guild_roles(&state, guild_id))
                .unwrap_or_default();
            let style = author_style::resolve(&state, &roles, guild_id, &user);
            let node = author_style::name_node(
                state.users.display_name(&user),
                TextVariant::TEXT_SM_SEMIBOLD,
                18.0,
                style.fill.as_ref(),
                colors::TEXT_DEFAULT,
            )
            .class(line_clamp_class(1));
            classes_of(node, &props.peek("className"))
        }
    };
    let asset_image = |props: &requiem_tsx::HostProps| {
        let Value::Str(name) = props.peek("name") else {
            return Node::new("div", BoxWidget);
        };
        let edge = |prop: &str| props.peek(prop).as_f32().max(1.0) as u32;
        let image = ui::asset_svg(&name, edge("width"), edge("height"));
        classes_of(
            Node::new("div", ImageWidget::new(image)),
            &props.peek("className"),
        )
    };
    let search_bar = |props: &requiem_tsx::HostProps| {
        let placeholder = props.peek("placeholder").to_child_text();
        let (changed, submitted) = (props.clone(), props.clone());
        let mut bar = SearchBarProps::new(&placeholder, move |query: &str| {
            changed.call("onChange", &[Value::str(query)]);
        })
        .on_submit(move |query: String| submitted.call("onSubmit", &[Value::str(query)]))
        .autofocus(props.peek("autofocus").truthy())
        .clearable(props.peek("clearable").truthy());
        if let Value::Str(text) = props.peek("text") {
            bar = bar.text(&*text);
        }
        classes_of(bar.build(), &props.peek("className"))
    };
    HostComponents::new()
        .component(
            "SearchBar",
            &[
                ("placeholder", "string", true),
                ("text", "string", false),
                ("autofocus", "boolean", false),
                ("clearable", "boolean", false),
                ("onChange", "(query: string) => void", true),
                ("onSubmit", "(query: string) => void", false),
                ("className", "string", false),
            ],
            search_bar,
        )
        .component(
            "AssetImage",
            &[
                ("name", "string", true),
                ("width", "number", true),
                ("height", "number", true),
                ("className", "string", false),
            ],
            asset_image,
        )
        .component(
            "Avatar",
            &[("userId", "UserId", true), ("className", "string", false)],
            avatar,
        )
        .component(
            "AuthorName",
            &[
                ("userId", "UserId", true),
                ("guildId", "GuildId | null", false),
                ("className", "string", false),
            ],
            author_name,
        )
}

pub fn options(queries: &StateQueries, components: &HostComponents) -> Options {
    Options {
        actions: UiAction::specs(),
        queries: queries.specs(),
        host: components.specs(),
        css_modules: css_modules(),
    }
}

thread_local! {
    static SHARED: RefCell<Option<Option<Rc<Tsx>>>> = const { RefCell::new(None) };
}

pub fn shared(
    state: &AppState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    tick: &Signal<u64>,
) -> Option<Rc<Tsx>> {
    SHARED.with(|cell| {
        let mut cell = cell.borrow_mut();
        if cell.is_none() {
            let built = Tsx::new(state, fonts, images, backend, tick, dispatch);
            if let Err(errors) = &built {
                let sources = thread_sources();
                for error in errors {
                    if let Some(source) = sources.iter().find(|source| source.path == error.path) {
                        log::error!("{}", error.diagnostic.render(&error.path, &source.text));
                    }
                }
            }
            *cell = Some(built.ok().map(Rc::new));
        }
        cell.clone().flatten()
    })
}

fn dispatch(action: UiAction) {
    thread_browser::handle(action.clone());
    message_list::handle_tsx(action);
}

pub struct Tsx {
    module: Module,
    host: Rc<Host>,
    declarations: Vec<DeclarationFile>,
}

impl Tsx {
    pub fn new(
        state: &AppState,
        fonts: &Fonts,
        images: &ImageCache,
        backend: &BackendConfig,
        tick: &Signal<u64>,
        on_action: impl Fn(UiAction) + 'static,
    ) -> Result<Self, Vec<FileDiagnostic>> {
        let queries = message_list::tsx_queries(thread_browser::browser_queries(ui_state::queries(state)));
        let components = message_list::tsx_components(components(state, fonts, images, backend, tick));
        let options = options(&queries, &components);
        let module = compile_project(&thread_sources(), &options)?;
        let declarations = declaration_files(
            &options.css_modules,
            &options.actions,
            &queries,
            &options.host,
            requiem_i18n::catalog(),
        );
        let host = Host::new(on_action);
        host.serve(&queries);
        host.serve_components(&components);
        Ok(Self {
            module,
            host,
            declarations,
        })
    }

    pub fn declaration_files(&self) -> &[DeclarationFile] {
        &self.declarations
    }

    pub fn component_names(&self) -> Vec<String> {
        self.module.components.iter().map(|c| c.name.clone()).collect()
    }

    pub fn node(&self, component: &str, props: Vec<(&str, Value)>) -> Node {
        let (module, host, component) = (self.module.clone(), self.host.clone(), component.to_string());
        let props: Vec<(String, Value)> = props
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect();
        Node::new("div", BoxWidget).on_mount(move |tree, id| {
            let signals: Vec<(String, Signal<Value>)> = props
                .iter()
                .map(|(name, value)| (name.clone(), Signal::new(value.clone())))
                .collect();
            let given: Vec<(&str, Signal<Value>)> = signals
                .iter()
                .map(|(name, signal)| (name.as_str(), signal.clone()))
                .collect();
            mount(tree, id, &module, &component, &given, &host);
        })
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::cell::RefCell;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::rc::Rc;

    use requiem_ui::event::{MouseButton, UiEvent};
    use requiem_ui::font_system::{self, TextFonts};
    use requiem_ui::signal::{Signal, flush};
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::theme as requiem_ui_theme;
    use requiem_ui::widget::primitives::TextWidget;
    use requiem_ui::widget::{AbsoluteRect, WidgetId, WidgetTree};
    use skia_safe::font_style::Weight;
    use tokio::runtime::Runtime;

    use super::*;
    use crate::models::id::DISCORD_EPOCH_MS;
    use crate::models::{Channel, Message, User};
    use crate::state::Action;
    use crate::ui_state::{ChannelView, queries};
    use crate::utils::time;

    struct Harness {
        tree: WidgetTree,
        tsx: Tsx,
        state: AppState,
        events: Rc<RefCell<Vec<UiAction>>>,
        _runtime: Runtime,
    }

    fn user(id: u64, name: &str) -> User {
        serde_json::from_value(serde_json::json!({
            "id": id.to_string(), "username": name, "discriminator": "0"
        }))
        .expect("user")
    }

    fn channel(json: serde_json::Value) -> Channel {
        serde_json::from_value(json).expect("channel")
    }

    fn harness() -> Harness {
        let runtime = Runtime::new().expect("runtime");
        let images = ImageCache::new(runtime.handle().clone(), wreq::Client::new(), || {});
        let fonts = Fonts {
            typeface: requiem_ui_theme::make_typeface(Weight::MEDIUM),
            text_fonts: TextFonts::new(1024 * 1024),
        };
        font_system::install_ambient(fonts.text_fonts.clone());
        let backend = BackendConfig {
            rest_base_url: String::new(),
            gateway_url: String::new(),
            cdn_base_url: String::new(),
        };
        let state = AppState::new();
        let events = Rc::new(RefCell::new(Vec::new()));
        let sink = events.clone();
        let tsx = Tsx::new(&state, &fonts, &images, &backend, &Signal::new(0), move |action| {
            sink.borrow_mut().push(action)
        })
        .unwrap_or_else(|errors| {
            let sources = thread_sources();
            panic!(
                "{}",
                errors
                    .iter()
                    .map(|e| {
                        let text = &sources.iter().find(|s| s.path == e.path).expect("source").text;
                        e.diagnostic.render(&e.path, text)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        });
        Harness {
            tree: WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default()),
            tsx,
            state,
            events,
            _runtime: runtime,
        }
    }

    fn texts(tree: &WidgetTree, id: WidgetId, out: &mut Vec<String>) {
        if let Some(text) = (&*tree.nodes[id].widget as &dyn Any).downcast_ref::<TextWidget>() {
            out.push(text.text().to_string());
        }
        for &child in &tree.nodes[id].children {
            texts(tree, child, out);
        }
    }

    fn mount_row(h: &mut Harness, thread: &Channel) -> WidgetId {
        let props = vec![("thread", requiem_tsx::to_value(&ChannelView::from(thread)))];
        let root = h.tree.root;
        let node = h.tsx.node("ThreadBrowserRow", props);
        h.tree.mount(root, node)
    }

    fn row_texts(h: &Harness, wrapper: WidgetId) -> Vec<String> {
        let mut out = Vec::new();
        texts(&h.tree, wrapper, &mut out);
        out
    }

    fn newest_snowflake() -> u64 {
        ((time::now_ms() as i64 - DISCORD_EPOCH_MS) as u64) << 22
    }

    #[test]
    fn an_active_thread_row_shows_its_name_latest_message_and_time() {
        let mut h = harness();
        h.state.users.observe(&user(1, "ana"));
        let thread = channel(serde_json::json!({
            "id": "200", "guild_id": "9", "type": 11, "name": "release notes", "parent_id": "100"
        }));
        h.state.channels.upsert(thread.clone(), false);
        let message: Message = serde_json::from_value(serde_json::json!({
            "id": newest_snowflake().to_string(),
            "channel_id": "200",
            "author": {"id": "1", "username": "ana", "discriminator": "0"},
            "content": "hi   there\nfriend",
            "timestamp": "2026-10-27T00:00:00.000000+00:00"
        }))
        .expect("message");
        h.state.dispatch(&Action::MessageCreate(message));
        let wrapper = mount_row(&mut h, &thread);
        assert_eq!(
            row_texts(&h, wrapper),
            ["release notes", "ana", ":\u{a0}hi there friend", "\u{a0} \u{2022} \u{a0}1m ago"]
        );
    }

    #[test]
    fn a_thread_without_messages_says_so() {
        let mut h = harness();
        let thread = channel(serde_json::json!({
            "id": "201", "guild_id": "9", "type": 11, "name": "quiet", "parent_id": "100"
        }));
        h.state.channels.upsert(thread.clone(), false);
        let wrapper = mount_row(&mut h, &thread);
        let texts = row_texts(&h, wrapper);
        assert_eq!(texts[0], "quiet");
        assert!(texts[1].starts_with("No recent messages"), "{texts:?}");
    }

    #[test]
    fn an_archived_thread_shows_who_started_it_or_a_thread_icon() {
        let mut h = harness();
        h.state.users.observe(&user(2, "bo"));
        let with_owner = channel(serde_json::json!({
            "id": "202", "guild_id": "9", "type": 11, "name": "old plans", "parent_id": "100",
            "owner_id": "2",
            "thread_metadata": {"archived": true, "auto_archive_duration": 1440}
        }));
        let without_owner = channel(serde_json::json!({
            "id": "203", "guild_id": "9", "type": 11, "name": "orphan", "parent_id": "100",
            "thread_metadata": {"archived": true, "auto_archive_duration": 1440}
        }));
        h.state.channels.upsert(with_owner.clone(), false);
        let first = mount_row(&mut h, &with_owner);
        assert_eq!(
            row_texts(&h, first),
            ["old plans", "Started by", "bo", "\u{2022}", "Last active 1m ago"]
        );
        let second = mount_row(&mut h, &without_owner);
        assert_eq!(row_texts(&h, second), ["orphan", "Last active 1m ago"]);
    }

    #[test]
    fn a_new_message_updates_the_row_in_place() {
        let mut h = harness();
        h.state.users.observe(&user(1, "ana"));
        let thread = channel(serde_json::json!({
            "id": "204", "guild_id": "9", "type": 11, "name": "live", "parent_id": "100"
        }));
        h.state.channels.upsert(thread.clone(), false);
        let wrapper = mount_row(&mut h, &thread);
        assert!(row_texts(&h, wrapper)[1].starts_with("No recent messages"));

        let message: Message = serde_json::from_value(serde_json::json!({
            "id": newest_snowflake().to_string(),
            "channel_id": "204",
            "author": {"id": "1", "username": "ana", "discriminator": "0"},
            "content": "arrived",
            "timestamp": "2026-10-27T00:00:00.000000+00:00"
        }))
        .expect("message");
        h.state.dispatch(&Action::MessageCreate(message));
        flush(&mut h.tree);
        assert_eq!(
            row_texts(&h, wrapper),
            ["live", "ana", ":\u{a0}arrived", "\u{a0} \u{2022} \u{a0}1m ago"]
        );
    }

    #[test]
    fn clicking_and_right_clicking_the_row_reach_the_rust_actions() {
        let mut h = harness();
        let thread = channel(serde_json::json!({
            "id": "205", "guild_id": "9", "type": 11, "name": "clicky", "parent_id": "100"
        }));
        h.state.channels.upsert(thread.clone(), false);
        let wrapper = mount_row(&mut h, &thread);
        let button = h.tree.nodes[wrapper].children[0];
        let rect = AbsoluteRect::default();
        h.tree.nodes[button].widget.handle_event(
            &UiEvent::MouseDown {
                x: 0.0,
                y: 0.0,
                button: MouseButton::Left,
                shift: true,
            },
            rect,
        );
        h.tree.nodes[button].widget.handle_event(
            &UiEvent::MouseUp {
                x: 0.0,
                y: 0.0,
                button: MouseButton::Left,
            },
            rect,
        );
        let menu = h.tree.nodes[button].context_menu.clone().expect("context menu");
        menu(12.0, 34.0);
        assert_eq!(
            *h.events.borrow(),
            vec![
                UiAction::GoToThread {
                    thread_id: Id::new(205),
                    shift_key: true
                },
                UiAction::OpenChannelMenu {
                    channel_id: Id::new(205),
                    guild_id: Some(Id::new(9)),
                    x: 12.0,
                    y: 34.0
                },
            ]
        );
        let _ = queries(&h.state);
    }

    #[test]
    fn the_checked_in_declarations_match_the_compiler() {
        let harness = harness();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("types");
        let files = harness.tsx.declaration_files();
        if env::var_os("REQUIEM_UPDATE_TYPES").is_some() {
            let _ = fs::remove_dir_all(&root);
            for file in files {
                let path = root.join(&file.path);
                fs::create_dir_all(path.parent().expect("parent")).expect("create dir");
                fs::write(path, &file.text).expect("write declaration");
            }
        }
        for file in files {
            let checked_in = fs::read_to_string(root.join(&file.path))
                .unwrap_or_default()
                .replace("\r\n", "\n");
            assert_eq!(
                checked_in, file.text,
                "{}: run with REQUIEM_UPDATE_TYPES=1 to regenerate",
                file.path
            );
        }
        let mut stale = Vec::new();
        collect(&root, &root, &mut stale);
        stale.retain(|path| !files.iter().any(|file| file.path == *path));
        assert!(stale.is_empty(), "stale declaration files: {stale:?}");
    }

    fn collect(root: &Path, dir: &Path, out: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect(root, &path, out);
            } else if let Ok(relative) = path.strip_prefix(root) {
                out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }

    #[test]
    fn the_thread_components_are_all_compiled() {
        let h = harness();
        let mut names = h.tsx.component_names();
        names.sort();
        assert_eq!(
            names,
            [
                "MessageEntry/MessageEntry",
                "MessageList/MessageList",
                "ThreadBrowser/ThreadBrowser",
                "ThreadBrowserEmpty/ThreadBrowserEmpty",
                "ThreadBrowserEntry/ThreadBrowserEntry",
                "ThreadBrowserHeader/ThreadBrowserHeader",
                "ThreadBrowserList/ThreadBrowserList",
                "ThreadBrowserRow/ThreadBrowserRow",
                "ThreadBrowserRowSubtext/ActiveThreadSubtext",
                "ThreadBrowserRowSubtext/ArchiveTimeSubtext",
            ]
        );
    }
}
