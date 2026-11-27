use requiem_ui::event::CursorIcon;
use requiem_ui::selection;
use std::cell::Cell;
use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

use cosmic_text::{
    Attrs, Buffer, Color as CosmicColor, Family, FontSystem, Metrics, Shaping, Wrap,
};
use skia_safe::{Canvas, Color, Paint, RRect, Rect};
use taffy::{AvailableSpace, Size};
use tree_sitter::Parser;

use requiem_ui::app::current_scale_factor;
use requiem_ui::colors;
use requiem_ui::font_system::{GG_SANS_MONO_FAMILY, TextFonts};
use requiem_ui::glyph_cache::{GlyphCache, paint_glyph};
use requiem_ui::node::Node;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::CodeHighlighter;
use requiem_ui::widget::{AbsoluteRect, Widget};

const CODE_FONT_SIZE: f32 = 14.0;

const CODE_LINE_HEIGHT: f32 = 18.0;

const CODE_PADDING: f32 = 8.0;

const CODE_RADIUS: f32 = 4.0;

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub lang: String,

    pub code: String,
}

pub enum Segment {
    Text(Range<usize>),

    Code(CodeBlock),
}

pub fn segments(text: &str) -> Vec<Segment> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut text_start = 0usize;
    let mut i = 0usize;
    while i + 3 <= text.len() {
        if bytes[i..i + 3] == b"```"[..] && (i == 0 || bytes[i - 1] == b'\n') {
            if let Some((end, block)) = parse_block(text, i) {
                if text_start < i {
                    out.push(Segment::Text(text_start..i));
                }
                out.push(Segment::Code(block));
                i = end;
                text_start = end;
                continue;
            }
        }
        i += 1;
    }
    if text_start < text.len() {
        out.push(Segment::Text(text_start..text.len()));
    }

    for idx in 0..out.len() {
        let prev_is_code = idx > 0 && matches!(out[idx - 1], Segment::Code(_));
        let next_is_code = idx + 1 < out.len() && matches!(out[idx + 1], Segment::Code(_));
        if let Segment::Text(range) = &mut out[idx] {
            if prev_is_code && text.as_bytes().get(range.start) == Some(&b'\n') {
                range.start += 1;
            }
            if next_is_code && range.end > range.start && text.as_bytes()[range.end - 1] == b'\n' {
                range.end -= 1;
            }
        }
    }
    out.retain(|s| !matches!(s, Segment::Text(r) if r.is_empty()));
    out
}

fn parse_block(text: &str, open: usize) -> Option<(usize, CodeBlock)> {
    let rest = &text[open + 3..];

    let lang_len = rest
        .bytes()
        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'-')
        .count();
    let (lang, content_start) = if lang_len > 0 && rest.as_bytes().get(lang_len) == Some(&b'\n') {
        let mut j = lang_len;
        while rest.as_bytes().get(j) == Some(&b'\n') {
            j += 1;
        }
        (rest[..lang_len].to_ascii_lowercase(), j)
    } else {
        let mut j = 0;
        while rest.as_bytes().get(j) == Some(&b'\n') {
            j += 1;
        }
        (String::new(), j)
    };
    let content = &rest[content_start..];
    let close = content.find("```")?;
    let code = content[..close].trim_end_matches('\n');
    Some((
        open + 3 + content_start + close + 3,
        CodeBlock {
            lang,
            code: code.to_string(),
        },
    ))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Highlight {
    Keyword,
    String,
    Comment,
    Number,
    Type,
    Function,
    Property,
    Constant,

    Addition,

    Deletion,
}

fn color_of(h: Highlight) -> Color {
    match h {
        Highlight::Keyword => colors::text_code_keyword(),
        Highlight::String => colors::text_code_string(),
        Highlight::Comment => colors::text_code_comment(),
        Highlight::Number => colors::text_code_number(),
        Highlight::Type => colors::text_code_type(),
        Highlight::Function => colors::text_code_title(),
        Highlight::Property => colors::text_code_property(),
        Highlight::Constant => colors::text_code_builtin(),
        Highlight::Addition => colors::text_code_addition(),
        Highlight::Deletion => colors::text_code_deletion(),
    }
}

