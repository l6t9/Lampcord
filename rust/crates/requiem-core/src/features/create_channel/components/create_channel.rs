use crate::models::PermissionOverwrite;
use crate::models::Permissions;
use crate::gateway::member_requests;
use crate::models::permissions::OverwriteKind;
use crate::models;
use crate::rest::endpoints::channels;
use crate::ui::classes::create_channel;
use crate::ui::classes::create_channel as cls;
use crate::ui::emoji_picker;
use crate::ui;
use requiem_ui::colors;
use requiem_ui::components::Align;
use requiem_ui::components;
use requiem_ui::event::NamedKey;
use requiem_ui::icon_handle;
use requiem_ui::widget::primitives::PopoverPanelWidget;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use requiem_ui::components::{
    ButtonProps, ButtonVariant, FormItemProps, HeadingLevel, HeadingProps, ModalProps, ModalSize,
    RadioGroupProps, RadioOption, SlideShow, StackProps, SwitchProps, TextInputProps, TextProps,
    TextVariant, anchored_popover_backdrop, checkbox_indicator, div, field_error,
    input_leading_icon,
};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::widget::anim::{ToggleMemo, ToggleState};
use requiem_ui::widget::{WidgetId, WidgetTree};

use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};
use crate::rest::RestError;
use crate::state::NavEvent;
use crate::ui::dialog::Dialog;
use crate::ui::emoji_button_icon::EmojiButtonIconWidget;
use crate::ui::home::HomeEnv;
use crate::ui::{TEXT_DEFAULT, TEXT_MUTED};
use requiem_ui::icons::{
    FORUM, FORUM_LOCK, LOCK, SHIELD_USER, TEXT, TEXT_LOCK, VOICE_LOCK, VOICE_NORMAL, X_LARGE,
    X_SMALL,
};

const KIND_TEXT: u8 = 0;
const KIND_VOICE: u8 = 2;
const KIND_CATEGORY: u8 = 4;
const KIND_FORUM: u8 = 15;

