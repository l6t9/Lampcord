use crate::models::id::marker;
use crate::rest::endpoints::guilds;
use crate::rest::endpoints::invites;
use crate::ui::classes::create_guild;
use crate::ui::classes::create_guild as cls;
use crate::ui;
use requiem_ui::colors::TEXT_MUTED;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{ButtonVariant, ModalSize};
use requiem_ui::components::{Justify, Padding, StackProps};
use requiem_ui::icons::Icon;
use requiem_ui::style;
use requiem_ui::timer;
use skia_safe::Paint;
use skia_safe::PaintStyle;
use skia_safe::Path;
use std::cell::{Cell, RefCell};
use std::f32::consts;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use requiem_ui::components::{
    ButtonProps, FormItemProps, MessageTextProps, ModalProps, TextInputProps, TextProps, div,
    field_error, icon, message_text, scroll_view,
};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, create_effect, signal};
use requiem_ui::widget::{AbsoluteRect, Widget, WidgetId, WidgetTree};
use skia_safe::font_style::Weight;
use skia_safe::{Color, Image};

use crate::models::Id;
use crate::models::id::marker::GuildMarker;
use crate::rest::RestError;
use crate::rest::endpoints::guild_templates::{self, GuildTemplate};
use crate::state::{AppState, NavEvent};
use crate::ui::dialog::Dialog;
use crate::ui::home::HomeEnv;
use crate::ui::profile_image::prepare_upload;
use crate::ui::{ImageTarget, PendingUpload, TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::icons::{CHEVRON_LARGE_RIGHT, X_SMALL};

const STEP_WIDTH: f32 = 438.0;

const BODY_PADDING: f32 = 24.0;

const BODY_INSET: f32 = 24.0;

const MASS: f32 = 1.0;
const TENSION: f32 = 300.0;
const FRICTION: f32 = 28.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Templates,
    Intent,
    Customize,
    Join,
}

impl Step {
    fn index(self) -> usize {
        self as usize
    }
}

struct Anim {
    to: Step,
    dir: f32,
    from_height: f32,

    to_height: f32,
    pos: f32,
    vel: f32,
    last: Instant,
}

thread_local! {
    static DIALOG: Dialog = Dialog::new();

    static CURRENT: Cell<Step> = const { Cell::new(Step::Templates) };

    static SLOTS: [Signal<Option<Step>>; 2] = [signal(Some(Step::Templates)), signal(None)];
    static SLOT_STYLES: [Signal<String>; 2] = [signal(String::new()), signal(String::new())];
    static HOST_STYLE: Signal<String> = signal(String::new());

    static SIZES: [Signal<(f32, f32)>; 2] = [signal((0.0, 0.0)), signal((0.0, 0.0))];

    static TARGET_SLOT: Signal<usize> = signal(0);

    static HOST_HEIGHT: Cell<f32> = const { Cell::new(0.0) };
    static ACTIVE_SLOT: Cell<usize> = const { Cell::new(0) };
    static ANIM: RefCell<Option<Anim>> = const { RefCell::new(None) };

    static NAME: RefCell<Option<String>> = const { RefCell::new(None) };
    static ICON: Signal<Option<PendingUpload>> = signal(None);

    static TEMPLATE: Cell<GuildTemplate> = const { Cell::new(guild_templates::CREATE) };
    static INVITE: RefCell<String> = const { RefCell::new(String::new()) };
    static ERROR: Signal<Option<String>> = signal(None);
    static BUSY: Signal<bool> = signal(false);

    static PICK_IMAGE: RefCell<Option<Rc<dyn Fn() -> Option<PathBuf>>>> = const { RefCell::new(None) };
}

pub fn install_image_picker(pick: Rc<dyn Fn() -> Option<PathBuf>>) {
    PICK_IMAGE.with(|p| *p.borrow_mut() = Some(pick));
}

pub fn is_open() -> bool {
    DIALOG.with(Dialog::is_open)
}

fn translate_style(x: f32) -> String {
    format!("left: {}px", x - BODY_INSET)
}

