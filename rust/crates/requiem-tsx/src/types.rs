use std::fmt::Write;
use std::mem;

use requiem_ui::typography::TextVariant;

use requiem_i18n::{ArgKind, Message};

use crate::actions::ActionSpec;
use crate::host::HostSpec;
use crate::state::StateQueries;
use crate::components::{
    BUTTON_SIZES, BUTTON_VARIANTS, STACK_ALIGNS, STACK_DIRECTIONS, STACK_JUSTIFIES, TEXT_COLORS,
    APP_MODULE, UI_MODULE, names,
};
use requiem_ui::icons;
use crate::css::{CssModules, camel};

fn union(name: &str, values: impl IntoIterator<Item = String>) -> String {
    let mut out = format!("  export type {name} =");
    for value in values {
        let _ = write!(out, "\n    | \"{value}\"");
    }
    out.push_str(";\n");
    out
}

pub struct DeclarationFile {
    pub path: String,
    pub text: String,
}

pub fn declarations(
    css: &CssModules,
    actions: &[ActionSpec],
    state: &StateQueries,
    host: &[HostSpec],
    messages: &[Message],
) -> String {
    declaration_files(css, actions, state, host, messages)
        .into_iter()
        .map(|file| file.text)
        .collect::<Vec<_>>()
        .join("
")
}

pub fn declaration_files(
    css: &CssModules,
    actions: &[ActionSpec],
    state: &StateQueries,
    host: &[HostSpec],
    messages: &[Message],
) -> Vec<DeclarationFile> {
    let mut files = Vec::new();
    let mut out = String::new();

    let _ = writeln!(out, "declare module \"{UI_MODULE}\" {{");
    out.push_str(&union(
        "TextVariant",
        TextVariant::named().into_iter().map(|(name, _)| name),
    ));
    out.push_str(&union(
        "TextColor",
        TEXT_COLORS.iter().map(|(name, _)| name.to_string()),
    ));
    out.push_str(&union(
        "ButtonVariant",
        BUTTON_VARIANTS.iter().map(|name| name.to_string()),
    ));
    out.push_str(&union(
        "StackDirection",
        names(STACK_DIRECTIONS).into_iter().map(String::from),
    ));
    out.push_str(&union(
        "StackAlign",
        names(STACK_ALIGNS).into_iter().map(String::from),
    ));
    out.push_str(&union(
        "StackJustify",
        names(STACK_JUSTIFIES).into_iter().map(String::from),
    ));
    out.push_str(&union(
        "ButtonSize",
        BUTTON_SIZES.iter().map(|name| name.to_string()),
    ));
    out.push_str(
        "
  export type TextChild = string | number | boolean | null | undefined;

  export interface TextProps {
    variant: TextVariant;
    color?: TextColor;
    lineClamp?: number;
    center?: boolean;
    className?: string;
    children?: TextChild | TextChild[];
  }

  export interface ClickEvent {
    shiftKey: boolean;
  }

  export interface ContextMenuEvent {
    x: number;
    y: number;
  }

  export interface ButtonProps {
    variant?: ButtonVariant;
    size?: ButtonSize;
    className?: string;
    disabled?: boolean;
    fullWidth?: boolean;
    onClick?: (event: ClickEvent) => void;
    onContextMenu?: (event: ContextMenuEvent) => void;
    children?: TextChild | TextChild[] | JSX.Children;
  }

  export interface IconProps {
    width?: number;
    height?: number;
    color?: TextColor;
    className?: string;
  }

  export interface ChevronProps {
    open: boolean;
    color?: TextColor;
    width?: number;
    height?: number;
    className?: string;
  }

  export interface StackProps {
    direction?: StackDirection;
    align?: StackAlign;
    justify?: StackJustify;
    wrap?: boolean;
    gap?: number;
    padding?: number;
    fullWidth?: boolean;
    className?: string;
    children?: JSX.Children;
  }

  export interface ScrollEvent {
    offset: number;
    max: number;
  }

  export interface ScrollPositionValue {
    pinned: boolean;
    key: string | null;
    offset: number;
    height: number;
  }

  export interface MeasureEvent {
    keys: string[];
    heights: number[];
  }

  export interface ScrollViewProps {
    className?: string;
    itemHeight?: number | string;
    itemGap?: number;
    initialOffset?: number;
    onScroll?: (event: ScrollEvent) => void;
    onNearEnd?: () => void;
    position?: ScrollPositionValue;
    pinThreshold?: number;
    onPosition?: (position: ScrollPositionValue) => void;
    onMeasure?: (event: MeasureEvent) => void;
    children?: JSX.Children;
  }

  export function Text(props: TextProps): JSX.Element;
  export function Button(props: ButtonProps): JSX.Element;
  export function ScrollView(props: ScrollViewProps): JSX.Element;
  export function Stack(props: StackProps): JSX.Element;
  export function Chevron(props: ChevronProps): JSX.Element;
",
    );
    out.push('\n');
    for (component, _) in icons::ALL {
        let _ = writeln!(out, "  export function {component}(props: IconProps): JSX.Element;");
    }
    out.push_str("}\n");
    files.push(DeclarationFile {
        path: "ui.d.ts".into(),
        text: mem::take(&mut out),
    });
    files.push(DeclarationFile {
        path: "react.d.ts".into(),
        text: "declare module \"react\" {
  export function useState<T>(initial: T): [T, (next: T | ((previous: T) => T)) => void];
}
"
        .into(),
    });
    out.push_str(
        "declare const elementBrand: unique symbol;

declare namespace JSX {
  interface Element {
    readonly [elementBrand]: true;
  }
  type Children = Element | boolean | null | undefined | Children[];
  interface IntrinsicAttributes {
    key?: string | number;
  }
  interface ElementChildrenAttribute {
    children: {};
  }
  interface IntrinsicElements {
    div: {
      key?: string | number;
      className?: string;
      style?: string;
      onClick?: () => void;
      onContextMenu?: (event: { x: number; y: number }) => void;
      children?: Children;
    };
  }
}
",
    );
    files.push(DeclarationFile {
        path: "jsx.d.ts".into(),
        text: mem::take(&mut out),
    });

    out.push_str("interface Translations {\n");
    for message in messages {
        let args = if message.args.is_empty() {
            "undefined".to_string()
        } else {
            let fields: Vec<String> = message
                .args
                .iter()
                .map(|arg| {
                    let ts = match arg.kind {
                        ArgKind::Number => "number",
                        ArgKind::Text => "string",
                        ArgKind::Any => "string | number",
                    };
                    let plain = arg
                        .name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_');
                    if plain {
                        format!("{}: {ts}", arg.name)
                    } else {
                        format!("\"{}\": {ts}", arg.name)
                    }
                })
                .collect();
            format!("{{ {} }}", fields.join("; "))
        };
        let _ = writeln!(out, "  \"{}\": {args};", message.key);
    }
    out.push_str(
        "}

declare function t<K extends keyof Translations>(
  key: K,
  ...args: Translations[K] extends undefined ? [] : [args: Translations[K]]
): string;
",
    );
    files.push(DeclarationFile {
        path: "i18n.d.ts".into(),
        text: mem::take(&mut out),
    });

    out.push_str("declare const actions: {\n");
    for action in actions {
        let _ = writeln!(out, "  {};", state.qualify(&action.signature()));
    }
    out.push_str("};\n");
    files.push(DeclarationFile {
        path: "actions.d.ts".into(),
        text: mem::take(&mut out),
    });
    let (types, queries) = state.declaration_parts();
    files.push(DeclarationFile {
        path: "types.d.ts".into(),
        text: types.trim_start().to_string(),
    });
    files.push(DeclarationFile {
        path: "state.d.ts".into(),
        text: queries.trim_start().to_string(),
    });

    if !host.is_empty() {
        let _ = writeln!(out, "declare module \"{APP_MODULE}\" {{");
        for component in host {
            let _ = writeln!(out, "  export interface {}Props {{", component.name);
            for prop in &component.props {
                let optional = if prop.required { "" } else { "?" };
                let _ = writeln!(out, "    {}{optional}: {};", prop.name, state.qualify(&prop.ts));
            }
            out.push_str("  }\n");
        }
        out.push('\n');
        for component in host {
            let _ = writeln!(
                out,
                "  export function {0}(props: {0}Props): JSX.Element;",
                component.name
            );
        }
        out.push_str("}\n");
        files.push(DeclarationFile {
            path: "app.d.ts".into(),
            text: mem::take(&mut out),
        });
    }

    for stem in css.stems() {
        let _ = writeln!(out, "declare module \"*/{stem}.module.css\" {{");
        out.push_str("  const classes: {\n");
        if let Some(module) = css.module(stem) {
            for class in module.keys() {
                let _ = writeln!(out, "    readonly \"{class}\": string;");
                let camel = camel(class);
                if camel != *class {
                    let _ = writeln!(out, "    readonly {camel}: string;");
                }
            }
        }
        out.push_str("  };\n  export default classes;\n}\n");
        files.push(DeclarationFile {
            path: format!("css/{stem}.module.css.d.ts"),
            text: mem::take(&mut out),
        });
    }
    files
}