const STEP_WIDTH: f32 = 478.0;
const SLIDE_INFO: usize = 0;
const SLIDE_MEMBERS: usize = 1;
thread_local! {
    static DIALOG: Dialog = Dialog::new();

    static GUILD: Cell<Option<Id<GuildMarker>>> = const { Cell::new(None) };

    static PARENT: Cell<Option<Id<ChannelMarker>>> = const { Cell::new(None) };

    static NAME: RefCell<String> = const { RefCell::new(String::new()) };

    static NAME_READY: Signal<bool> = signal(false);
    static KIND: Signal<u8> = signal(KIND_TEXT);
    static PRIVATE: Signal<bool> = signal(false);
    static SLIDES: OnceCell<Rc<SlideShow>> = const { OnceCell::new() };
    static BUSY: Signal<bool> = signal(false);
    static ERROR: Signal<Option<String>> = signal(None);
    static KIND_TOGGLES: [(u8, ToggleMemo); 3] = [
        (KIND_TEXT, ToggleMemo::new(true)),
        (KIND_VOICE, ToggleMemo::new(false)),
        (KIND_FORUM, ToggleMemo::new(false)),
    ];
    static PRIVATE_TOGGLE: ToggleMemo = ToggleMemo::new(false);

    static QUERY: Signal<String> = signal(String::new());

    static PICKED: Signal<Vec<Pick>> = signal(Vec::new());

    static ROWS: RefCell<Vec<(Option<Pick>, Signal<bool>)>> = const { RefCell::new(Vec::new()) };

    static SELECTED: Cell<Option<usize>> = const { Cell::new(None) };

    static NAME_EMOJI_OPEN: Signal<bool> = signal(false);
    static EMOJI_SIGNALS: emoji_picker::EmojiPickerSignals = emoji_picker::EmojiPickerSignals { search: signal(String::new()), scroll_offset: signal(0.0), category_jump: signal(None), hovered: signal(None), collapsed: signal(Vec::new()) };

    static INSERT_TEXT: RefCell<Option<Rc<dyn Fn(&'static str, String)>>> = const { RefCell::new(None) };
}

const NAME_DOM_ID: &str = "create-channel-name";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pick {
    Role(Id<RoleMarker>),
    Member(Id<UserMarker>),
}

fn slides() -> Rc<SlideShow> {
    SLIDES.with(|cell| cell.get_or_init(|| SlideShow::new(STEP_WIDTH)).clone())
}

pub fn is_open() -> bool {
    DIALOG.with(Dialog::is_open)
}

pub fn open(guild_id: Id<GuildMarker>, parent: Option<Id<ChannelMarker>>) {
    reset_state();
    GUILD.with(|g| g.set(Some(guild_id)));
    PARENT.with(|p| p.set(parent));
    DIALOG.with(Dialog::open);
}

pub fn open_category(guild_id: Id<GuildMarker>) {
    reset_state();
    KIND.with(|k| k.set(KIND_CATEGORY));
    GUILD.with(|g| g.set(Some(guild_id)));
    DIALOG.with(Dialog::open);
}

pub fn close() {
    DIALOG.with(|dialog| dialog.close_then(reset_state));
}

fn reset_state() {
    NAME.with(|n| n.borrow_mut().clear());
    NAME_READY.with(|n| n.set(false));
    KIND.with(|k| k.set(KIND_TEXT));
    KIND_TOGGLES.with(|toggles| {
        for (value, memo) in toggles {
            memo.reset(*value == KIND_TEXT);
        }
    });
    PRIVATE.with(|p| p.set(false));
    PRIVATE_TOGGLE.with(|memo| memo.reset(false));
    QUERY.with(|q| q.set(String::new()));
    PICKED.with(|p| p.set(Vec::new()));
    SELECTED.with(|s| s.set(None));
    ROWS.with(|rows| rows.borrow_mut().clear());
    BUSY.with(|b| b.set(false));
    ERROR.with(|e| e.set(None));
    close_emoji_picker();
    slides().reset(SLIDE_INFO);
}

pub fn finish(error: Option<String>) {
    if let Some(message) = error {
        ERROR.with(|e| e.set(Some(message)));
        BUSY.with(|b| b.set(false));
        slides().reset(SLIDE_INFO);
    } else {
        close();
    }
}

fn kind_icon(kind: u8, private: bool) -> icon_handle::IconSpec {
    match (kind, private) {
        (KIND_VOICE, true) => VOICE_LOCK.into(),
        (KIND_VOICE, false) => VOICE_NORMAL.into(),
        (KIND_FORUM, true) => FORUM_LOCK.into(),
        (KIND_FORUM, false) => FORUM.into(),
        (_, true) => TEXT_LOCK.into(),
        (_, false) => TEXT.into(),
    }
}

fn placeholder(kind: u8) -> String {
    match kind {
        KIND_FORUM => requiem_i18n::t!("create-channel-forum-placeholder"),
        KIND_CATEGORY => requiem_i18n::t!("create-channel-new-category"),
        _ => requiem_i18n::t!("create-channel-new-channel"),
    }
}

fn name_text() -> String {
    NAME.with(|n| n.borrow().clone())
}

fn set_name(text: &str) {
    let ready = !text.trim().is_empty();
    NAME.with(|n| *n.borrow_mut() = text.to_string());
    if NAME_READY.with(|n| n.peek()) != ready {
        NAME_READY.with(|n| n.set(ready));
    }
}

fn dialog(env: &HomeEnv) -> Node {
    let show = slides();
    let env = env.clone();
    let body = Node::reactive(move || {
        show.view(|index| {
            if index == SLIDE_MEMBERS {
                members_slide(&env)
            } else {
                info_slide(&env)
            }
        })
    });
    ModalProps::default()
        .size(ModalSize::Medium)
        .header(false)
        .flush_body(true)
        .dismissable(true)
        .on_dismiss(close)
        .body(body)
        .open(DIALOG.with(Dialog::toggle))
        .build()
}

fn info_slide(env: &HomeEnv) -> Node {
    let kind = KIND.with(Signal::get);
    let private = PRIVATE.with(Signal::get);
    let category = kind == KIND_CATEGORY;
    let option = |value: u8, label: String, desc: String| {
        let state = KIND_TOGGLES.with(|toggles| {
            toggles
                .iter()
                .find(|(v, _)| *v == value)
                .map(|(_, memo)| memo.track(kind == value))
                .expect("channel kind memo")
        });
        RadioOption::new(value.to_string(), label, state)
            .with_description(desc)
            .with_icon(kind_icon(value, private))
    };

    let create_env = env.clone();
    let emoji = Node::new("span", EmojiButtonIconWidget::new()).class(cls::FIELD_EMOJI);
    let emoji_btn = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::EMOJI_BTN.to_string()])
        .on_click(toggle_emoji_picker)
        .build()
        .child(emoji);

    let emoji_picker = Node::reactive(emoji_picker_panel).class(cls::EMOJI_PICKER_SLOT);
    let emoji_trailing = div()
        .class(cls::EMOJI_ANCHOR)
        .children([emoji_btn, emoji_picker]);
    let mut name_input = TextInputProps::new()
        .text(name_text())
        .placeholder(placeholder(kind))
        .trailing(emoji_trailing)
        .max_length(100usize)
        .autofocus(true)
        .on_change(|text: &str| set_name(text))
        .on_submit(move |text| submit_name(&create_env, text));

    if !category {
        name_input = name_input.leading(input_leading_icon(kind_icon(kind, private)));
    }
    let name_input = name_input.build().dom_id(NAME_DOM_ID);
    let name_error = Node::reactive(move || match ERROR.with(Signal::get) {
        Some(message) => field_error(message),
        None => div(),
    });
    let name_label = if category {
        requiem_i18n::t!("create-channel-category-name")
    } else {
        requiem_i18n::t!("create-channel-channel-name")
    };
    let name_field = FormItemProps::new(
        name_label,
        StackProps::vertical()
            .gap(0)
            .full_width(false)
            .build()
            .children([name_input, name_error]),
    )
    .build();

    let private_on = PRIVATE.with(Signal::get);
    let private_switch = SwitchProps::new(
        PRIVATE_TOGGLE.with(|memo| memo.track(private_on)),
        |on: bool| PRIVATE.with(|p| p.set(on)),
    )
    .build();
    let (private_title, private_description) = if category {
        (
            requiem_i18n::t!("create-channel-private-category"),
            requiem_i18n::t!("create-channel-private-category-description"),
        )
    } else {
        (
            requiem_i18n::t!("create-channel-private-channel"),
            requiem_i18n::t!("create-channel-private-channel-description"),
        )
    };
    let private_field = FormItemProps::default()
        .title(private_title.to_string())
        .description(private_description.to_string())
        .icon(LOCK)
        .horizontal(true)
        .children(private_switch)
        .build();

    let mut fields = Vec::new();
    if !category {
        let channel_type = RadioGroupProps::new(move |value: String| {
            KIND.with(|k| k.set(value.parse::<u8>().unwrap_or(KIND_TEXT)))
        })
        .options(vec![
            option(
                KIND_TEXT,
                requiem_i18n::t!("create-channel-text").to_string(),
                requiem_i18n::t!("create-channel-text-description").to_string(),
            ),
            option(
                KIND_VOICE,
                requiem_i18n::t!("create-channel-voice").to_string(),
                requiem_i18n::t!("create-channel-voice-description").to_string(),
            ),
            option(
                KIND_FORUM,
                requiem_i18n::t!("create-channel-forum").to_string(),
                requiem_i18n::t!("create-channel-forum-description").to_string(),
            ),
        ])
        .build();
        fields.push(
            FormItemProps::new(
                requiem_i18n::t!("create-channel-channel-type"),
                channel_type,
            )
            .build(),
        );
    }
    fields.push(name_field);
    fields.push(private_field);
    let body = StackProps::vertical()
        .gap(20)
        .full_width(true)
        .build()
        .children(fields);

    let submit_env = env.clone();
    let cancel = ButtonProps::new(
        ButtonVariant::Secondary,
        requiem_i18n::t!("ui-cancel"),
        close,
    )
    .build();
    let ready = NAME_READY.with(Signal::get);
    let submit_label = if private {
        requiem_i18n::t!("create-channel-next")
    } else {
        create_label(kind)
    };
    let create_button = ButtonProps::new(ButtonVariant::Primary, submit_label, move || {
        submit_name(&submit_env, name_text())
    })
    .disabled(!ready || BUSY.with(|b| b.peek()))
    .build();
    let mut header = title_block(create_label(kind).to_string());
    if let Some(category) = PARENT
        .with(|p| p.get())
        .filter(|_| !category)
        .and_then(|parent| ui::find_channel(&env.state, parent))
        .and_then(|(_, channel)| channel.name)
    {
        header = header.child(
            TextProps::styled(
                requiem_i18n::t!("create-channel-in-category", category = category),
                TextVariant::TEXT_MD_NORMAL,
            )
            .color(TEXT_MUTED)
            .build(),
        );
    }
    step(header, body, vec![cancel, create_button])
}