fn set_host_height(step_height: f32) {
    HOST_HEIGHT.with(|h| h.set(step_height));
    HOST_STYLE.with(|h| h.set(format!("height: {}px", step_height - BODY_PADDING)));
}

fn follow_measured_height(_tree: &mut WidgetTree) {
    let slot = TARGET_SLOT.with(Signal::get);
    let (_, height) = SIZES.with(|sizes| sizes[slot].get());
    if height <= 0.0 {
        return;
    }
    let sliding = ANIM.with(|a| {
        let mut anim = a.borrow_mut();
        anim.as_mut().map(|anim| anim.to_height = height).is_some()
    });
    if !sliding {
        set_host_height(height);
    }
}

fn set_error(message: Option<String>) {
    ERROR.with(|e| e.set(message));
}

pub fn open(step: Step) {
    CURRENT.with(|c| c.set(step));
    ANIM.with(|a| *a.borrow_mut() = None);
    ACTIVE_SLOT.with(|a| a.set(0));
    SLOTS.with(|slots| {
        slots[0].set(Some(step));
        slots[1].set(None);
    });
    SLOT_STYLES.with(|styles| styles.iter().for_each(|s| s.set(translate_style(0.0))));
    SIZES.with(|sizes| sizes.iter().for_each(|s| s.set((0.0, 0.0))));
    TARGET_SLOT.with(|t| t.set(0));
    HOST_HEIGHT.with(|h| h.set(0.0));
    HOST_STYLE.with(|h| h.set("height: 0px".to_string()));
    NAME.with(|n| *n.borrow_mut() = None);
    TEMPLATE.with(|t| t.set(guild_templates::CREATE));
    INVITE.with(|i| i.borrow_mut().clear());
    ICON.with(|i| i.set(None));
    set_error(None);
    BUSY.with(|b| b.set(false));
    DIALOG.with(Dialog::open);
}

pub fn close() {
    DIALOG.with(Dialog::close);
}

fn go_to(step: Step) {
    if ANIM.with(|a| a.borrow().is_some()) || CURRENT.with(Cell::get) == step {
        return;
    }
    let from = CURRENT.with(Cell::get);
    let from_height = HOST_HEIGHT.with(Cell::get);
    set_error(None);
    let (leaving, entering) = (ACTIVE_SLOT.with(Cell::get), 1 - ACTIVE_SLOT.with(Cell::get));
    let dir = if step.index() > from.index() {
        1.0
    } else {
        -1.0
    };
    SLOTS.with(|slots| slots[entering].set(Some(step)));
    SLOT_STYLES.with(|styles| {
        styles[leaving].set(translate_style(0.0));
        styles[entering].set(translate_style(dir * STEP_WIDTH));
    });
    TARGET_SLOT.with(|t| t.set(entering));
    ANIM.with(|a| {
        *a.borrow_mut() = Some(Anim {
            to: step,
            dir,
            from_height,
            to_height: from_height,
            pos: 0.0,
            vel: 0.0,
            last: Instant::now(),
        })
    });
    timer::request_animation_frame(tick);
}

fn tick() {
    let finished = ANIM.with(|anim| {
        let mut anim = anim.borrow_mut();
        let a = anim.as_mut()?;
        let now = Instant::now();
        let mut remaining = now.duration_since(a.last).as_secs_f32().min(0.1);
        a.last = now;
        while remaining > 0.0 {
            let dt = remaining.min(0.004);
            a.vel += (-TENSION * (a.pos - 1.0) - FRICTION * a.vel) / MASS * dt;
            a.pos += a.vel * dt;
            remaining -= dt;

            if a.pos >= 1.0 {
                a.pos = 1.0;
                break;
            }
        }
        let (leaving, entering) = (ACTIVE_SLOT.with(Cell::get), 1 - ACTIVE_SLOT.with(Cell::get));
        SLOT_STYLES.with(|styles| {
            styles[leaving].set(translate_style(-a.dir * STEP_WIDTH * a.pos));
            styles[entering].set(translate_style(a.dir * STEP_WIDTH * (1.0 - a.pos)));
        });
        set_host_height(a.from_height + (a.to_height - a.from_height) * a.pos);
        (a.pos >= 1.0).then_some(a.to)
    });
    match finished {
        Some(to) => {
            let leaving = ACTIVE_SLOT.with(Cell::get);
            SLOTS.with(|slots| slots[leaving].set(None));
            ACTIVE_SLOT.with(|a| a.set(1 - leaving));
            CURRENT.with(|c| c.set(to));
            ANIM.with(|a| *a.borrow_mut() = None);
            let slot = TARGET_SLOT.with(Signal::peek);
            let height = SIZES.with(|sizes| sizes[slot].peek().1);
            if height > 0.0 {
                set_host_height(height);
            }
        }
        None => {
            if ANIM.with(|a| a.borrow().is_some()) {
                timer::request_animation_frame(tick);
            }
        }
    }
}

