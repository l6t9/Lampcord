use crate::ui::classes::account_popout as cls;
use crate::ui::profile_card::CardContainer;
use crate::ui::profile_card;
use crate::ui::profile_decorations;
use crate::ui::profile_theme;
use crate::ui::profile_popout::ProfileSurface;
use crate::ui::profile_popout;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Align, Padding, StackProps};
use requiem_ui::components::{menu, popover_backdrop, status_indicator};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::components::{
    ButtonProps, MenuItem, PopoverProps, TextProps, TooltipProps, TooltipState, div, icon,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::timer::{TimeoutHandle, clear_timeout, set_timeout};
use requiem_ui::widget::primitives::HoverFlag;

use crate::presence::CustomStatus;
use crate::ui::custom_status::{self, BubbleActions, CustomStatusModalState};

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::presence::OwnStatus;
use crate::query::QueryClient;
use crate::rest::HttpClient;
use crate::state::AppState;
use crate::ui::{Fonts, TEXT_SECONDARY};
use requiem_ui::icons::{CHEVRON_SMALL_RIGHT, COPY, PENCIL, USER_CIRCLE};

const HOVER_LINGER: Duration = Duration::from_millis(500);

#[derive(Clone)]
pub struct AccountPopoutState {
    pub open: Signal<bool>,
    pub status_menu_open: Signal<bool>,
    pub panel_hover: HoverFlag,

    pub on_set_status: Rc<dyn Fn(OwnStatus)>,

    pub on_edit_profile: Rc<dyn Fn()>,

    pub on_set_custom_status: Rc<dyn Fn(Option<CustomStatus>)>,

    pub on_view_full_profile: Rc<dyn Fn()>,

    pub custom_status: CustomStatusModalState,

    pub prompt: Signal<&'static str>,

    pub hovered: Signal<bool>,
    hover_timer: Rc<Cell<Option<TimeoutHandle>>>,

    pub copy_tooltip: TooltipState,
}

impl Default for AccountPopoutState {
    fn default() -> Self {
        Self {
            open: signal(false),
            status_menu_open: signal(false),
            panel_hover: HoverFlag::new(),
            on_set_status: Rc::new(|_| {}),
            on_edit_profile: Rc::new(|| {}),
            on_set_custom_status: Rc::new(|_| {}),
            on_view_full_profile: Rc::new(|| {}),
            custom_status: CustomStatusModalState::new(),
            prompt: signal(custom_status::PROMPTS[0]),
            hovered: signal(false),
            hover_timer: Rc::new(Cell::new(None)),
            copy_tooltip: TooltipState::new(),
        }
    }
}

impl AccountPopoutState {
    pub fn close(&self) {
        self.panel_hover.set(false);
        if self.status_menu_open.peek() {
            self.status_menu_open.set(false);
        }
        if self.open.peek() {
            self.open.set(false);
        }
    }

    pub fn toggle(&self) {
        if self.open.peek() {
            self.close();
        } else {
            self.prompt.set(custom_status::random_prompt());
            self.open.set(true);
        }
    }

    fn set_hovered(&self, inside: bool) {
        if let Some(handle) = self.hover_timer.take() {
            clear_timeout(handle);
        }
        if inside {
            if !self.hovered.peek() {
                self.hovered.set(true);
            }
        } else {
            let hovered = self.hovered.clone();
            self.hover_timer
                .set(Some(set_timeout(HOVER_LINGER, move || hovered.set(false))));
        }
    }

    pub fn open_custom_status(&self, current: Option<&CustomStatus>) {
        self.close();
        self.custom_status.open_with(current);
    }
}

pub fn build(
    state: &AppState,
    ui: &AccountPopoutState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) -> Node {
    let layer = div().class(cls::LAYER);
    if !ui.open.get() {
        return layer;
    }
    let Some(user) = state.users.current.get() else {
        return layer;
    };
    let layer = layer.class(cls::LAYER_OPEN);
    let status = state.presence.status.get();
    let status_menu_open = ui.status_menu_open.get();

    let close = {
        let ui = ui.clone();
        Rc::new(move || ui.close()) as Rc<dyn Fn()>
    };

    let user_id = user.id;
    let profile = profile_popout::load_profile(
        queries,
        http,
        &state.users,
        user_id,
        None,
        ProfileSurface::Plain,
    );
    let banner_image = profile
        .as_ref()
        .and_then(|p| p.user.banner.as_deref())
        .and_then(|hash| images.get_or_decode_image(&backend.banner_url(user_id, hash)));
    let banner_fallback = profile_popout::banner_color(
        user_id,
        &user.discriminator,
        profile.as_ref().and_then(|p| p.user.accent_color),
        user.avatar.as_deref(),
        images,
        backend,
    );
    let avatar_image = user
        .avatar
        .as_deref()
        .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(user.id, hash)));
    let avatar_decoration = profile
        .as_ref()
        .and_then(|p| p.user.avatar_decoration_asset())
        .or_else(|| user.avatar_decoration_asset())
        .and_then(|asset| images.get_or_decode_image(&backend.avatar_decoration_url(asset, 80)));
    let colors = profile
        .as_deref()
        .map(profile_theme::resolve)
        .unwrap_or_default();
    let view_full_profile = {
        let (ui, on_view) = (ui.clone(), ui.on_view_full_profile.clone());
        Rc::new(move || {
            ui.close();
            on_view();
        }) as Rc<dyn Fn()>
    };

    let custom = state.presence.custom_status.get();
    let actions = BubbleActions {
        on_open: {
            let (ui, custom) = (ui.clone(), custom.clone());
            Rc::new(move || ui.open_custom_status(custom.as_ref()))
        },
        on_clear: {
            let on_set = ui.on_set_custom_status.clone();
            Rc::new(move || on_set(None))
        },
    };
    let prompt = ui.prompt.peek();

    let display_name = user
        .global_name
        .clone()
        .unwrap_or_else(|| user.username.clone());
    let username = user.username.clone();
    let copy_tooltip = ui.copy_tooltip.clone();
    let copy = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::COPY_BTN.to_string()])
        .on_click(move || copy_tooltip.copy(username.clone()))
        .build()
        .child(icon(COPY, TEXT_SECONDARY).class(cls::COPY_ICON));
    let copy = TooltipProps::default()
        .text(requiem_i18n::t!("account-popout-copy-username").to_string())
        .state(ui.copy_tooltip.clone())
        .build(copy);
    let copy = div()
        .class(cls::COPY_REVEAL)
        .bind_class(ui.hovered.clone(), cls::COPY_SHOWN, cls::COPY_HIDDEN)
        .child(copy);

    let edit_profile = {
        let (on_edit, close, ui) = (ui.on_edit_profile.clone(), close.clone(), ui.clone());
        menu_row(
            icon(PENCIL, TEXT_SECONDARY).class(cls::ITEM_ICON),
            &requiem_i18n::t!("account-popout-edit-profile"),
            None,
            move || {
                close();
                on_edit();
            },
        )
        .hover(move |hovered| {
            if hovered && ui.status_menu_open.peek() {
                ui.status_menu_open.set(false);
            }
        })
    };
    let status_row = {
        let (open_menu, toggle_menu) = (ui.status_menu_open.clone(), ui.status_menu_open.clone());
        let dot = div()
            .class(cls::ITEM_ICON)
            .child(status_indicator(status.indicator()).class(cls::STATUS_ICON));
        menu_row(
            dot,
            &status.label(),
            Some(icon(CHEVRON_SMALL_RIGHT, TEXT_SECONDARY).class(cls::CARET)),
            move || toggle_menu.set(!toggle_menu.peek()),
        )
        .hover(move |hovered| {
            if hovered && !open_menu.peek() {
                open_menu.set(true);
            }
        })
        .selected(status_menu_open)
    };
    let card_status = div()
        .class(cls::CARD)
        .child(edit_profile.build(true))
        .child(status_row.build(false));

    let switch_accounts = {
        let ui = ui.clone();
        menu_row(
            icon(USER_CIRCLE, TEXT_SECONDARY).class(cls::ITEM_ICON),
            &requiem_i18n::t!("account-popout-switch-accounts"),
            Some(icon(CHEVRON_SMALL_RIGHT, TEXT_SECONDARY).class(cls::CARET)),
            || {},
        )
        .hover(move |hovered| {
            if hovered && ui.status_menu_open.peek() {
                ui.status_menu_open.set(false);
            }
        })
    };
    let card_accounts = div().class(cls::CARD).child(switch_accounts.build(false));

    let pronouns = profile
        .as_deref()
        .and_then(|p| p.user_profile.as_ref())
        .map(|m| m.pronouns.clone())
        .unwrap_or_default();
    let bio = profile
        .as_deref()
        .map(|p| p.user.bio.clone())
        .unwrap_or_default();
    let view = profile_card::ProfileCardView {
        user_id: user.id,
        discriminator: &user.discriminator,
        username: &user.username,
        display_name: &display_name,
        pronouns: &pronouns,
        bio: &bio,
        avatar: avatar_image,
        avatar_decoration,
        banner_image,
        banner_color: banner_fallback,
        theme: colors.theme,
        name_color: colors.name_color,
        status: Some(status.indicator()),
        bot: false,
        badges_from: profile.as_deref(),
    };
    let menus = StackProps::vertical()
        .gap(12)
        .full_width(false)
        .build()
        .child(card_status)
        .child(card_accounts);
    let header_layers = profile
        .as_deref()
        .map(|profile| profile_decorations::layers(queries, http, profile, images, backend))
        .unwrap_or_default();
    let nameplate_asset = profile
        .as_ref()
        .and_then(|profile| profile.user.nameplate_asset())
        .or_else(|| user.nameplate_asset());
    let background_layers = profile_decorations::nameplate_node(
        nameplate_asset,
        images,
        backend,
        crate::ui::classes::profile_popout::NAMEPLATE_BACKDROP,
    )
    .into_iter()
    .collect();
    let parts = profile_card::CardParts {
        open_profile: Some(view_full_profile.clone()),
        name_suffix: vec![copy],
        header_layers,
        background_layers,
        after_bio: vec![menus],
        roomy_body: true,
        ..Default::default()
    };
    let hover_ui = ui.clone();
    let panel = PopoverProps::default()
        .on_hover_change(move |inside| hover_ui.set_hovered(inside))
        .build()
        .class(cls::PANEL);
    let panel = profile_card::profile_card(
        view,
        profile_card::CardStatus {
            status: custom.as_ref(),
            actions: Some(&actions),
            prompt,
        },
        parts,
        None,
        CardContainer::Panel(panel),
        fonts,
        images,
        backend,
    );

    let mut layer = layer.child(popover_backdrop(move || close())).child(panel);
    if status_menu_open {
        layer = layer.child(status_menu(ui).class(cls::STATUS_MENU));
    }
    layer
}

