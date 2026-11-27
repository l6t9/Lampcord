use std::rc::Rc;

use requiem_ui::components::{Align, ButtonSize, ButtonVariant, Direction, Justify};
use requiem_ui::icon_handle::Icon;
use requiem_ui::typography::TextVariant;
use skia_safe::Color;

use crate::value::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    ToUpperCase,
    ToLowerCase,
    Trim,
    Includes,
    Join,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Lit(Value),
    Var(usize),
    Member(Box<Expr>, Rc<str>),
    Index(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Neg(Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Nullish(Box<Expr>, Box<Expr>),
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
    Template(Vec<Rc<str>>, Vec<Expr>),
    Method(Box<Expr>, Method, Vec<Expr>),
    List(Vec<Expr>),
    Obj(Vec<(Rc<str>, Expr)>),
    Closure {
        params: usize,
        body: Rc<Vec<Stmt>>,
    },
    Query {
        path: Rc<str>,
        args: Vec<Expr>,
    },
    Translate {
        key: Rc<str>,
        args: Vec<(&'static str, Expr)>,
    },
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Set {
        slot: usize,
        value: Expr,
    },
    Update {
        slot: usize,
        param: usize,
        value: Expr,
    },
    Action {
        name: Rc<str>,
        args: Vec<Expr>,
    },
    Call {
        callee: Expr,
        args: Vec<Expr>,
    },
    If {
        cond: Expr,
        then: Vec<Stmt>,
        otherwise: Vec<Stmt>,
    },
}

#[derive(Clone, Debug)]
pub enum Attr {
    None,
    Static(Rc<str>),
    Dyn(Expr),
}

#[derive(Clone, Debug)]
pub enum Element {
    Div(Div),
    Text(TextEl),
    Button(ButtonEl),
    Icon(IconEl),
    Chevron(ChevronEl),
    Host(HostEl),
    Scroll(ScrollEl),
    Stack(StackEl),
    Use(UseEl),
}

#[derive(Clone, Debug)]
pub struct HostEl {
    pub name: Rc<str>,
    pub props: Vec<(Rc<str>, Expr)>,
}

#[derive(Clone, Debug)]
pub struct ChevronEl {
    pub open: Expr,
    pub color: Color,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub class: Option<Rc<str>>,
}

#[derive(Clone, Debug)]
pub struct IconEl {
    pub icon: Icon,
    pub color: Color,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub class: Option<Rc<str>>,
}

#[derive(Clone, Debug)]
pub struct StackEl {
    pub direction: Direction,
    pub align: Align,
    pub justify: Justify,
    pub wrap: bool,
    pub gap: f32,
    pub padding: f32,
    pub full_width: bool,
    pub class: Option<Rc<str>>,
    pub children: Vec<Child>,
}

#[derive(Clone, Debug)]
pub struct ScrollEl {
    pub class: Option<Rc<str>>,
    pub window: Option<Window>,
    pub initial_offset: Option<Expr>,
    pub on_scroll: Option<Handler>,
    pub on_near_end: Option<Rc<Vec<Stmt>>>,
    pub chat: Option<ChatSpec>,
    pub children: Vec<Child>,
}

#[derive(Clone, Debug)]
pub struct ChatSpec {
    pub position: Expr,
    pub pin_threshold: f32,
    pub on_position: Option<Handler>,
    pub on_measure: Option<Handler>,
}

#[derive(Clone, Debug)]
pub enum ItemHeight {
    Fixed(f32),
    Field(Rc<str>),
}

#[derive(Clone, Debug)]
pub struct Window {
    pub height: ItemHeight,
    pub gap: f32,
}

#[derive(Clone, Debug)]
pub struct UseEl {
    pub name: Rc<str>,
    pub props: Vec<(Rc<str>, Expr)>,
}

#[derive(Clone, Debug)]
pub struct Handler {
    pub body: Rc<Vec<Stmt>>,
    pub event: bool,
}

#[derive(Clone, Debug)]
pub struct Div {
    pub class: Attr,
    pub style: Attr,
    pub on_click: Option<Rc<Vec<Stmt>>>,
    pub on_context_menu: Option<Handler>,
    pub children: Vec<Child>,
}

#[derive(Clone, Debug)]
pub enum Part {
    Lit(Rc<str>),
    Dyn(Expr),
}

#[derive(Clone, Debug)]
pub struct TextEl {
    pub variant: TextVariant,
    pub color: Color,
    pub line_clamp: Option<u32>,
    pub center: bool,
    pub class: Option<Rc<str>>,
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug)]
pub struct ButtonEl {
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub label: Vec<Part>,
    pub class: Option<Rc<str>>,
    pub disabled: bool,
    pub full_width: bool,
    pub on_click: Option<Rc<Vec<Stmt>>>,
    pub on_click_event: bool,
    pub on_context_menu: Option<Handler>,
    pub children: Vec<Child>,
}

#[derive(Clone, Debug)]
pub enum Child {
    Element(Element),
    If {
        cond: Expr,
        then: Vec<Element>,
        otherwise: Vec<Element>,
    },
    Each {
        list: Expr,
        key: Expr,
        item: usize,
        body: Box<Element>,
    },
}

#[derive(Clone, Debug)]
pub enum Decl {
    State { init: Expr },
    Derived { expr: Expr },
}

#[derive(Clone, Debug)]
pub struct Component {
    pub name: String,
    pub props: Vec<Rc<str>>,
    pub decls: Vec<Decl>,
    pub root: Element,
}
