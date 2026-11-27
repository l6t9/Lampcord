use crate::classes::text_input as classes_text_input;
use crate::colors::{TEXT_SECONDARY, TEXT_STRONG};
use crate::colors;
use crate::event;
use crate::font_system;
use crate::icon_handle;
use crate::paint as crate_paint;
use crate::style;
use crate::widget::primitives;
use crate::widget;
use skia_safe::Color;
use std::cell::RefCell;

use crate::font_system::TextFonts;
use crate::node::Node;
use crate::signal::Signal;
use std::cell::Cell;
use std::rc::Rc;

use crate::widget::Widget;
use crate::widget::primitives::{
    AutocompleteTrigger, ButtonWidget, DynamicAutocomplete, EmojiProvider, MarkdownFeatures,
    TextInputWidget, TextWidget,
};

use super::text::{TextProps, text};

use super::container::div;

#[derive(Default)]
/// Current text after each edit.
pub struct OnChange(pub Option<Box<dyn FnMut(&str)>>);

impl<F: FnMut(&str) + 'static> From<F> for OnChange {
    fn from(f: F) -> Self {
        OnChange(Some(Box::new(f)))
    }
}

#[derive(Default)]
/// Enter. The string is the text at submit.
pub struct OnSubmit(pub Option<Box<dyn FnMut(String)>>);

impl<F: FnMut(String) + 'static> From<F> for OnSubmit {
    fn from(f: F) -> Self {
        OnSubmit(Some(Box::new(f)))
    }
}

#[derive(Default)]
/// `true` on focus, `false` on blur.
pub struct OnFocusChange(pub Option<Box<dyn FnMut(bool)>>);

impl<F: FnMut(bool) + 'static> From<F> for OnFocusChange {
    fn from(f: F) -> Self {
        OnFocusChange(Some(Box::new(f)))
    }
}

#[derive(Default)]
/// Key, Shift, current text. Return `true` to consume the key.
pub struct KeyHook(pub Option<Box<dyn FnMut(event::NamedKey, bool, &str) -> bool>>);

impl<F: FnMut(event::NamedKey, bool, &str) -> bool + 'static> From<F> for KeyHook {
    fn from(f: F) -> Self {
        KeyHook(Some(Box::new(f)))
    }
}

#[derive(Default)]
pub struct TextInputProps {
    pub text: String,
    pub placeholder: String,
    pub font_size: f32,
    pub color: Color,
    pub placeholder_color: Color,

    pub fonts: Option<TextFonts>,
    pub focused: bool,
    pub on_change: OnChange,
    pub on_submit: OnSubmit,
    pub on_focus_change: OnFocusChange,

    pub autocomplete: Vec<AutocompleteTrigger>,

    pub dynamic_autocomplete: Vec<DynamicAutocomplete>,

    /// Extra px the autocomplete popup may extend past the field, `(x, y)`.
    pub popup_outset: Option<(f32, f32)>,

    pub multiline: bool,

    /// Soft wrap. Only applies when `multiline`.
    pub wrap: bool,

    pub focus_border_color: Option<Color>,

    pub error: Option<String>,

    pub markdown: MarkdownFeatures,

    pub emoji_provider: Option<EmojiProvider>,

    pub custom_emoji: Option<primitives::CustomEmojiResolver>,

    pub code_highlighter: Option<primitives::CodeHighlighter>,

    /// Paints bullets. The stored text is unchanged.
    pub password: bool,

    pub autofocus: bool,

    pub helper_text: Option<String>,

    /// Char limit. Further input is dropped. `None` is unlimited.
    pub max_length: Option<usize>,

    /// `current/max` under the field. Needs `max_length`.
    pub show_character_count: bool,

    /// Clear button while the field has text.
    pub clearable: bool,

    /// Focusable, but edits are ignored.
    pub read_only: bool,

    pub disabled: bool,

    pub leading: Option<Node>,

    pub trailing: Option<Node>,

    /// Visible lines for a multiline field. `None` grows with the text.
    pub rows: Option<u32>,

    /// Horizontal padding inside the field, `(left, right)` in px.
    pub insets: Option<(f32, f32)>,

    pub placeholder_font_size: Option<f32>,