fn create_label(kind: u8) -> String {
    if kind == KIND_CATEGORY {
        requiem_i18n::t!("context-menus-create-category")
    } else {
        requiem_i18n::t!("context-menus-create-channel")
    }
}

fn toggle_emoji_picker() {
    let open = !NAME_EMOJI_OPEN.with(|o| o.peek());
    if open {
        EMOJI_SIGNALS.with(|s| s.reset_for_open());
    }
    NAME_EMOJI_OPEN.with(|o| o.set(open));
}

fn close_emoji_picker() {
    if NAME_EMOJI_OPEN.with(|o| o.peek()) {
        NAME_EMOJI_OPEN.with(|o| o.set(false));
    }
}

fn emoji_picker_panel() -> Node {
    if !NAME_EMOJI_OPEN.with(Signal::get) {
        return div();
    }
    let on_pick = Rc::new(|glyph: String, keep_open: bool| {
        if let Some(insert) = INSERT_TEXT.with(|slot| slot.borrow().clone()) {
            insert(NAME_DOM_ID, glyph);
        }
        if !keep_open {
            close_emoji_picker();
        }
    });

    let panel = EMOJI_SIGNALS
        .with(|signals| emoji_picker::build_emoji_only(signals, on_pick, None))
        .style("right: 0px; bottom: 0px");
    div().children([
        anchored_popover_backdrop(close_emoji_picker),
        Node::new(
            "div",
            PopoverPanelWidget::new(),
        )
        .child(panel),
    ])
}

