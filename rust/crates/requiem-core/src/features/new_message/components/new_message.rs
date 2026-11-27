use crate::models;
use crate::rest::endpoints::channels;
use crate::ui::classes::new_message as cls;
use crate::ui;
use requiem_ui::components::TextVariant;
use requiem_ui::components::scroll_view;
use requiem_ui::components::{ButtonVariant, ModalSize};
use requiem_ui::widget::anim::ToggleState;
use std::rc::Rc;

use requiem_ui::components::{
    AvatarStatusSize, ButtonProps, ModalProps, TextInputProps, TextProps, avatar_with_status,
    checkbox_indicator, div,
};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};

use requiem_ui::widget::{WidgetId, WidgetTree};

use crate::models::Id;
use crate::models::id::marker::UserMarker;
use crate::rest::RestError;
use crate::state::NavEvent;
use crate::ui::dialog::Dialog;
use crate::ui::home::{HomeEnv, user_name};
use crate::ui::{TEXT_STRONG, TEXT_SUBTLE};

const MAX_OTHERS: usize = 24;

thread_local! {
    static DIALOG: Dialog = Dialog::new();
    static QUERY: Signal<String> = signal(String::new());
    static PICKED: Signal<Vec<Id<UserMarker>>> = signal(Vec::new());
}

pub fn is_open() -> bool {
    DIALOG.with(Dialog::is_open)
}

pub fn open() {
    QUERY.with(|q| q.set(String::new()));
    PICKED.with(|p| p.set(Vec::new()));
    DIALOG.with(Dialog::open);
}

pub fn close() {
    DIALOG.with(Dialog::close);
}

fn toggle(id: Id<UserMarker>) {
    PICKED.with(|picked| {
        picked.update(|list| {
            if let Some(i) = list.iter().position(|p| *p == id) {
                list.remove(i);
            } else if list.len() < MAX_OTHERS {
                list.push(id);
            }
        })
    });
}

fn create(env: &HomeEnv) {
    let picked = PICKED.with(|p| p.peek());
    if picked.is_empty() {
        return;
    }
    close();

    if let [only] = picked[..] {
        if let Some(dm) = env.state.dms.with_user(only) {
            (env.on_select_channel)(dm.id);
            return;
        }
    }
    let (http, inbox) = (env.http.clone(), env.state.nav.inbox());
    env.queries.mutate(Vec::new(), move || async move {
        let channel = match picked[..] {
            [only] => {
                channels::create_dm(&http, only, Some("New Group DM"))
                    .await?
            }
            _ => {
                channels::create_group_dm(
                    &http,
                    &picked,
                    Some("New Group DM"),
                )
                .await?
            }
        };
        inbox
            .lock()
            .expect("nav inbox poisoned")
            .push(NavEvent::DmOpened {
                channel_id: channel.id,
                recipients: picked,
            });
        Ok::<(), RestError>(())
    });
}

fn dialog(env: &HomeEnv) -> Node {
    let toggle_state = DIALOG.with(Dialog::toggle);

    let field = TextInputProps::new()
        .placeholder(requiem_i18n::t!("new-message-search"))
        .helper_text(requiem_i18n::t!(
            "new-message-add-friends-or-server-members-to"
        ))
        .autofocus(true)
        .on_change(|text: &str| QUERY.with(|q| q.set(text.to_string())))
        .build();

    let list_env = env.clone();
    let list = Node::reactive(move || friend_list(&list_env)).class(cls::LIST_SLOT);
    let body = div().class(cls::BODY).children([field, list]);

    let cancel = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .label(requiem_i18n::t!("new-message-cancel").to_string())
        .on_click(close)
        .build();
    let create_env = env.clone();
    let create_button = Node::reactive(move || {
        let empty = PICKED.with(Signal::get).is_empty();
        let env = create_env.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .label(requiem_i18n::t!("new-message-create-message").to_string())
            .disabled(empty)
            .on_click(move || create(&env))
            .build()
    })
    .class(cls::CREATE_SLOT);

    ModalProps::default()
        .size(ModalSize::Medium)
        .heading(requiem_i18n::t!("new-message-new-message"))
        .subtitle(requiem_i18n::t!("new-message-group-dms-can-have-up-to").to_string())
        .dismissable(true)
        .on_dismiss(close)
        .open(toggle_state)
        .body(body)
        .footer(vec![cancel, create_button])
        .panel_class(cls::PANEL.to_string())
        .build()
}

fn friend_list(env: &HomeEnv) -> Node {
    let _ = env.tick.get();
    let query = QUERY.with(Signal::get).trim().to_lowercase();
    let picked = PICKED.with(Signal::get);
    let fonts = &env.fonts;

    let mut friends: Vec<(models::User, String)> = env
        .state
        .relationships
        .list
        .get()
        .iter()
        .filter(|r| r.is_friend())
        .map(|r| (r.user.clone(), user_name(&env.state, &r.user)))
        .filter(|(user, name)| {
            query.is_empty()
                || name.to_lowercase().contains(&query)
                || user.username.to_lowercase().contains(&query)
        })
        .collect();
    friends.sort_by_key(|(_, name)| name.to_lowercase());

    let mut list = scroll_view().class(cls::LIST);
    for (user, name) in friends {
        let id = user.id;
        let on = picked.contains(&id);
        let mut props = ui::user_avatar_props(id, &user.discriminator, fonts);
        if let Some(image) = user.avatar.as_deref().and_then(|hash| {
            env.images
                .get_or_decode_image(&env.backend.avatar_url(id, hash))
        }) {
            props = props.image(image);
        }
        let avatar = avatar_with_status(props, AvatarStatusSize::Px32, None).class(cls::AVATAR);
        let texts = div().class(cls::TEXTS).children([
            TextProps::styled(name, TextVariant::TEXT_MD_SEMIBOLD)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
            TextProps::styled(user.username.clone(), TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_SUBTLE)
                .line_clamp(1_u32)
                .build(),
        ]);
        let mut row = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::ROW.to_string()])
            .on_click(move || toggle(id))
            .build()
            .children([
                avatar,
                texts,
                div().class(cls::SPACER),
                checkbox_indicator(ToggleState::new(on)),
            ]);
        if on {
            row = row.class(cls::ROW_ON);
        }
        list = list.child(row);
    }
    list
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId, env: Rc<dyn Fn() -> HomeEnv>) -> Effect {
    section(tree, parent, move || {
        if DIALOG.with(Dialog::is_mounted) {
            dialog(&env())
        } else {
            div()
        }
    })
}