    /// Placeholder that can change without rebuilding the field. Wins over `placeholder`.
    pub placeholder_live: Option<Signal<String>>,

    pub line_height: Option<f32>,

    pub vertical_padding: Option<f32>,

    /// Enter on an empty field submits only when this returns `true`.
    pub submit_when_empty: Option<Rc<dyn Fn() -> bool>>,

    /// Paste is consumed, and the clipboard is not inserted, when this returns `true`.
    pub paste_override: Option<Rc<dyn Fn() -> bool>>,

    pub key_hook: KeyHook,

    /// Arrow at the start or end, and Escape and Tab, before the field handles them.
    pub edge_hook: KeyHook,

    /// Width follows the text instead of the container.
    pub auto_width: bool,

    /// Rect the autocomplete popup must stay inside.
    pub popup_frame: Option<Signal<widget::AbsoluteRect>>,

    /// No border, background, or padding. The field is just the text.
    pub bare: bool,
}

impl TextInputProps {
    pub fn new() -> Self {
        Self {
            font_size: 16.0,
            color: colors::TEXT_DEFAULT,
            placeholder_color: colors::INPUT_PLACEHOLDER,
            ..Default::default()
        }
    }
}

crate::builder_fields!(TextInputProps {
    text: String,
    placeholder: String,
    font_size: f32,
    color: Color,
    placeholder_color: Color,
    opt fonts: TextFonts,
    focused: bool,
    on_change: OnChange,
    on_submit: OnSubmit,
    on_focus_change: OnFocusChange,
    autocomplete: Vec<AutocompleteTrigger>,
    dynamic_autocomplete: Vec<DynamicAutocomplete>,
    opt popup_outset: (f32, f32),
    multiline: bool,
    wrap: bool,
    opt focus_border_color: Color,
    opt error: String,
    markdown: MarkdownFeatures,
    opt emoji_provider: EmojiProvider,
    opt custom_emoji: primitives::CustomEmojiResolver,
    opt code_highlighter: primitives::CodeHighlighter,
    password: bool,
    autofocus: bool,
    opt helper_text: String,
    opt max_length: usize,
    show_character_count: bool,
    clearable: bool,
    read_only: bool,
    disabled: bool,
    opt leading: Node,
    opt trailing: Node,
    opt rows: u32,
    opt insets: (f32, f32),
    opt placeholder_font_size: f32,
    opt placeholder_live: Signal<String>,
    opt line_height: f32,
    opt vertical_padding: f32,
    opt submit_when_empty: Rc<dyn Fn() -> bool>,
    opt paste_override: Rc<dyn Fn() -> bool>,
    key_hook: KeyHook,
    edge_hook: KeyHook,
    auto_width: bool,
    opt popup_frame: Signal<widget::AbsoluteRect>,
    bare: bool,
});

impl TextInputProps {
    pub fn build(self) -> Node {
        text_input(self)
    }

    pub fn build_area(self) -> Node {
        text_area(self)
    }
}