pub fn finish(state: &AppState, guild_id: Option<Id<GuildMarker>>, error: Option<String>) {
    BUSY.with(|b| b.set(false));
    match error {
        Some(message) => set_error(Some(message)),
        None => {
            close();
            state.nav.pending_guild.set(guild_id);
        }
    }
}

pub fn select_pending(
    state: &AppState,
    on_select: &dyn Fn(Id<marker::ChannelMarker>),
) {
    let Some(guild_id) = state.nav.pending_guild.get() else {
        return;
    };
    if state.guilds.find(guild_id).is_none() {
        return;
    }
    state.nav.pending_guild.set(None);
    if let Some(channel) = ui::default_channel(state, Some(guild_id)) {
        on_select(channel.id);
    }
}

#[derive(Clone, Copy)]
struct ArtRef {
    name: &'static str,
    px: u32,
}

macro_rules! art {
    ($name:literal, $px:literal) => {
        ArtRef {
            name: concat!("create_guild/", $name),
            px: $px,
        }
    };
}

struct Art {
    source: ArtRef,
    image: RefCell<Option<Option<Image>>>,
}

impl Widget for Art {
    fn paint(
        &self,
        canvas: &skia_safe::Canvas,
        rect: AbsoluteRect,
        _style: &style::ComputedStyle,
    ) {
        let mut slot = self.image.borrow_mut();
        let image = slot.get_or_insert_with(|| {
            ui::asset_svg(self.source.name, self.source.px, self.source.px)
        });
        let Some(image) = image else { return };
        let (iw, ih) = (image.width() as f32, image.height() as f32);
        let scale = (rect.width / iw).min(rect.height / ih);
        let (w, h) = (iw * scale, ih * scale);
        let dst = skia_safe::Rect::from_xywh(
            rect.x + (rect.width - w) / 2.0,
            rect.y + (rect.height - h) / 2.0,
            w,
            h,
        );
        let sampling = skia_safe::SamplingOptions::new(
            skia_safe::FilterMode::Linear,
            skia_safe::MipmapMode::Linear,
        );
        canvas.draw_image_rect_with_sampling_options(
            image,
            None,
            dst,
            sampling,
            &Paint::default(),
        );
    }
}

fn art(source: ArtRef) -> Node {
    Node::new(
        "div",
        Art {
            source,
            image: RefCell::new(None),
        },
    )
}

struct UploadCircle;