fn status_menu(ui: &AccountPopoutState) -> Node {
    let mut items = Vec::new();
    for status in OwnStatus::ALL {
        let (ui, on_set) = (ui.clone(), ui.on_set_status.clone());
        let dot = status_indicator(status.indicator()).class(cls::MENU_DOT);
        let mut item = MenuItem::action(status.label(), move || {
            on_set(status);
            ui.close();
        })
        .with_leading(dot);
        if let Some(description) = status.description() {
            item = item.with_subtitle(description);
        }
        items.push(item);

        if status == OwnStatus::Online {
            items.push(MenuItem::Separator);
        }
    }
    menu(items)
}

struct MenuRow {
    icon: Node,
    label: String,
    trailing: Option<Node>,
    on_click: Box<dyn FnMut()>,
    on_hover: Option<Box<dyn FnMut(bool)>>,
    selected: bool,
}

fn menu_row(
    icon: Node,
    label: &str,
    trailing: Option<Node>,
    on_click: impl FnMut() + 'static,
) -> MenuRow {
    MenuRow {
        icon,
        label: label.to_string(),
        trailing,
        on_click: Box::new(on_click),
        on_hover: None,
        selected: false,
    }
}

impl MenuRow {
    fn hover(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_hover = Some(Box::new(f));
        self
    }

    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn build(self, divider: bool) -> Node {
        let label = TextProps::styled(self.label, TextVariant::TEXT_SM_MEDIUM)
            .line_height(16.0_f32)
            .color(TEXT_SECONDARY)
            .line_clamp(1_u32)
            .build()
            .class(cls::ITEM_LABEL);
        let mut content = StackProps::horizontal()
            .align(Align::Center)
            .padding(Padding::all(8))
            .build()
            .child(self.icon)
            .child(label);
        if let Some(trailing) = self.trailing {
            content = content.child(div().class(cls::CARET_WRAP).child(trailing));
        }
        let on_click = self.on_click;
        let mut props = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::ITEM.to_string()])
            .on_click(on_click);
        if let Some(on_hover) = self.on_hover {
            props = props.on_hover_change(on_hover);
        }
        let mut item = props.build().child(content);
        if self.selected {
            item = item.class(cls::ITEM_SELECTED);
        }
        let mut wrap = StackProps::vertical()
            .gap(0)
            .full_width(false)
            .build()
            .child(item);
        if divider {
            wrap = wrap.child(div().class(cls::DIVIDER));
        }
        wrap
    }
}
