use crate::classes::modal as classes_modal;
use crate::colors::{TEXT_DEFAULT, TEXT_PLACEHOLDER, TEXT_SECONDARY, TEXT_STRONG};
use crate::font_system;
use crate::icons;
use crate::typography::TextVariant;
use crate::widget::primitives::ButtonWidget;
use std::rc::Rc;

use crate::font_system::TextFonts;

use crate::node::Node;
use crate::widget::anim::ToggleState;
use crate::widget::primitives::{ModalBackdropWidget, ModalPanelWidget, TextInputWidget};

use super::container::div;
use super::icon::icon;
use super::text::{TextProps, text};
use crate::icons::TEXT;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ModalSize {
    #[default]
    Small,
    Medium,
    Large,

    /// No preset width and no outer padding. The caller lays out the panel.
    Bare,
}

/// Backdrop click, close button, and Escape. Wrapper so a builder setter accepts a closure.
pub struct OnDismiss(pub Rc<dyn Fn()>);

impl<F: Fn() + 'static> From<F> for OnDismiss {
    fn from(f: F) -> Self {
        OnDismiss(Rc::new(f))
    }
}

pub struct ModalProps {
    pub size: ModalSize,

    /// Header title. Use `heading` for a plain string.
    pub title: Node,

    /// `false` omits the title row and the close button.
    pub header: bool,
    pub subtitle: Option<String>,

    /// Info icon before `subtitle`.
    pub subtitle_icon: bool,

    /// Replaces `subtitle` when set.
    pub subtitle_content: Option<Node>,

    /// `Some` paints a search row. The string is the placeholder, not a bound value.
    pub search_placeholder: Option<String>,

    /// Row under the header, above the body.
    pub controls: Option<Node>,

    /// Scrolling column. Independent of `body` and `preview`.
    pub list: Option<Node>,

    /// Right column beside `list`.
    pub preview: Option<Node>,

    /// Close button, backdrop click, and Escape call `on_dismiss`.
    pub dismissable: bool,
    pub on_dismiss: OnDismiss,

    pub body: Option<Node>,

    pub footer: Vec<Node>,

    /// Left side of the footer. `footer` stays on the right.
    pub footer_leading: Option<Node>,

    pub fonts: Option<TextFonts>,

    /// Drives the open and close transition. Closing does not drop the node.
    pub open: ToggleState,

    /// Extra class on the panel, for a caller that needs its own width or padding.
    pub panel_class: Option<String>,

    /// Body has no padding.
    pub flush_body: bool,

    /// Body scrolls instead of growing the panel.
    pub scroll_body: bool,
}

impl Default for ModalProps {
    fn default() -> Self {
        Self {
            size: ModalSize::default(),
            title: Node::default(),
            header: true,
            subtitle: None,
            subtitle_icon: false,
            search_placeholder: None,
            dismissable: false,
            on_dismiss: OnDismiss(Rc::new(|| {})),
            body: None,
            footer: Vec::new(),
            footer_leading: None,
            subtitle_content: None,
            controls: None,
            list: None,
            preview: None,
            fonts: None,
            open: ToggleState::new(true),
            panel_class: None,
            flush_body: false,
            scroll_body: false,
        }
    }
}

crate::builder_fields!(ModalProps {
    size: ModalSize,
    title: Node,
    header: bool,
    opt subtitle: String,
    subtitle_icon: bool,
    opt search_placeholder: String,
    opt subtitle_content: Node,
    opt controls: Node,
    opt list: Node,
    opt preview: Node,
    dismissable: bool,
    on_dismiss: OnDismiss,
    opt body: Node,
    footer: Vec<Node>,
    opt footer_leading: Node,
    opt fonts: TextFonts,
    open: ToggleState,
    opt panel_class: String,
    flush_body: bool,
    scroll_body: bool,
});

impl ModalProps {
    pub fn heading(self, text: impl Into<String>) -> Self {
        self.title(
            super::heading::HeadingProps::new(super::heading::HeadingLevel::H1, text)
                .variant(TextVariant::HEADING_LG_SEMIBOLD)
                .build(),
        )
    }

    pub fn build(self) -> Node {
        modal(self)
    }
}