impl Widget for UploadCircle {
    fn paint(
        &self,
        canvas: &skia_safe::Canvas,
        rect: AbsoluteRect,
        _style: &style::ComputedStyle,
    ) {
        let scale = rect.width / 80.0;

        let ink = Color::from_rgb(0xab, 0xac, 0xb2);
        canvas.save();
        canvas.translate((rect.x, rect.y));
        canvas.scale((scale, scale));
        if let Some(upload) = ICON.with(|i| i.peek()) {
            canvas.save();
            canvas.clip_path(
                &Path::circle((40.0, 40.0), 40.0, None),
                skia_safe::ClipOp::Intersect,
                true,
            );
            let sampling = skia_safe::SamplingOptions::new(
                skia_safe::FilterMode::Linear,
                skia_safe::MipmapMode::Linear,
            );
            canvas.draw_image_rect_with_sampling_options(
                &upload.preview,
                None,
                skia_safe::Rect::from_wh(80.0, 80.0),
                sampling,
                &Paint::default(),
            );
            canvas.restore();
        } else {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(2.0);
            paint.set_color(ink);

            let radius = 39.0_f32;
            let half = 4.0 / radius;
            for i in 0..20 {
                let center = -consts::FRAC_PI_2 + i as f32 * consts::TAU / 20.0;
                let oval = skia_safe::Rect::from_xywh(
                    40.0 - radius,
                    40.0 - radius,
                    2.0 * radius,
                    2.0 * radius,
                );
                canvas.draw_arc(
                    oval,
                    (center - half).to_degrees(),
                    (2.0 * half).to_degrees(),
                    false,
                    &paint,
                );
            }
            paint.set_style(PaintStyle::Fill);

            let path =
                |shape: &'static str| Path::from_svg(Icon::Named(shape).path()).unwrap_or_default();

            canvas.draw_path(&path("shape/upload_camera_lens"), &paint);
            canvas.draw_path(&path("shape/upload_camera_body"), &paint);
        }

        let mut badge = Paint::default();
        badge.set_anti_alias(true);
        badge.set_color(Color::from_rgb(0x58, 0x65, 0xf2));
        canvas.draw_circle((68.0, 12.0), 12.0, &badge);
        badge.set_color(Color::WHITE);
        canvas.draw_path(
            &Path::from_svg(Icon::Named("shape/upload_plus_badge").path()).unwrap_or_default(),
            &badge,
        );
        canvas.restore();
    }
}

fn line_height(size: f32) -> f32 {
    match size as u32 {
        24 => 30.0,
        20 => 24.0,
        16 => 20.0,
        14 => 18.0,
        _ => 16.0,
    }
}

fn close_button() -> Node {
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::CLOSE.to_string()])
        .on_click(close)
        .build()
        .child(icon(X_SMALL, TEXT_SECONDARY).class(cls::CLOSE_ICON))
}

fn text_button(label: &str, on_click: impl FnMut() + 'static) -> Node {
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::TEXT_BUTTON.to_string()])
        .on_click(on_click)
        .build()
        .child(
            TextProps::styled(label.to_string(), TextVariant::TEXT_SM_MEDIUM)
                .color(TEXT_DEFAULT)
                .line_height(16.0_f32)
                .build(),
        )
}

fn option_row(art_source: ArtRef, label: &str, on_click: impl FnMut() + 'static) -> Node {
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::OPTION.to_string()])
        .on_click(on_click)
        .build()
        .children([
            art(art_source).class(cls::OPTION_ART),
            TextProps::styled(label, TextVariant::TEXT_MD_NORMAL)
                .color(TEXT_DEFAULT)
                .line_height(line_height(16.0_f32))
                .build()
                .class(cls::OPTION_LABEL),
            div().class(cls::SPACER),
            icon(CHEVRON_LARGE_RIGHT, Color::from_rgb(0x4f, 0x56, 0x60)).class(cls::OPTION_ARROW),
        ])
}

fn field(
    title: &str,
    placeholder: &str,
    initial: &str,
    on_change: impl FnMut(&str) + 'static,
    on_submit: impl FnMut(String) + 'static,
) -> Node {
    let input = TextInputProps::new()
        .text(initial.to_string())
        .placeholder(placeholder.to_string())
        .on_change(on_change)
        .on_submit(on_submit)
        .build();
    let error = Node::reactive(move || match ERROR.with(Signal::get) {
        Some(message) => field_error(message),
        None => div(),
    });
    FormItemProps::default()
        .title(title.to_string())
        .required(true)
        .children(
            StackProps::vertical()
                .gap(0)
                .full_width(false)
                .build()
                .children([input, error]),
        )
        .build()
}