fn members_slide(env: &HomeEnv) -> Node {
    let kind = KIND.with(Signal::get);
    let private = PRIVATE.with(Signal::get);

    let mut subtitle = StackProps::horizontal()
        .gap(4)
        .align(Align::Center)
        .full_width(false)
        .build();
    if kind != KIND_CATEGORY {
        subtitle = subtitle.child(
            components::icon(kind_icon(kind, private), TEXT_MUTED)
                .class(cls::SUBTITLE_ICON),
        );
    }
    let subtitle = subtitle.child(
        TextProps::styled(name_text(), TextVariant::TEXT_MD_NORMAL)
            .color(TEXT_MUTED)
            .build(),
    );
    let header =
        title_block(requiem_i18n::t!("create-channel-add-members").to_string()).child(subtitle);

    if let Some(guild) = GUILD.with(|g| g.get()) {
        member_requests::request_query(
            guild.get(),
            &member_query(&QUERY.with(|q| q.peek())),
        );
    }
    let search_env = env.clone();
    let search = Node::reactive(move || search_box(&search_env));
    let hint = div().class(cls::SEARCH_HINT).child(
        TextProps::styled(
            requiem_i18n::t!("create-channel-add-members-help").to_string(),
            TextVariant::TEXT_XS_NORMAL,
        )
        .color(TEXT_DEFAULT)
        .build(),
    );
    let controls = div().class(cls::MEMBER_CONTROLS).children([search, hint]);
    let list_env = env.clone();
    let list = components::scroll_view()
        .class(cls::MEMBER_LIST)
        .child(Node::reactive(move || picker_results(&list_env)).class(cls::MEMBER_ROWS_SLOT));
    let body = div().class(cls::MEMBER_BODY).children([controls, list]);

    let back = ButtonProps::new(
        ButtonVariant::Secondary,
        requiem_i18n::t!("create-channel-back"),
        || slides().go(SLIDE_INFO),
    )
    .build();
    let submit_env = env.clone();

    let submit = Node::reactive(move || {
        let env = submit_env.clone();
        let label = if PICKED.with(Signal::get).is_empty() {
            requiem_i18n::t!("create-channel-skip")
        } else {
            create_label(kind)
        };
        ButtonProps::new(ButtonVariant::Primary, label, move || create(&env))
            .disabled(BUSY.with(Signal::get))
            .build()
    })
    .class(cls::FOOTER_SLOT);

    step(header, body, vec![back, submit])
}