pub fn text_input(props: TextInputProps) -> Node {
    let TextInputProps {
        text: initial_text,
        placeholder,
        font_size,
        color,
        placeholder_color,
        fonts,
        focused,
        on_change,
        on_submit,
        on_focus_change,
        autocomplete,
        dynamic_autocomplete,
        popup_outset,
        multiline,
        wrap,
        focus_border_color,
        error,
        markdown,
        emoji_provider,
        custom_emoji,
        code_highlighter,
        password,
        autofocus,
        helper_text,
        max_length,
        show_character_count,
        clearable,
        read_only,
        disabled,
        leading,
        trailing,
        rows,
        insets,
        placeholder_font_size,
        placeholder_live,
        line_height,
        vertical_padding,
        submit_when_empty,
        paste_override,
        key_hook,
        edge_hook,
        auto_width,
        popup_frame,
        bare,
    } = props;
    let fonts = font_system::resolve(fonts);

    let count = Rc::new(Cell::new(initial_text.chars().count()));
    let mut on_change = on_change.0;
    if show_character_count {
        let count = count.clone();
        let mut inner = on_change.take();
        on_change = Some(Box::new(move |text: &str| {
            count.set(text.chars().count());
            crate_paint::request_repaint();
            if let Some(inner) = inner.as_mut() {
                inner(text);
            }
        }));
    }

    let mut widget = TextInputWidget::new(
        initial_text,
        placeholder,
        font_size,
        color,
        placeholder_color,
        fonts.font_system.clone(),
        fonts.glyph_cache.clone(),
        focused || autofocus,
    );
    if let Some(on_change) = on_change {
        widget = widget.with_on_change(on_change);
    }
    if let Some(on_submit) = on_submit.0 {
        widget = widget.with_on_submit(on_submit);
    }
    if let Some(on_focus_change) = on_focus_change.0 {
        widget = widget.with_on_focus_change(on_focus_change);
    }
    if !autocomplete.is_empty() {
        widget = widget.with_autocomplete(autocomplete);
    }
    for source in dynamic_autocomplete {
        widget = widget.with_dynamic_autocomplete(source);
    }
    if let Some((left, right)) = popup_outset {
        widget = widget.with_popup_outset(left, right);
    }
    if multiline {
        widget = widget.with_multiline();
    }
    if wrap {
        widget = widget.with_wrap();
    }
    if let Some(focus_border_color) = focus_border_color {
        widget = widget.with_focus_border_color(focus_border_color);
    }
    if markdown != MarkdownFeatures::NONE {
        widget = widget.with_markdown(markdown);
    }
    if let Some(provider) = emoji_provider {
        widget = widget.with_emoji_provider(provider);
    }
    if let Some(resolver) = custom_emoji {
        widget = widget.with_custom_emoji(resolver);
    }
    if let Some(highlighter) = code_highlighter {
        widget = widget.with_code_highlighter(highlighter);
    }
    if password {
        widget = widget.with_password();
    }
    if read_only {
        widget = widget.with_read_only();
    }
    if disabled {
        widget = widget.with_disabled();
    }
    if let Some(max) = max_length {
        widget = widget.with_max_length(max);
    }
    if clearable {
        widget = widget.with_clear_button();
    }
    if multiline {
        widget = widget.with_insets(10.0, 10.0);
    }
    if let Some((left, right)) = insets {
        widget = widget.with_insets(left, right);
    }
    if let Some(live) = placeholder_live {
        widget = widget.with_live_placeholder(live);
    }
    if let Some(size) = placeholder_font_size {
        widget = widget.with_placeholder_font_size(size);
    }
    if let Some(px) = line_height {
        widget = widget.with_line_height(px);
    }
    if let Some(px) = vertical_padding {
        widget = widget.with_vertical_padding(px);
    }
    if let Some(allowed) = submit_when_empty {
        widget = widget.with_submit_when_empty(allowed);
    }
    if let Some(hook) = paste_override {
        widget = widget.with_paste_override(hook);
    }
    if let Some(hook) = key_hook.0 {
        widget = widget.with_key_hook(hook);
    }
    if let Some(hook) = edge_hook.0 {
        widget = widget.with_edge_hook(hook);
    }
    widget = widget.with_auto_width(auto_width);
    if let Some(frame) = popup_frame {
        widget = widget.with_popup_frame(frame);
    }

    let has_error = error.is_some();
    let box_class = if multiline {
        classes_text_input::TEXTAREA_BOX
    } else {
        classes_text_input::INPUT_BOX
    };
    let state_classes = |mut node: Node| {
        if has_error {
            node = node.class(classes_text_input::INPUT_BOX_ERROR);
        }
        if read_only {
            node = node.class(classes_text_input::INPUT_BOX_READ_ONLY);
        }
        if disabled {
            node = node.class(classes_text_input::INPUT_BOX_DISABLED);
        }
        node
    };

    let input_box = if leading.is_some() || trailing.is_some() {
        let (left, right) = insets.unwrap_or((
            if leading.is_some() { 0.0 } else { 12.0 },
            if trailing.is_some() { 0.0 } else { 12.0 },
        ));
        let widget = widget.with_insets(left, right);
        let mut inner = Node::new("div", widget).class(classes_text_input::INPUT_INNER);
        if autofocus {
            inner = inner.wants_focus();
        }
        let mut row = div()
            .class(box_class)
            .class(classes_text_input::INPUT_ROW)
            .focus_ring_host();
        if let Some(leading) = leading {
            row = row
                .class(classes_text_input::HAS_LEADING)
                .child(leading);
        }
        row = row.child(inner);
        if let Some(trailing) = trailing {
            row = row
                .class(classes_text_input::HAS_TRAILING)
                .child(trailing);
        }
        state_classes(row)
    } else {
        let widget = match insets {
            Some((left, right)) => widget.with_insets(left, right),
            None => widget,
        };
        let mut node = if bare {
            Node::new("div", widget)
        } else {
            state_classes(Node::new("div", widget).class(box_class))
        };
        if autofocus {
            node = node.wants_focus();
        }
        if multiline {
            node = node.style(format!("height: {}px", 26 + 21 * rows.unwrap_or(3)));
        }
        node
    };

    let error = error.filter(|e| !e.is_empty());
    let counter = (show_character_count && max_length.is_some()).then(|| {
        Node::new(
            "div",
            CharCountWidget::new(count, max_length.unwrap_or(0), &fonts),
        )
    });
    if error.is_none() && helper_text.is_none() && counter.is_none() && !has_error {
        return input_box;
    }

    let mut field = div()
        .class(classes_text_input::FIELD)
        .child(input_box);
    if error.is_some() || helper_text.is_some() || counter.is_some() {
        let mut helper_row = div().class(classes_text_input::HELPER_ROW);
        let mut left = div().class(classes_text_input::HELPER_TEXT);
        if let Some(error) = error {
            left = left.child(super::form_item::field_error(error));
        } else if let Some(helper) = helper_text {
            left = left.child(text(TextProps {
                content: helper,
                size: 12.0,
                color: TEXT_SECONDARY,
                fonts: Some(fonts.clone()),
                ..Default::default()
            }));
        }
        helper_row = helper_row.child(left);
        if let Some(counter) = counter {
            helper_row = helper_row.child(counter);
        }
        if disabled {
            helper_row = helper_row.class(classes_text_input::INPUT_BOX_DISABLED);
        }
        field = field.child(helper_row);
    }
    field
}