fn templates_step() -> Node {
    let header = div().class(cls::HEADER).children([
        TextProps::new(
            requiem_i18n::t!("create-guild-create-your-server"),
            24.0_f32,
        )
        .weight(Weight::BOLD)
        .color(TEXT_STRONG)
        .line_height(line_height(24.0_f32))
        .center(true)
        .build(),
        TextProps::styled(
            requiem_i18n::t!("create-guild-your-server-is-where-you-and"),
            TextVariant::TEXT_MD_NORMAL,
        )
        .color(TEXT_DEFAULT)
        .line_height(line_height(16.0_f32))
        .center(true)
        .build()
        .class(cls::SUBTITLE),
        close_button(),
    ]);

    let choose = |template: GuildTemplate| {
        move || {
            TEMPLATE.with(|t| t.set(template));
            go_to(Step::Intent);
        }
    };
    let mut list = scroll_view().class(cls::TEMPLATES_LIST);
    list = list.child(option_row(
        art!("create_my_own", 80),
        &requiem_i18n::t!("create-guild-create-my-own"),
        choose(guild_templates::CREATE),
    ));
    list = list.child(
        TextProps::styled(
            requiem_i18n::t!("create-guild-start-from-a-template"),
            TextVariant::TEXT_XS_BOLD,
        )
        .color(TEXT_DEFAULT)
        .line_height(line_height(12.0_f32))
        .build()
        .class(cls::OPTION_HEADER),
    );
    for (art_source, label, template) in [
        (
            art!("gaming", 60),
            requiem_i18n::t!("create-guild-gaming"),
            guild_templates::GAMING,
        ),
        (
            art!("friends", 71),
            requiem_i18n::t!("create-guild-friends"),
            guild_templates::FRIENDS,
        ),
        (
            art!("study_group", 80),
            requiem_i18n::t!("create-guild-study-group"),
            guild_templates::STUDY_GROUP,
        ),
        (
            art!("school_club", 80),
            requiem_i18n::t!("create-guild-school-club"),
            guild_templates::SCHOOL_CLUB,
        ),
        (
            art!("local_community", 80),
            requiem_i18n::t!("create-guild-local-community"),
            guild_templates::LOCAL_COMMUNITY,
        ),
        (
            art!("artists", 80),
            requiem_i18n::t!("create-guild-artists-creators"),
            guild_templates::ARTISTS_CREATORS,
        ),
    ] {
        list = list.child(option_row(art_source, &label, choose(template)));
    }

    let join = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .label(requiem_i18n::t!("create-guild-join-a-server").to_string())
        .full_width(true)
        .on_click(|| go_to(Step::Join))
        .build();
    let footer = div().class(cls::FOOTER_STACK).children([
        TextProps::styled(
            requiem_i18n::t!("create-guild-have-an-invite-already"),
            TextVariant::TEXT_LG_SEMIBOLD,
        )
        .color(TEXT_STRONG)
        .line_height(line_height(20.0_f32))
        .center(true)
        .build()
        .class(cls::FOOTER_TITLE),
        join,
    ]);
    div()
        .class(cls::STEP)
        .class(cls::STEP_TEMPLATES)
        .children([header, list, footer])
}