fn step(header: Node, body: Node, footer: Vec<Node>) -> Node {
    let close_button = ButtonProps::default()
        .variant(ButtonVariant::IconOnly)
        .icon(X_LARGE)
        .on_click(close)
        .extra_classes(vec![cls::CLOSE.to_string()])
        .build();
    let header_row = StackProps::horizontal()
        .align(Align::Start)
        .full_width(true)
        .build()
        .child(div().class(cls::STEP_TITLE).child(header))
        .child(close_button);
    div()
        .class(cls::STEP)
        .child(div().class(cls::STEP_HEADER).child(header_row))
        .child(div().class(cls::STEP_BODY).child(body))
        .child(div().class(cls::STEP_FOOTER).children(footer))
}

fn title_block(text: String) -> Node {
    StackProps::vertical()
        .gap(8)
        .full_width(false)
        .build()
        .child(
            HeadingProps::new(HeadingLevel::H1, text)
                .variant(TextVariant::HEADING_LG_SEMIBOLD)
                .build(),
        )
}

fn member_query(raw: &str) -> String {
    let trimmed = raw.trim();
    trimmed.strip_prefix('@').unwrap_or(trimmed).to_string()
}

fn set_query(text: &str) {
    QUERY.with(|q| q.set(text.to_string()));
    if let Some(guild) = GUILD.with(|g| g.get()) {
        member_requests::request_query(guild.get(), &member_query(text));
    }
}

fn search_box(env: &HomeEnv) -> Node {
    let picked = PICKED.with(Signal::get);
    let placeholder = if picked.is_empty() {
        requiem_i18n::t!("create-channel-add-members-placeholder").to_string()
    } else {
        String::new()
    };
    let input = TextInputProps::new()
        .bare(true)
        .text(QUERY.with(|q| q.peek()))
        .placeholder(placeholder)
        .insets((8.0_f32, 8.0_f32))
        .autofocus(true)
        .on_change(|text: &str| set_query(text))
        .key_hook(|key, _shift, text: &str| picker_key(key, text))
        .build()
        .class(cls::SEARCH_INPUT);
    let mut tags: Vec<Node> = picked.iter().filter_map(|pick| tag(env, *pick)).collect();
    tags.push(input);
    div().class(cls::SEARCH_BOX).children(tags)
}

fn tag(env: &HomeEnv, pick: Pick) -> Option<Node> {
    let guild_id = GUILD.with(|g| g.get())?;
    let (lead, label) = match pick {
        Pick::Role(id) => {
            let role = env
                .state
                .permissions
                .guild_roles(guild_id)
                .into_iter()
                .find(|r| r.id == id)?;
            let [r, g, b] = rgb(role_color(role.color));
            (
                div()
                    .class(cls::TAG_ROLE_COLOR)
                    .style(format!("background-color: #{r:02x}{g:02x}{b:02x}")),
                role.name,
            )
        }
        Pick::Member(id) => {
            let member = env
                .state
                .members
                .by_guild
                .with_untracked(|all| all.get(&guild_id).and_then(|m| m.get(&id)).cloned())?;
            (
                member_avatar(env, &member.user).class(cls::TAG_AVATAR),
                display_name(&member),
            )
        }
    };
    let text = TextProps::styled(label, TextVariant::TEXT_MD_NORMAL)
        .color(TEXT_DEFAULT)
        .line_clamp(1_u32)
        .build();
    Some(
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::TAG.to_string()])
            .on_click(move || PICKED.with(|p| p.update(|list| list.retain(|x| *x != pick))))
            .build()
            .children([
                lead,
                div().class(cls::TAG_LABEL).child(text),
                components::icon(X_SMALL, TEXT_DEFAULT).class(cls::TAG_CLOSE),
            ]),
    )
}

fn picker_key(key: NamedKey, text: &str) -> bool {
    let count = ROWS.with(|rows| rows.borrow().len());
    match key {
        NamedKey::ArrowDown | NamedKey::ArrowUp => {
            if count > 0 {
                let next = match (SELECTED.with(Cell::get), key) {
                    (None, NamedKey::ArrowDown) => 0,
                    (None, _) => count - 1,
                    (Some(i), NamedKey::ArrowDown) => (i + 1) % count,
                    (Some(i), _) => (i + count - 1) % count,
                };
                select(next);
            }
            true
        }
        NamedKey::Enter => {
            let pick = SELECTED
                .with(Cell::get)
                .and_then(|i| ROWS.with(|rows| rows.borrow().get(i).and_then(|(pick, _)| *pick)));
            if let Some(pick) = pick {
                choose(pick);
            }
            true
        }
        NamedKey::Backspace if text.is_empty() && !PICKED.with(|p| p.peek()).is_empty() => {
            PICKED.with(|p| {
                p.update(|list| {
                    list.pop();
                })
            });
            true
        }
        _ => false,
    }
}