pub fn modal(props: ModalProps) -> Node {
    let fonts = font_system::resolve(props.fonts);

    let backdrop_widget = if props.dismissable {
        let dismiss = props.on_dismiss.0.clone();
        ModalBackdropWidget::new(props.open).with_on_click(move || dismiss())
    } else {
        ModalBackdropWidget::new(props.open)
    };

    let size_class = match props.size {
        ModalSize::Small => classes_modal::MODAL_SMALL,
        ModalSize::Medium => classes_modal::MODAL_MEDIUM,
        ModalSize::Large => classes_modal::MODAL_LARGE,
        ModalSize::Bare => classes_modal::MODAL_BARE,
    };

    let mut panel = Node::new("div", ModalPanelWidget::new(props.open))
        .class(classes_modal::MODAL_CONTAINER)
        .class(size_class);
    if let Some(extra) = props.panel_class {
        panel = panel.class(extra);
    }

    if props.header {
        panel = panel.child(build_header(
            props.title,
            props.subtitle,
            props.subtitle_icon,
            props.subtitle_content,
            props.dismissable,
            props.on_dismiss.0.clone(),
            &fonts,
        ));
    }

    if let Some(placeholder) = &props.search_placeholder {
        panel = panel.child(build_search(placeholder));
    }

    if let Some(controls) = props.controls {
        panel = panel
            .child(div().class(classes_modal::MODAL_BODY_SPACER_TOP))
            .child(
                div()
                    .class(classes_modal::MODAL_CONTROLS)
                    .child(controls),
            );
    }

    let list_mode = props.list.is_some();
    if let Some(list) = props.list {
        panel = panel.class(classes_modal::MODAL_HAS_LIST);
        panel = panel
            .child(
                super::scroll_view::scroll_view()
                    .class(classes_modal::MODAL_LIST)
                    .child(list),
            )
            .child(div().class(classes_modal::MODAL_BODY_SPACER_BOTTOM));
    }

    if let Some(body) = props.body {
        if props.scroll_body {
            panel = panel.class(classes_modal::MODAL_HAS_LIST);
            let body = body.class(classes_modal::MODAL_SCROLL_BODY_INNER);
            panel = panel
                .child(div().class(classes_modal::MODAL_BODY_SPACER_TOP))
                .child(
                    super::scroll_view::scroll_view()
                        .class(classes_modal::MODAL_SCROLL_BODY)
                        .child(body),
                )
                .child(div().class(classes_modal::MODAL_BODY_SPACER_BOTTOM));
        } else {
            let body_class = if props.flush_body {
                classes_modal::MODAL_BODY_FLUSH
            } else {
                classes_modal::MODAL_BODY
            };
            panel = panel.child(div().class(body_class).child(body));
        }
    }

    if let Some(preview) = props.preview {
        let last = props.footer.is_empty() && props.footer_leading.is_none();
        let class = if last {
            classes_modal::MODAL_PREVIEW_LAST
        } else {
            classes_modal::MODAL_PREVIEW
        };
        panel = panel.child(div().class(class).child(preview));
    }

    if let Some(leading) = props.footer_leading {
        panel = panel.child(
            div()
                .class(classes_modal::MODAL_FOOTER)
                .child(
                    div()
                        .class(classes_modal::MODAL_FOOTER_LEADING)
                        .child(leading),
                )
                .child(
                    div()
                        .class(classes_modal::MODAL_FOOTER_TRAILING)
                        .children(props.footer),
                ),
        );
    } else if !props.footer.is_empty() {
        panel = panel.child(
            div()
                .class(classes_modal::MODAL_FOOTER)
                .children(props.footer),
        );
    }

    let mut node = Node::new("div", backdrop_widget)
        .class(classes_modal::MODAL_BACKDROP)
        .child(panel);
    if list_mode || props.scroll_body || !matches!(props.size, ModalSize::Bare) {
        node = node.class(classes_modal::MODAL_BACKDROP_PADDED);
    }

    if props.open.on {
        node.focus_scope()
    } else {
        node
    }
}

#[allow(clippy::too_many_arguments)]
fn build_header(
    title: Node,
    subtitle: Option<String>,
    subtitle_icon: bool,
    subtitle_content: Option<Node>,
    dismissable: bool,
    on_dismiss: Rc<dyn Fn()>,
    fonts: &TextFonts,
) -> Node {
    let mut row = div()
        .class(classes_modal::MODAL_HEADER_ROW)
        .child(title.class(classes_modal::MODAL_TITLE));

    if dismissable {
        let close_btn =
            ButtonWidget::new().with_on_click(move || on_dismiss());
        let close_node = Node::new("button", close_btn)
            .class(classes_modal::MODAL_CLOSE_BTN)
            .child(
                icon(icons::X_LARGE, TEXT_SECONDARY)
                    .class(classes_modal::MODAL_CLOSE_ICON),
            );
        row = row.child(close_node);
    }

    let mut header = div().class(classes_modal::MODAL_HEADER).child(row);

    if let Some(subtitle) = subtitle {
        let mut subtitle_row = div().class(classes_modal::MODAL_SUBTITLE_ROW);
        if subtitle_icon {
            subtitle_row = subtitle_row.child(
                icon(TEXT, TEXT_SECONDARY).class(classes_modal::MODAL_SUBTITLE_ICON),
            );
        }
        subtitle_row = subtitle_row.child(text(TextProps {
            content: subtitle,
            size: 16.0,
            color: TEXT_SECONDARY,
            fonts: Some(fonts.clone()),
            ..Default::default()
        }));
        header = header.child(subtitle_row);
    } else if let Some(content) = subtitle_content {
        header = header.child(
            div()
                .class(classes_modal::MODAL_SUBTITLE_ROW)
                .child(content),
        );
    }

    header
}

pub fn modal_search_box(placeholder: &str, placeholder_color: skia_safe::Color) -> Node {
    let fonts = font_system::resolve(None);
    let input = TextInputWidget::new(
        "",
        placeholder,
        16.0,
        TEXT_DEFAULT,
        placeholder_color,
        fonts.font_system.clone(),
        fonts.glyph_cache.clone(),
        false,
    );
    div()
        .class(classes_modal::MODAL_SEARCH_BOX)
        .focus_ring_host()
        .child(
            icon(icons::MAGNIFYING_GLASS, TEXT_STRONG)
                .class(classes_modal::MODAL_SEARCH_ICON),
        )
        .child(Node::new("div", input).class(classes_modal::MODAL_SEARCH_INPUT))
}

fn build_search(placeholder: &str) -> Node {
    div()
        .class(classes_modal::MODAL_SEARCH_ROW)
        .child(modal_search_box(placeholder, TEXT_PLACEHOLDER))
}