pub fn composer_highlighter() -> CodeHighlighter {
    Rc::new(|lang: &str, code: &str| match Lang::from_alias(lang) {
        Some(lang) => highlight(code, lang)
            .into_iter()
            .map(|(range, h)| (range, color_of(h)))
            .collect(),
        None => Vec::new(),
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Lang {
    Rust,
    JavaScript,
    TypeScript,
    Tsx,
    Python,
    Json,
    Bash,
    C,
    Cpp,
    CSharp,
    Css,
    Html,
    Go,
    Java,
    Yaml,
    Diff,
}

impl Lang {
    fn from_alias(alias: &str) -> Option<Lang> {
        match alias {
            "rust" | "rs" => Some(Lang::Rust),
            "javascript" | "js" | "jsx" | "mjs" | "cjs" | "node" => Some(Lang::JavaScript),
            "typescript" | "ts" => Some(Lang::TypeScript),
            "tsx" => Some(Lang::Tsx),
            "python" | "py" | "python3" => Some(Lang::Python),
            "json" => Some(Lang::Json),
            "bash" | "sh" | "shell" | "zsh" | "ksh" => Some(Lang::Bash),
            "c" | "h" => Some(Lang::C),
            "cpp" | "c++" | "cc" | "cxx" | "hpp" | "hxx" => Some(Lang::Cpp),
            "csharp" | "cs" | "c#" | "c-sharp" => Some(Lang::CSharp),
            "css" => Some(Lang::Css),
            "html" | "xml" | "xhtml" | "htm" | "svg" => Some(Lang::Html),
            "go" | "golang" => Some(Lang::Go),
            "java" => Some(Lang::Java),
            "yaml" | "yml" => Some(Lang::Yaml),
            "diff" | "patch" => Some(Lang::Diff),
            _ => None,
        }
    }

    fn language(self) -> tree_sitter::Language {
        match self {
            Lang::Rust => tree_sitter_rust::LANGUAGE.into(),
            Lang::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Lang::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Lang::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Lang::Python => tree_sitter_python::LANGUAGE.into(),
            Lang::Json => tree_sitter_json::LANGUAGE.into(),
            Lang::Bash => tree_sitter_bash::LANGUAGE.into(),
            Lang::C => tree_sitter_c::LANGUAGE.into(),
            Lang::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Lang::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            Lang::Css => tree_sitter_css::LANGUAGE.into(),
            Lang::Html => tree_sitter_html::LANGUAGE.into(),
            Lang::Go => tree_sitter_go::LANGUAGE.into(),
            Lang::Java => tree_sitter_java::LANGUAGE.into(),
            Lang::Yaml => tree_sitter_yaml::LANGUAGE.into(),
            Lang::Diff => tree_sitter_yaml::LANGUAGE.into(),
        }
    }
}

fn highlight(code: &str, lang: Lang) -> Vec<(Range<usize>, Highlight)> {
    if lang == Lang::Diff {
        return highlight_diff(code);
    }
    let mut parser = Parser::new();
    if parser.set_language(&lang.language()).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(code, None) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    walk(tree.root_node(), None, None, &mut out);
    out.sort_by_key(|(r, _)| r.start);
    out
}

fn highlight_diff(code: &str) -> Vec<(Range<usize>, Highlight)> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    for line in code.split_inclusive('\n') {
        let h = if line.starts_with("+++") || line.starts_with("---") {
            Some(Highlight::Comment)
        } else if line.starts_with("@@") {
            Some(Highlight::Function)
        } else if line.starts_with('+') {
            Some(Highlight::Addition)
        } else if line.starts_with('-') {
            Some(Highlight::Deletion)
        } else {
            None
        };
        if let Some(h) = h {
            out.push((pos..pos + line.trim_end_matches('\n').len(), h));
        }
        pos += line.len();
    }
    out
}

fn walk(
    node: tree_sitter::Node,
    parent: Option<&str>,
    field: Option<&str>,
    out: &mut Vec<(Range<usize>, Highlight)>,
) {
    if node.is_error() || node.is_missing() {
        return;
    }
    let kind = node.kind();

    if let Some(h) = classify_direct(kind, node.is_named(), node.child_count() == 0) {
        out.push((node.start_byte()..node.end_byte(), h));
        return;
    }
    if node.named_child_count() == 0 {
        if let Some(h) = classify_identifier_leaf(kind, field, parent) {
            out.push((node.start_byte()..node.end_byte(), h));
        }
        return;
    }

    let n = node.child_count();
    for i in 0..n {
        if let Some(child) = node.child(i) {
            walk(child, Some(kind), node.field_name_for_child(i), out);
        }
    }
}

fn is_string_kind(kind: &str) -> bool {
    kind.contains("string")
        || kind.contains("char")
        || kind.contains("heredoc")
        || matches!(
            kind,
            "character_literal" | "chunk" | "template_string" | "interpreted_string_literal"
        )
}

fn is_number_kind(kind: &str) -> bool {
    kind.contains("number") || kind.contains("integer") || kind.contains("float")
}

fn is_constant_kind(kind: &str) -> bool {
    matches!(
        kind,
        "true"
            | "false"
            | "null"
            | "none"
            | "nil"
            | "undefined"
            | "nullptr"
            | "constant"
            | "boolean_scalar"
            | "null_scalar"
            | "self"
            | "Self"
            | "this"
    )
}

fn is_keyword(kind: &str) -> bool {
    matches!(
        kind,
        "abstract"
            | "alignas"
            | "alignof"
            | "and"
            | "as"
            | "assert"
            | "async"
            | "auto"
            | "await"
            | "base"
            | "become"
            | "box"
            | "break"
            | "byte"
            | "case"
            | "catch"
            | "chan"
            | "char"
            | "checked"
            | "class"
            | "co_await"
            | "co_return"
            | "co_yield"
            | "concept"
            | "const"
            | "const_cast"
            | "constexpr"
            | "continue"
            | "crate"
            | "debugger"
            | "decimal"
            | "declare"
            | "decltype"
            | "default"
            | "defer"
            | "def"
            | "del"
            | "delete"
            | "do"
            | "double"
            | "dyn"
            | "dynamic_cast"
            | "elif"
            | "else"
            | "enum"
            | "event"
            | "except"
            | "explicit"
            | "export"
            | "extends"
            | "extern"
            | "fallthrough"
            | "final"
            | "finally"
            | "fixed"
            | "float"
            | "for"
            | "foreach"
            | "friend"
            | "from"
            | "fn"
            | "func"
            | "function"
            | "get"
            | "global"
            | "go"
            | "goto"
            | "if"
            | "impl"
            | "implements"
            | "implicit"
            | "import"
            | "in"
            | "infer"
            | "inline"
            | "instanceof"
            | "int"
            | "interface"
            | "internal"
            | "is"
            | "keyof"
            | "lambda"
            | "let"
            | "lock"
            | "long"
            | "loop"
            | "macro"
            | "macro_rules"
            | "map"
            | "match"
            | "mod"
            | "module"
            | "move"
            | "mut"
            | "mutable"
            | "namespace"
            | "native"
            | "new"
            | "noexcept"
            | "nonlocal"
            | "not"
            | "object"
            | "of"
            | "operator"
            | "or"
            | "out"
            | "override"
            | "package"
            | "params"
            | "pass"
            | "permit"
            | "permits"
            | "priv"
            | "private"
            | "protected"
            | "public"
            | "pub"
            | "raise"
            | "range"
            | "readonly"
            | "record"
            | "ref"
            | "register"
            | "reinterpret_cast"
            | "required"
            | "requires"
            | "restrict"
            | "return"
            | "satisfies"
            | "sbyte"
            | "sealed"
            | "select"
            | "set"
            | "short"
            | "signed"
            | "sizeof"
            | "stackalloc"
            | "static"
            | "static_assert"
            | "static_cast"
            | "strictfp"
            | "string"
            | "struct"
            | "super"
            | "switch"
            | "synchronized"
            | "template"
            | "this"
            | "thread_local"
            | "throw"
            | "throws"
            | "trait"
            | "transient"
            | "try"
            | "type"
            | "typedef"
            | "typename"
            | "typeof"
            | "uint"
            | "ulong"
            | "unchecked"
            | "union"
            | "unsafe"
            | "unsigned"
            | "unsized"
            | "use"
            | "using"
            | "ushort"
            | "var"
            | "virtual"
            | "void"
            | "volatile"
            | "where"
            | "while"
            | "with"
            | "yield"
    )
}

fn is_type_kind(kind: &str) -> bool {
    matches!(
        kind,
        "type_identifier" | "primitive_type" | "class_identifier" | "builtin_type"
    )
}

fn is_identifier_kind(kind: &str) -> bool {
    matches!(
        kind,
        "identifier"
            | "name"
            | "field_identifier"
            | "property_identifier"
            | "shorthand_property_identifier"
            | "private_property_identifier"
            | "attribute_name"
            | "property_name"
            | "class_name"
            | "id_name"
            | "tag_name"
            | "variable_name"
            | "command_name"
            | "function_name"
            | "label_name"
            | "package_identifier"
            | "alias_name"
            | "anchor_name"
            | "statement_identifier"
    )
}

fn is_property_kind(kind: &str) -> bool {
    matches!(
        kind,
        "property_identifier"
            | "field_identifier"
            | "attribute_name"
            | "property_name"
            | "shorthand_property_identifier"
            | "private_property_identifier"
    )
}

fn is_function_parent(kind: &str) -> bool {
    kind.contains("function")
        || kind.contains("call")
        || kind.contains("method")
        || kind.contains("constructor")
}

fn is_type_parent(kind: &str) -> bool {
    kind.contains("type")
        || kind.contains("class")
        || kind.contains("struct")
        || kind.contains("enum")
        || kind.contains("interface")
        || kind.contains("trait")
}

fn classify_direct(kind: &str, named: bool, leaf: bool) -> Option<Highlight> {
    if kind.contains("comment") {
        return Some(Highlight::Comment);
    }
    if is_string_kind(kind) {
        return Some(Highlight::String);
    }
    if is_number_kind(kind) {
        return Some(Highlight::Number);
    }
    if leaf && is_constant_kind(kind) {
        return Some(Highlight::Constant);
    }
    if !named && is_keyword(kind) {
        return Some(Highlight::Keyword);
    }
    if is_type_kind(kind) {
        return Some(Highlight::Type);
    }
    if is_property_kind(kind) {
        return Some(Highlight::Property);
    }
    None
}

fn classify_identifier_leaf(
    kind: &str,
    field: Option<&str>,
    parent: Option<&str>,
) -> Option<Highlight> {
    if !is_identifier_kind(kind) {
        return None;
    }
    match field {
        Some("name") => {
            if parent.is_some_and(is_function_parent) {
                return Some(Highlight::Function);
            }
            if parent.is_some_and(is_type_parent) {
                return Some(Highlight::Type);
            }
        }
        Some("function") | Some("method") | Some("callee") => return Some(Highlight::Function),
        Some("type") | Some("superclass") => return Some(Highlight::Type),
        _ => {}
    }
    if let Some(p) = parent {
        if is_function_parent(p) {
            return Some(Highlight::Function);
        }
        if is_type_parent(p) {
            return Some(Highlight::Type);
        }
    }
    None
}

pub struct CodeBlockWidget {
    code: String,
    highlights: Vec<(Range<usize>, Highlight)>,
    font_system: Rc<RefCell<FontSystem>>,
    glyph_cache: Rc<GlyphCache>,
    selection_key: Option<u64>,

    buffers: RefCell<Vec<(u32, Buffer)>>,
    theme_seen: Cell<u64>,
}

impl CodeBlockWidget {
    fn new(
        code: String,
        lang: Option<Lang>,
        fonts: &TextFonts,
        selection_key: Option<u64>,
    ) -> Self {
        let highlights = lang.map_or_else(Vec::new, |l| highlight(&code, l));
        Self {
            code,
            highlights,
            font_system: fonts.font_system.clone(),
            glyph_cache: fonts.glyph_cache.clone(),
            selection_key,
            buffers: RefCell::new(Vec::new()),
            theme_seen: Cell::new(requiem_ui::theme_generation()),
        }
    }

    fn build_buffer(&self, wrap: Wrap, width: Option<f32>) -> Buffer {
        let mut font_system = self.font_system.borrow_mut();
        let metrics = Metrics::new(CODE_FONT_SIZE, CODE_LINE_HEIGHT);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_wrap(wrap);
        buffer.set_size(width, None);

        let ink = colors::text_code();
        let default = Attrs::new()
            .family(Family::Name(GG_SANS_MONO_FAMILY))
            .color(CosmicColor::rgba(ink.r(), ink.g(), ink.b(), ink.a()));
        let mut spans: Vec<(&str, Attrs)> = Vec::with_capacity(self.highlights.len() + 1);
        let mut last = 0usize;
        for (range, h) in &self.highlights {
            if range.start > last {
                spans.push((&self.code[last..range.start], default.clone()));
            }
            let color = color_of(*h);
            spans.push((
                &self.code[range.clone()],
                Attrs::new()
                    .family(Family::Name(GG_SANS_MONO_FAMILY))
                    .color(CosmicColor::rgba(
                        color.r(),
                        color.g(),
                        color.b(),
                        color.a(),
                    )),
            ));
            last = range.end;
        }
        if last < self.code.len() {
            spans.push((&self.code[last..], default.clone()));
        }
        buffer.set_rich_text(spans, &default, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut font_system, false);
        buffer
    }

    fn with_wrapped_buffer<R>(&self, inner_width: f32, f: impl FnOnce(&Buffer) -> R) -> R {
        let width = inner_width.max(1.0).round();
        let key = width.to_bits();
        let theme = requiem_ui::theme_generation();
        let mut buffers = self.buffers.borrow_mut();
        if self.theme_seen.get() != theme {
            buffers.clear();
            self.theme_seen.set(theme);
        }
        let index = match buffers.iter().position(|(w, _)| *w == key) {
            Some(index) => index,
            None => {
                const CACHE_SLOTS: usize = 4;
                if buffers.len() >= CACHE_SLOTS {
                    buffers.remove(0);
                }
                buffers.push((key, self.build_buffer(Wrap::WordOrGlyph, Some(width))));
                buffers.len() - 1
            }
        };
        f(&buffers[index].1)
    }

    fn extent(buffer: &Buffer) -> (f32, f32) {
        buffer.layout_runs().fold((0.0f32, 0.0f32), |(w, h), run| {
            (w.max(run.line_w), h.max(run.line_top + run.line_height))
        })
    }

    fn natural_width(&self) -> f32 {
        Self::extent(&self.build_buffer(Wrap::None, None)).0
    }

    fn paint_text(&self, canvas: &Canvas, rect: AbsoluteRect) {
        let scale = current_scale_factor();
        let origin = (
            (rect.x + CODE_PADDING) * scale,
            (rect.y + CODE_PADDING) * scale,
        );
        let ink = colors::text_code();
        let default = CosmicColor::rgba(ink.r(), ink.g(), ink.b(), ink.a());
        let inner_width = (rect.width - 2.0 * CODE_PADDING).max(1.0);
        canvas.save();
        canvas.scale((1.0 / scale, 1.0 / scale));
        self.with_wrapped_buffer(inner_width, |buffer| {
            let mut font_system = self.font_system.borrow_mut();
            for run in buffer.layout_runs() {
                if let Some(id) = self.selection_key {
                    if let Some(range) = selection::range_for(id) {
                        if let Some((x, w)) = selection_span(&run, range) {
                            let mut paint = Paint::default();
                            paint.set_color(Color::from_argb(0x80, 0x58, 0x65, 0xf2));
                            canvas.draw_rect(
                                Rect::from_xywh(
                                    origin.0 + x * scale,
                                    origin.1 + run.line_top * scale,
                                    w * scale,
                                    run.line_height * scale,
                                ),
                                &paint,
                            );
                        }
                    }
                }
                for glyph in run.glyphs {
                    let physical = glyph.physical((0.0, run.line_y * scale), scale);
                    let color = glyph.color_opt.map_or(default, |c| c);
                    paint_glyph(
                        canvas,
                        &self.glyph_cache,
                        &mut font_system,
                        physical,
                        color,
                        origin,
                    );
                }
            }
        });
        canvas.restore();
    }
}

impl Widget for CodeBlockWidget {
    fn measure(&self, known: Size<Option<f32>>, available: Size<AvailableSpace>) -> Size<f32> {
        let width = match known.width {
            Some(w) => w,
            None => match available.width {
                AvailableSpace::Definite(w) => w,
                AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                    self.natural_width() + 2.0 * CODE_PADDING
                }
            },
        };

        let inner_width = (width - 2.0 * CODE_PADDING).max(1.0);
        let height = self.with_wrapped_buffer(inner_width, |buffer| Self::extent(buffer).1);
        Size {
            width,
            height: height + 2.0 * CODE_PADDING,
        }
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let rrect = RRect::new_rect_xy(bounds, CODE_RADIUS, CODE_RADIUS);
        canvas.save();
        canvas.clip_rrect(rrect, None, true);
        let mut bg = Paint::default();
        bg.set_anti_alias(true);
        bg.set_color(colors::background_code());
        canvas.draw_rect(bounds, &bg);
        let mut border = Paint::default();
        border.set_anti_alias(true);
        border.set_style(skia_safe::PaintStyle::Stroke);
        border.set_stroke_width(1.0);
        border.set_color(colors::live(colors::BORDER_NORMAL));
        canvas.draw_rrect(rrect, &border);
        self.paint_text(canvas, rect);
        canvas.restore();
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.selection_key()
            .map(|_| CursorIcon::Text)
    }

    fn selection_key(&self) -> Option<u64> {
        self.selection_key.filter(|_| !self.code.is_empty())
    }

    fn selection_offset_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> u64 {
        let inner = (rect.width - 2.0 * CODE_PADDING).max(1.0);
        self.with_wrapped_buffer(inner, |buffer| {
            offset_at(buffer, x - rect.x - CODE_PADDING, y - rect.y - CODE_PADDING)
        })
    }

    fn selection_end(&self, rect: AbsoluteRect) -> u64 {
        let inner = (rect.width - 2.0 * CODE_PADDING).max(1.0);
        self.with_wrapped_buffer(inner, |buffer| {
            buffer.layout_runs().last().map_or(0, |run| {
                encode(run.line_i, run.glyphs.last().map_or(0, |g| g.end))
            })
        })
    }

    fn selection_word(&self, rect: AbsoluteRect, offset: u64) -> (u64, u64) {
        let inner = (rect.width - 2.0 * CODE_PADDING).max(1.0);
        self.with_wrapped_buffer(inner, |buffer| word_range(buffer, offset))
    }

    fn selection_paragraph(&self, rect: AbsoluteRect, offset: u64) -> (u64, u64) {
        let inner = (rect.width - 2.0 * CODE_PADDING).max(1.0);
        self.with_wrapped_buffer(inner, |buffer| paragraph_range(buffer, offset))
    }

    fn selection_text(&self, rect: AbsoluteRect, range: (u64, u64)) -> String {
        let inner = (rect.width - 2.0 * CODE_PADDING).max(1.0);
        self.with_wrapped_buffer(inner, |buffer| selected_text(buffer, range))
    }
}

pub fn node(block: &CodeBlock, fonts: &TextFonts, selection_key: Option<u64>) -> Node {
    let lang = Lang::from_alias(&block.lang);
    Node::new(
        "div",
        CodeBlockWidget::new(block.code.clone(), lang, fonts, selection_key),
    )
    .style("margin: 4px 0")
}

fn encode(line: usize, index: usize) -> u64 {
    ((line as u64) << 32) | (index as u64 & 0xffff_ffff)
}

fn decode(offset: u64) -> (usize, usize) {
    ((offset >> 32) as usize, (offset & 0xffff_ffff) as usize)
}

fn selection_span(run: &cosmic_text::LayoutRun, range: (u64, u64)) -> Option<(f32, f32)> {
    let ((start_line, start_index), (end_line, end_index)) = (decode(range.0), decode(range.1));
    if range.0 == range.1 || run.line_i < start_line || run.line_i > end_line {
        return None;
    }
    let from = if run.line_i == start_line {
        start_index
    } else {
        0
    };
    let to = if run.line_i == end_line {
        end_index
    } else {
        usize::MAX
    };
    let mut span: Option<(f32, f32)> = None;
    for glyph in run.glyphs {
        if glyph.start < to && glyph.end > from {
            let (left, right) = (glyph.x, glyph.x + glyph.w);
            span = Some(match span {
                Some((l, r)) => (l.min(left), r.max(right)),
                None => (left, right),
            });
        }
    }
    span.map(|(l, r)| (l, r - l))
}

fn offset_at(buffer: &Buffer, local_x: f32, local_y: f32) -> u64 {
    let runs: Vec<_> = buffer.layout_runs().collect();
    let Some(first) = runs.first() else { return 0 };
    let run = runs
        .iter()
        .find(|r| local_y >= r.line_top && local_y < r.line_top + r.line_height)
        .unwrap_or(if local_y < first.line_top {
            first
        } else {
            runs.last().unwrap_or(first)
        });
    let mut index = run.glyphs.first().map_or(0, |g| g.start);
    for glyph in run.glyphs {
        if local_x < glyph.x + glyph.w / 2.0 {
            return encode(run.line_i, glyph.start);
        }
        index = glyph.end;
    }
    encode(run.line_i, index)
}

fn word_range(buffer: &Buffer, offset: u64) -> (u64, u64) {
    let (line, index) = decode(offset);
    let Some(text) = buffer.lines.get(line).map(|l| l.text()) else {
        return (offset, offset);
    };
    for (start, word) in text.split_word_bound_indices() {
        if index >= start && index < start + word.len() {
            return (encode(line, start), encode(line, start + word.len()));
        }
    }
    (offset, offset)
}

fn paragraph_range(buffer: &Buffer, offset: u64) -> (u64, u64) {
    let (line, _) = decode(offset);
    match buffer.lines.get(line) {
        Some(l) => (encode(line, 0), encode(line, l.text().len())),
        None => (offset, offset),
    }
}

fn selected_text(buffer: &Buffer, range: (u64, u64)) -> String {
    let ((start_line, start_index), (end_line, end_index)) = (decode(range.0), decode(range.1));
    let mut out = String::new();
    for (line, text) in buffer.lines.iter().map(|l| l.text()).enumerate() {
        if line < start_line || line > end_line {
            continue;
        }
        let from = if line == start_line {
            start_index.min(text.len())
        } else {
            0
        };
        let to = if line == end_line {
            end_index.min(text.len())
        } else {
            text.len()
        };
        out.push_str(&text[from..to]);
        if line < end_line {
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use requiem_ui::font_system::TextFonts;
    use taffy::{AvailableSpace, Size};

    fn code(s: &str) -> Vec<Segment> {
        segments(s)
    }

    fn as_code(segment: &Segment) -> Option<&CodeBlock> {
        match segment {
            Segment::Code(b) => Some(b),
            _ => None,
        }
    }

    #[test]
    fn plain_text_is_one_text_segment() {
        let segs = code("hello world");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], Segment::Text(r) if *r == (0..11)));
    }

    #[test]
    fn unicode_around_a_fence_is_byte_safe() {
        let text = "😀 hello\n```rust\nlet café = \"✓\";\n```\n世界";
        let segs = code(text);
        assert_eq!(segs.len(), 3);
        assert!(matches!(&segs[0], Segment::Text(r) if &text[r.clone()] == "😀 hello"));
        let block = as_code(&segs[1]).unwrap();
        assert_eq!(block.lang, "rust");
        assert_eq!(block.code, "let café = \"✓\";");
        assert!(matches!(&segs[2], Segment::Text(r) if &text[r.clone()] == "世界"));
    }

    #[test]
    fn fenced_with_language() {
        let segs = code("```rust\nfn main() {}\n```");
        assert_eq!(segs.len(), 1);
        let block = as_code(&segs[0]).unwrap();
        assert_eq!(block.lang, "rust");
        assert_eq!(block.code, "fn main() {}");
    }

    #[test]
    fn fenced_without_language() {
        let segs = code("```\ncode\n```");
        let block = as_code(&segs[0]).unwrap();
        assert_eq!(block.lang, "");
        assert_eq!(block.code, "code");
    }

    #[test]
    fn single_line_fence_has_no_language() {
        let segs = code("```code```");
        let block = as_code(&segs[0]).unwrap();
        assert_eq!(block.lang, "");
        assert_eq!(block.code, "code");
    }

    #[test]
    fn blank_line_after_language_is_stripped() {
        let segs = code("```py\n\nprint(1)\n\n```");
        let block = as_code(&segs[0]).unwrap();
        assert_eq!(block.lang, "py");
        assert_eq!(block.code, "print(1)");
    }

    #[test]
    fn text_around_a_block_splits() {
        let text = "before\n```js\nlet x = 1;\n```\nafter";
        let segs = code(text);
        assert_eq!(segs.len(), 3);
        assert!(matches!(&segs[0], Segment::Text(r) if &text[r.clone()] == "before"));
        let block = as_code(&segs[1]).unwrap();
        assert_eq!(block.lang, "js");
        assert_eq!(block.code, "let x = 1;");
        assert!(matches!(&segs[2], Segment::Text(r) if &text[r.clone()] == "after"));
    }

    #[test]
    fn blank_line_before_block_is_preserved() {
        let text = "a\n\n```code```";
        let segs = code(text);
        assert_eq!(segs.len(), 2);
        assert!(matches!(&segs[0], Segment::Text(r) if &text[r.clone()] == "a\n"));
        assert_eq!(as_code(&segs[1]).unwrap().code, "code");
    }

    #[test]
    fn unclosed_fence_stays_text() {
        let segs = code("```rust\nno closing fence");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], Segment::Text(_)));
    }

    #[test]
    fn fence_must_start_a_line() {
        let segs = code("say ```js\nx\n```");
        assert_eq!(segs.len(), 1);
        assert!(matches!(&segs[0], Segment::Text(_)));
    }

    #[test]
    fn language_is_case_insensitive() {
        let segs = code("```RUST\nx\n```");
        let block = as_code(&segs[0]).unwrap();
        assert_eq!(block.lang, "rust");
    }

    #[test]
    fn rust_highlights_keywords_and_strings() {
        let code = "fn main() {\n    let s = \"hi\";\n}";
        let spans = highlight(code, Lang::Rust);
        assert!(
            spans
                .iter()
                .any(|(r, h)| *h == Highlight::Keyword && &code[r.clone()] == "fn")
        );
        assert!(
            spans
                .iter()
                .any(|(r, h)| *h == Highlight::String && &code[r.clone()] == "\"hi\"")
        );

        for pair in spans.windows(2) {
            assert!(pair[0].0.end <= pair[1].0.start);
        }
    }

    #[test]
    fn macro_string_still_highlights() {
        let code = "println!(\"hi\");";
        let spans = highlight(code, Lang::Rust);
        assert!(
            spans
                .iter()
                .any(|(r, h)| *h == Highlight::String && &code[r.clone()] == "\"hi\"")
        );
    }

    #[test]
    fn python_highlights_tokens_not_the_whole_tree() {
        let code = "# caf\u{e9} \u{2713}\ndef f(x):\n    return lambda y: x * 2 if y else None";
        let spans: Vec<(&str, Highlight)> = highlight(code, Lang::Python)
            .into_iter()
            .map(|(r, h)| (&code[r], h))
            .collect();
        assert!(
            spans.iter().all(|(text, _)| text.len() < code.len() / 2),
            "{spans:?}"
        );
        assert!(
            spans.contains(&("# caf\u{e9} \u{2713}", Highlight::Comment)),
            "{spans:?}"
        );
        for keyword in ["def", "return", "lambda", "if", "else"] {
            assert!(
                spans.contains(&(keyword, Highlight::Keyword)),
                "{keyword}: {spans:?}"
            );
        }
        assert!(spans.contains(&("f", Highlight::Function)), "{spans:?}");
        assert!(spans.contains(&("2", Highlight::Number)), "{spans:?}");
        assert!(spans.contains(&("None", Highlight::Constant)), "{spans:?}");
    }

    #[test]
    fn unknown_language_highlights_nothing() {
        assert!(Lang::from_alias("madeup").is_none());
    }

    #[test]
    fn diff_highlights_add_and_remove_lines() {
        let code = "+added\n-removed\ncontext\n";
        let spans = highlight(code, Lang::Diff);
        assert!(
            spans
                .iter()
                .any(|(r, h)| *h == Highlight::Addition && &code[r.clone()] == "+added")
        );
        assert!(
            spans
                .iter()
                .any(|(r, h)| *h == Highlight::Deletion && &code[r.clone()] == "-removed")
        );
    }

    #[test]
    fn widget_shapes_and_measures() {
        let fonts = TextFonts::new(0);
        let block = CodeBlock {
            lang: "rust".to_string(),
            code: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string(),
        };
        let lang = Lang::from_alias(&block.lang);
        let widget = CodeBlockWidget::new(block.code.clone(), lang, &fonts, None);
        let known = Size {
            width: None,
            height: None,
        };
        let wide = widget.measure(
            known,
            Size {
                width: AvailableSpace::Definite(600.0),
                height: AvailableSpace::Definite(0.0),
            },
        );
        let narrow = widget.measure(
            known,
            Size {
                width: AvailableSpace::Definite(120.0),
                height: AvailableSpace::Definite(0.0),
            },
        );
        assert_eq!(wide.width, 600.0);
        assert_eq!(narrow.width, 120.0);
        assert!(
            narrow.height > wide.height,
            "narrow code should wrap: wide={} narrow={}",
            wide.height,
            narrow.height
        );
        assert!(
            widget.with_wrapped_buffer(120.0 - 2.0 * CODE_PADDING, |buffer| buffer
                .layout_runs()
                .count())
                > 1
        );
    }
}