fn select(index: usize) {
    let previous = SELECTED.with(|s| s.replace(Some(index)));
    if previous == Some(index) {
        return;
    }
    ROWS.with(|rows| {
        let rows = rows.borrow();
        if let Some((_, flag)) = previous.and_then(|i| rows.get(i)) {
            flag.set(false);
        }
        if let Some((_, flag)) = rows.get(index) {
            flag.set(true);
        }
    });
}

fn deselect(index: usize) {
    if SELECTED.with(Cell::get) != Some(index) {
        return;
    }
    SELECTED.with(|s| s.set(None));
    ROWS.with(|rows| {
        if let Some((_, flag)) = rows.borrow().get(index) {
            flag.set(false);
        }
    });
}

fn choose(pick: Pick) {
    set_query("");
    PICKED.with(|p| {
        p.update(|list| {
            if let Some(index) = list.iter().position(|x| *x == pick) {
                list.remove(index);
            } else {
                list.push(pick);
            }
        })
    });
}

fn role_color(color: i64) -> skia_safe::Color {
    if color == 0 {
        colors::DEFAULT_ROLE
    } else {
        skia_safe::Color::from_rgb(
            ((color >> 16) & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            (color & 0xff) as u8,
        )
    }
}

fn rgb(color: skia_safe::Color) -> [u8; 3] {
    [color.r(), color.g(), color.b()]
}

fn display_name(member: &models::GuildMember) -> String {
    member.user.display_name(member.nick.as_deref())
}

fn user_tag(user: &models::User) -> String {
    match user.discriminator.as_str() {
        "" | "0" | "0000" => user.username.clone(),
        discriminator => format!("{}#{discriminator}", user.username),
    }
}

fn member_avatar(env: &HomeEnv, user: &models::User) -> Node {
    let mut props = ui::user_avatar_props(user.id, &user.discriminator, &env.fonts);
    if let Some(hash) = user.avatar.as_deref() {
        let url = env.backend.avatar_url(user.id, hash);
        let images = env.images.clone();
        let tick = env.tick.clone();
        props = props.bind_image(tick, move || images.get_or_decode_image(&url));
    }
    props.build()
}

fn picker_row(
    index: usize,
    leading: Node,
    label: Node,
    subtext: Option<Node>,
    trailing: Option<String>,
    on: bool,
    disabled: bool,
) -> Node {
    let mut body = div()
        .class(cls::ROW_BODY)
        .child(leading)
        .child(label.class(cls::ROW_LABEL));
    if let Some(subtext) = subtext {
        body = body.child(subtext.class(cls::ROW_SUBTEXT));
    }
    let mut check = checkbox_indicator(ToggleState::new(on)).class(cls::ROW_CHECK);
    if disabled {
        check = check.class(cls::ROW_CHECK_DISABLED);
    }
    let mut group = div().class(cls::ROW_GROUP).child(check).child(body);
    if disabled {
        group = group.class(cls::ROW_GROUP_DISABLED);
    }
    let flag = signal(SELECTED.with(Cell::get) == Some(index));
    let pick = ROWS.with(|rows| rows.borrow().get(index).and_then(|(pick, _)| *pick));
    ROWS.with(|rows| {
        if let Some(row) = rows.borrow_mut().get_mut(index) {
            row.1 = flag.clone();
        }
    });
    let mut row = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::PICKER_ROW.to_string()])
        .on_hover_change(move |hovered: bool| {
            if hovered {
                select(index);
            } else {
                deselect(index);
            }
        })
        .on_click(move || {
            if let Some(pick) = pick {
                choose(pick);
            }
        })
        .build()
        .bind_class(flag, cls::ROW_SELECTED, cls::ROW_UNSELECTED)
        .child(group);
    if let Some(trailing) = trailing {
        row = row.child(
            TextProps::styled(trailing, TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_MUTED)
                .build()
                .class(cls::ROW_TYPE),
        );
    }
    row
}

fn section_title(text: String) -> Node {
    div()
        .class(create_channel::SECTION_TITLE)
        .child(
            TextProps::styled(text, TextVariant::HEADING_SM_SEMIBOLD)
                .color(TEXT_DEFAULT)
                .build(),
        )
}