fn intent_step() -> Node {
    let header = div()
        .class(cls::HEADER)
        .class(cls::HEADER_INTENT)
        .children([
            TextProps::new(
                requiem_i18n::t!("create-guild-tell-us-more-about-your-server"),
                24.0_f32,
            )
            .weight(Weight::BOLD)
            .color(TEXT_STRONG)
            .line_height(line_height(24.0_f32))
            .center(true)
            .build(),
            TextProps::styled(
                requiem_i18n::t!("create-guild-in-order-to-help-you-with"),
                TextVariant::TEXT_MD_NORMAL,
            )
            .color(TEXT_DEFAULT)
            .line_height(line_height(16.0_f32))
            .center(true)
            .build()
            .class(cls::SUBTITLE),
            close_button(),
        ]);
    let skip = div().class(cls::SKIP).children([
        TextProps::styled(
            requiem_i18n::t!("create-guild-not-sure-you-can"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_DEFAULT)
        .line_height(line_height(14.0_f32))
        .build(),
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .on_click(|| go_to(Step::Customize))
            .build()
            .child(
                TextProps::styled(
                    requiem_i18n::t!("create-guild-skip-this-question"),
                    TextVariant::TEXT_SM_NORMAL,
                )
                .color(Color::from_rgb(0x4d, 0x96, 0xee))
                .line_height(line_height(14.0_f32))
                .build()
                .class(create_guild::LINK_TEXT),
            ),
        TextProps::styled(
            requiem_i18n::t!("create-guild-for-now"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_DEFAULT)
        .line_height(line_height(14.0_f32))
        .build(),
    ]);
    let rows = div().class(cls::INTENT_ROWS).children([
        option_row(
            art!("intent_friends", 71),
            &requiem_i18n::t!("create-guild-for-me-and-my-friends"),
            || go_to(Step::Customize),
        ),
        option_row(
            art!("intent_community", 80),
            &requiem_i18n::t!("create-guild-for-a-club-or-community"),
            || go_to(Step::Customize),
        ),
        skip,
    ]);
    let back = div()
        .class(cls::INTENT_BACK)
        .child(text_button(&requiem_i18n::t!("create-guild-back"), || {
            go_to(Step::Templates)
        }));
    div().class(cls::STEP).children([header, rows, back])
}

fn customize_step(env: &HomeEnv) -> Node {
    let header = div().class(cls::HEADER).class(cls::HEADER_TALL).children([
        TextProps::new(
            requiem_i18n::t!("create-guild-customize-your-server"),
            24.0_f32,
        )
        .weight(Weight::BOLD)
        .color(TEXT_STRONG)
        .line_height(line_height(24.0_f32))
        .center(true)
        .build(),
        TextProps::styled(
            requiem_i18n::t!("create-guild-give-your-new-server-a-personality"),
            TextVariant::TEXT_MD_NORMAL,
        )
        .color(TEXT_DEFAULT)
        .line_height(line_height(16.0_f32))
        .center(true)
        .build()
        .class(cls::SUBTITLE),
        close_button(),
    ]);

    let pick = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::UPLOAD.to_string()])
        .on_click(|| {
            let Some(picker) = PICK_IMAGE.with(|p| p.borrow().clone()) else {
                return;
            };
            let Some(path) = picker() else { return };
            match prepare_upload(&path, ImageTarget::Avatar, false) {
                Ok(upload) => ICON.with(|i| i.set(Some(upload))),
                Err(err) => log::warn!("couldn't use {}: {err}", path.display()),
            }
        });
    let circle = Node::new("div", UploadCircle).class(cls::UPLOAD_CIRCLE);

    let label = Node::reactive(move || {
        if ICON.with(Signal::get).is_some() {
            div()
        } else {
            TextProps::styled(
                requiem_i18n::t!("create-guild-upload"),
                TextVariant::TEXT_XS_BOLD,
            )
            .color(TEXT_SECONDARY)
            .line_height(line_height(12.0_f32))
            .center(true)
            .build()
            .class(cls::UPLOAD_LABEL)
        }
    });
    let upload = StackProps::horizontal()
        .justify(Justify::Center)
        .gap(0)
        .full_width(false)
        .padding(Padding::top(4))
        .build()
        .child(pick.build().children([circle, label]));

    let default_name = requiem_i18n::t!(
        "create-guild-default-server-name",
        name = env
            .state
            .users
            .current
            .peek()
            .map(|u| u.username)
            .unwrap_or_default()
    );
    let name = NAME.with(|n| n.borrow().clone()).unwrap_or(default_name);
    let create_env = env.clone();
    let form = field(
        &requiem_i18n::t!("create-guild-server-name"),
        "",
        &name,
        |text| NAME.with(|n| *n.borrow_mut() = Some(text.to_string())),
        move |_| create(&create_env),
    );

    let guidelines = message_text(MessageTextProps {
        content: requiem_i18n::md!("create-guild-guidelines"),
        size: 12.0,
        color: TEXT_MUTED,
        line_height: Some(16.0),
        ..Default::default()
    })
    .class(cls::GUIDELINES);
    let content = div()
        .class(cls::CONTENT)
        .children([upload, form, guidelines]);

    let create_button_env = env.clone();
    let create_button = Node::reactive(move || {
        let busy = BUSY.with(Signal::get);
        let env = create_button_env.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .label(requiem_i18n::t!("create-guild-create").to_string())
            .loading(busy)
            .min_width(100.0_f32)
            .on_click(move || create(&env))
            .build()
    });
    let footer = div().class(cls::FOOTER_ROW).children([
        text_button(&requiem_i18n::t!("create-guild-back"), || {
            go_to(Step::Intent)
        }),
        create_button,
    ]);
    div().class(cls::STEP).children([header, content, footer])
}