pub fn input_accessory(
    icon_path: impl icon_handle::IconPath,
    on_click: impl FnMut() + 'static,
) -> Node {
    Node::new("button", ButtonWidget::new().with_on_click(on_click))
        .class(classes_text_input::ACCESSORY)
        .child(
            super::icon::icon(icon_path, TEXT_STRONG)
                .class(classes_text_input::ACCESSORY_ICON),
        )
}

pub fn input_leading_icon(icon_path: impl icon_handle::IconPath) -> Node {
    div().class(classes_text_input::LEADING_ICON).child(
        super::icon::icon(icon_path, TEXT_STRONG)
            .class(classes_text_input::LEADING_ICON_GLYPH),
    )
}

struct CharCountWidget {
    count: Rc<Cell<usize>>,
    max: usize,
    shown: Cell<usize>,
    label: RefCell<TextWidget>,

    widest: TextWidget,
}

impl CharCountWidget {
    fn new(count: Rc<Cell<usize>>, max: usize, fonts: &TextFonts) -> Self {
        let make = |t: String| {
            TextWidget::new(
                t,
                12.0,
                TEXT_SECONDARY,
                fonts.font_system.clone(),
                fonts.glyph_cache.clone(),
            )
        };
        let n = count.get();
        Self {
            count,
            max,
            shown: Cell::new(n),
            label: RefCell::new(make(format!("{n}/ {max}"))),
            widest: make(format!("{max}/ {max}")),
        }
    }
}

impl Widget for CharCountWidget {
    fn measure(
        &self,
        known: taffy::Size<Option<f32>>,
        available: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::Size<f32> {
        self.widest.measure(known, available)
    }

    fn paint(
        &self,
        canvas: &skia_safe::Canvas,
        rect: widget::AbsoluteRect,
        style: &style::ComputedStyle,
    ) {
        let n = self.count.get();
        if n != self.shown.get() {
            self.shown.set(n);
            self.label
                .borrow_mut()
                .set_text(format!("{n}/ {}", self.max));
        }
        self.label.borrow().paint(canvas, rect, style);
    }
}

pub fn text_area(props: TextInputProps) -> Node {
    text_input(TextInputProps {
        multiline: true,
        ..props
    })
}