fn matches_query(text: &str, query: &str) -> bool {
    query.is_empty() || text.to_lowercase().contains(query)
}

fn picker_results(env: &HomeEnv) -> Node {
    let Some(guild_id) = GUILD.with(|g| g.get()) else {
        return div();
    };
    let raw = QUERY.with(Signal::get);
    let query = member_query(&raw).to_lowercase();
    let members_only = raw.trim().starts_with('@');
    let picked = PICKED.with(Signal::get);

    let all_roles = env.state.permissions.guild_roles(guild_id);
    let roles: Vec<_> = if members_only {
        Vec::new()
    } else {
        all_roles
            .iter()
            .filter(|role| {
                role.id.get() != guild_id.get()
                    && !role.managed
                    && !role
                        .permissions
                        .contains(Permissions::ADMINISTRATOR)
                    && matches_query(&role.name, &query)
            })
            .cloned()
            .collect()
    };
    let placeholder =
        !members_only && roles.is_empty() && raw.trim().is_empty() && all_roles.len() <= 1;

    let guild = env.state.guilds.find(guild_id);
    let me = env
        .state
        .users
        .current
        .with_untracked(|u| u.as_ref().map(|u| u.id));
    let mut members: Vec<_> = env.state.members.by_guild.with(|all| {
        all.get(&guild_id)
            .map(|m| m.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    });
    members.retain(|member| {
        let user = &member.user;
        Some(user.id) != guild.as_ref().and_then(|g| g.owner_id)
            && Some(user.id) != me
            && (matches_query(&display_name(member), &query)
                || matches_query(&user.username, &query)
                || matches_query(&user.discriminator, &query))
    });
    members.sort_by_key(|member| display_name(member).to_lowercase());

    let mut order: Vec<Option<Pick>> = Vec::new();
    if placeholder {
        order.push(None);
    }
    order.extend(roles.iter().map(|role| Some(Pick::Role(role.id))));
    order.extend(
        members
            .iter()
            .map(|member| Some(Pick::Member(member.user.id))),
    );
    ROWS.with(|rows| {
        *rows.borrow_mut() = order.iter().map(|pick| (*pick, signal(false))).collect()
    });
    if SELECTED.with(Cell::get).is_some_and(|i| i >= order.len()) {
        SELECTED.with(|s| s.set(None));
    }

    let mut list = div().class(cls::MEMBER_ROWS);
    let mut index = 0;
    if placeholder || !roles.is_empty() {
        list = list.child(section_title(
            requiem_i18n::t!("create-channel-roles").to_string(),
        ));
    }
    if placeholder {
        let shield = components::icon(SHIELD_USER, colors::DEFAULT_ROLE)
            .class(cls::ROLE_ICON);
        let label = TextProps::styled(
            requiem_i18n::t!("create-channel-no-roles").to_string(),
            TextVariant::TEXT_SM_MEDIUM,
        )
        .color(TEXT_MUTED)
        .line_clamp(1_u32)
        .build();
        list = list.child(picker_row(
            index,
            div().class(cls::ROLE_ICON_BOX).child(shield),
            label,
            None,
            None,
            true,
            true,
        ));
        index += 1;
    }
    for role in &roles {
        let on = picked.contains(&Pick::Role(role.id));
        let shield =
            components::icon(SHIELD_USER, role_color(role.color)).class(cls::ROLE_ICON);
        let label = TextProps::styled(role.name.clone(), TextVariant::TEXT_SM_MEDIUM)
            .color(TEXT_DEFAULT)
            .line_clamp(1_u32)
            .build();
        list = list.child(picker_row(
            index,
            div().class(cls::ROLE_ICON_BOX).child(shield),
            label,
            None,
            Some(requiem_i18n::t!("create-channel-row-role").to_string()),
            on,
            false,
        ));
        index += 1;
    }
    if !members.is_empty() {
        list = list.child(section_title(
            requiem_i18n::t!("message-list-members").to_string(),
        ));
    }
    for member in &members {
        let on = picked.contains(&Pick::Member(member.user.id));
        let avatar = member_avatar(env, &member.user).class(cls::ROW_AVATAR);
        let label = TextProps::styled(display_name(member), TextVariant::TEXT_SM_NORMAL)
            .color(TEXT_DEFAULT)
            .line_clamp(1_u32)
            .build();
        let subtext = TextProps::styled(user_tag(&member.user), TextVariant::TEXT_SM_NORMAL)
            .color(TEXT_MUTED)
            .line_clamp(1_u32)
            .build();
        list = list.child(picker_row(
            index,
            avatar,
            label,
            Some(subtext),
            None,
            on,
            false,
        ));
        index += 1;
    }
    list
}

fn submit_name(env: &HomeEnv, text: String) {
    if BUSY.with(|b| b.peek()) || text.trim().is_empty() {
        return;
    }
    set_name(&text);
    if PRIVATE.with(Signal::get) {
        slides().go(SLIDE_MEMBERS);
    } else {
        create(env);
    }
}

fn create(env: &HomeEnv) {
    if BUSY.with(|b| b.peek()) {
        return;
    }
    let name = name_text().trim().to_string();
    if name.is_empty() {
        return;
    }
    let Some(guild) = GUILD.with(|g| g.get()) else {
        return;
    };
    let kind = KIND.with(Signal::get);

    let parent = if kind == KIND_CATEGORY {
        None
    } else {
        PARENT.with(|p| p.get())
    };
    let overwrites = if PRIVATE.with(Signal::get) {
        private_overwrites(env, guild, kind, &PICKED.with(|p| p.peek()))
    } else {
        Vec::new()
    };
    ERROR.with(|e| e.set(None));
    BUSY.with(|b| b.set(true));

    let (http, inbox) = (env.http.clone(), env.state.nav.inbox());
    env.queries.mutate(Vec::new(), move || async move {
        let result = channels::create_guild_channel(
            &http,
            guild,
            &name,
            kind,
            parent,
            &overwrites,
        )
        .await;

        let (channel_id, error) = match result {
            Ok(channel) => ((channel.kind != KIND_CATEGORY).then_some(channel.id), None),
            Err(message) => (None, Some(message)),
        };
        inbox
            .lock()
            .expect("nav inbox poisoned")
            .push(NavEvent::ChannelCreated { channel_id, error });
        Ok::<(), RestError>(())
    });
}

fn private_overwrites(
    env: &HomeEnv,
    guild: Id<GuildMarker>,
    kind: u8,
    picks: &[Pick],
) -> Vec<PermissionOverwrite> {
    let bits = match kind {
        KIND_VOICE | KIND_CATEGORY => Permissions::VIEW_CHANNEL | Permissions::CONNECT,
        _ => Permissions::VIEW_CHANNEL,
    };
    let grant = |id: u64, kind: OverwriteKind| PermissionOverwrite {
        id,
        kind,
        allow: bits,
        deny: Permissions::NONE,
    };
    let mut overwrites = vec![PermissionOverwrite {
        id: guild.get(),
        kind: OverwriteKind::Role,
        allow: Permissions::NONE,
        deny: bits,
    }];
    overwrites.extend(picks.iter().map(|pick| match pick {
        Pick::Role(id) => grant(id.get(), OverwriteKind::Role),
        Pick::Member(id) => grant(id.get(), OverwriteKind::Member),
    }));
    let me = env
        .state
        .users
        .current
        .with_untracked(|u| u.as_ref().map(|u| u.id));
    let guild_info = env.state.guilds.find(guild);
    let owner = me.is_some() && guild_info.as_ref().and_then(|g| g.owner_id) == me;
    let admin = guild_info.as_ref().is_some_and(|g| {
        env.state
            .permissions
            .guild_permissions(g)
            .contains(Permissions::ADMINISTRATOR)
    });
    let mine = env.state.permissions.my_role_ids(guild);
    let covered = picks
        .iter()
        .any(|pick| matches!(pick, Pick::Role(id) if mine.contains(id)));
    if let (Some(me), false) = (me, covered || admin || owner) {
        overwrites.push(grant(me.get(), OverwriteKind::Member));
    }
    overwrites
}

pub fn mount(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: Rc<dyn Fn() -> HomeEnv>,
    insert_text: Rc<dyn Fn(&'static str, String)>,
) -> Vec<Effect> {
    INSERT_TEXT.with(|slot| *slot.borrow_mut() = Some(insert_text));
    let dialog = section(tree, parent, move || {
        if DIALOG.with(Dialog::is_mounted) {
            dialog(&env())
        } else {
            div()
        }
    });
    vec![dialog, slides().effect(tree)]
}