fn join_step(env: &HomeEnv) -> Node {
    let header = div().class(cls::HEADER).class(cls::HEADER_JOIN).children([
        TextProps::new(requiem_i18n::t!("create-guild-join-a-server"), 24.0_f32)
            .weight(Weight::BOLD)
            .color(TEXT_STRONG)
            .line_height(line_height(24.0_f32))
            .center(true)
            .build()
            .class(cls::JOIN_TITLE),
        TextProps::styled(
            requiem_i18n::t!("create-guild-enter-an-invite-below-to-join"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_DEFAULT)
        .line_height(line_height(14.0_f32))
        .center(true)
        .build(),
        close_button(),
    ]);

    let submit_env = env.clone();
    let form = field(
        &requiem_i18n::t!("create-guild-invite-link"),
        "https://discord.gg/hTKzmak",
        &INVITE.with(|i| i.borrow().clone()),
        |text| INVITE.with(|i| *i.borrow_mut() = text.to_string()),
        move |_| join(&submit_env),
    );

    let sample = |content: &str| {
        div().class(cls::SAMPLE).child(
            TextProps::styled(content, TextVariant::TEXT_SM_NORMAL)
                .color(TEXT_STRONG)
                .line_height(line_height(14.0_f32))
                .build(),
        )
    };
    let samples = StackProps::vertical()
        .gap(0)
        .full_width(false)
        .build()
        .children([
            TextProps::styled(
                requiem_i18n::t!("create-guild-invites-should-look-like"),
                TextVariant::TEXT_XS_BOLD,
            )
            .color(TEXT_DEFAULT)
            .line_height(line_height(12.0_f32))
            .build()
            .class(cls::OPTION_HEADER),
            StackProps::horizontal()
                .gap(0)
                .full_width(false)
                .build()
                .children([sample("hTKzmak"), sample("https://discord.gg/hTKzmak")]),
            StackProps::horizontal()
                .gap(0)
                .full_width(false)
                .build()
                .child(sample("https://discord.gg/wumpus-friends")),
        ]);

    let discover = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::DISCOVER_ROW.to_string()])
        .on_click(|| {})
        .build()
        .children([
            art(art!("discover", 160)).class(cls::DISCOVER_ART),
            div().class(cls::DISCOVER_TEXT).children([
                TextProps::styled(
                    requiem_i18n::t!("create-guild-don-t-have-an-invite"),
                    TextVariant::TEXT_MD_SEMIBOLD,
                )
                .color(TEXT_STRONG)
                .line_height(line_height(16.0_f32))
                .build(),
                TextProps::styled(
                    requiem_i18n::t!("create-guild-discovery-note"),
                    TextVariant::TEXT_XS_NORMAL,
                )
                .color(TEXT_DEFAULT)
                .line_height(line_height(12.0_f32))
                .build(),
            ]),
            icon(CHEVRON_LARGE_RIGHT, Color::from_rgb(0x4f, 0x56, 0x60)).class(cls::OPTION_ARROW),
        ]);
    let content = div()
        .class(cls::CONTENT)
        .class(cls::JOIN_CONTENT)
        .children([form, samples, discover]);

    let join_env = env.clone();
    let join_button = Node::reactive(move || {
        let busy = BUSY.with(Signal::get);
        let env = join_env.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .label(requiem_i18n::t!("create-guild-join-server").to_string())
            .loading(busy)
            .on_click(move || join(&env))
            .build()
    });
    let footer = div().class(cls::FOOTER_ROW).children([
        text_button(&requiem_i18n::t!("create-guild-back"), || {
            go_to(Step::Templates)
        }),
        join_button,
    ]);
    div().class(cls::STEP).children([header, content, footer])
}

fn render(step: Step, env: &HomeEnv) -> Node {
    match step {
        Step::Templates => templates_step(),
        Step::Intent => intent_step(),
        Step::Customize => customize_step(env),
        Step::Join => join_step(env),
    }
}

fn create(env: &HomeEnv) {
    if BUSY.with(|b| b.peek()) {
        return;
    }
    let name = NAME.with(|n| n.borrow().clone()).unwrap_or_else(|| {
        requiem_i18n::t!(
            "create-guild-default-server-name",
            name = env
                .state
                .users
                .current
                .peek()
                .map(|u| u.username)
                .unwrap_or_default()
        )
    });
    let name = name.trim().to_string();
    if name.chars().count() < 2 {
        set_error(Some(
            requiem_i18n::t!("create-guild-must-be-between-2-and-100").to_string(),
        ));
        return;
    }
    set_error(None);
    BUSY.with(|b| b.set(true));
    let icon = ICON.with(|i| i.peek()).map(|u| u.data_uri.to_string());
    let template = TEMPLATE.with(Cell::get);
    let (http, inbox) = (env.http.clone(), env.state.nav.inbox());
    env.queries.mutate(Vec::new(), move || async move {
        let (guild_id, error) = match guilds::create_guild(
            &http,
            &name,
            icon.as_deref(),
            &template,
        )
        .await
        {
            Ok(id) => (Some(id), None),
            Err(message) => (None, Some(message)),
        };
        inbox
            .lock()
            .expect("nav inbox poisoned")
            .push(NavEvent::GuildFlow {
                create: true,
                guild_id,
                error,
            });
        Ok::<(), RestError>(())
    });
}

fn join(env: &HomeEnv) {
    if BUSY.with(|b| b.peek()) {
        return;
    }
    let input = INVITE.with(|i| i.borrow().clone());
    let Some(code) = invites::invite_code(&input) else {
        set_error(Some(
            requiem_i18n::t!("create-guild-please-enter-a-valid-invite-link").to_string(),
        ));
        return;
    };
    set_error(None);
    BUSY.with(|b| b.set(true));
    let (http, inbox) = (env.http.clone(), env.state.nav.inbox());
    env.queries.mutate(Vec::new(), move || async move {
        let joined =
            match invites::resolve_invite(&http, &code, &input).await {
                Ok(invite) => {
                    invites::accept_invite(&http, &code, &invite).await
                }
                Err(message) => Err(message),
            };
        let (guild_id, error) = match joined {
            Ok(id) => (id, None),
            Err(message) => (None, Some(message)),
        };
        inbox
            .lock()
            .expect("nav inbox poisoned")
            .push(NavEvent::GuildFlow {
                create: false,
                guild_id,
                error,
            });
        Ok::<(), RestError>(())
    });
}

fn dialog(env: &HomeEnv) -> Node {
    let toggle = DIALOG.with(Dialog::toggle);

    let host_env = env.clone();
    let host = Node::reactive(move || {
        let slot_node = |index: usize| {
            let env = host_env.clone();
            let content = Node::reactive(move || match SLOTS.with(|slots| slots[index].get()) {
                Some(step) => render(step, &env),
                None => div(),
            });
            div()
                .class(cls::SLOT)
                .bind_style(SLOT_STYLES.with(|styles| styles[index].clone()))
                .observe_size(SIZES.with(|sizes| sizes[index].clone()))
                .child(content)
        };
        div()
            .class(cls::HOST)
            .bind_style(HOST_STYLE.with(Clone::clone))
            .children([slot_node(0), slot_node(1)])
    })
    .class(cls::HOST_WRAP);
    ModalProps::default()
        .size(ModalSize::Small)
        .header(false)
        .dismissable(true)
        .on_dismiss(close)
        .open(toggle)
        .body(host)
        .panel_class(cls::PANEL.to_string())
        .build()
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId, env: Rc<dyn Fn() -> HomeEnv>) -> Vec<Effect> {
    let dialog = section(tree, parent, move || {
        if DIALOG.with(Dialog::is_mounted) {
            dialog(&env())
        } else {
            div()
        }
    });
    vec![dialog, create_effect(follow_measured_height, tree)]
}
