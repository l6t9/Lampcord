use crate::svg_icon;
use std::fmt as std_fmt;
use std::rc;
use std::sync::OnceLock;

type Resolver = Box<dyn Fn(&str) -> Option<&'static str> + Send + Sync>;

static RESOLVER: OnceLock<Resolver> = OnceLock::new();

pub fn install_resolver(
    resolver: impl Fn(&str) -> Option<&'static str> + Send + Sync + 'static,
) -> bool {
    RESOLVER.set(Box::new(resolver)).is_ok()
}

static SVG_RESOLVER: OnceLock<Resolver> = OnceLock::new();

pub fn install_svg_resolver(
    resolver: impl Fn(&str) -> Option<&'static str> + Send + Sync + 'static,
) -> bool {
    SVG_RESOLVER.set(Box::new(resolver)).is_ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Named(&'static str),

    Local(&'static str),
}

impl Icon {
    pub fn path(&self) -> &'static str {
        match self {
            Icon::Local(data) => data,
            Icon::Named(name) => RESOLVER
                .get()
                .and_then(|resolve| resolve(name))
                .unwrap_or(""),
        }
    }
}

impl Icon {
    pub fn svg(&self) -> Option<&'static str> {
        match self {
            Icon::Local(_) => None,
            Icon::Named(name) => SVG_RESOLVER.get().and_then(|resolve| resolve(name)),
        }
    }
}

impl std_fmt::Display for Icon {
    fn fmt(&self, f: &mut std_fmt::Formatter<'_>) -> std_fmt::Result {
        f.write_str(self.path())
    }
}

impl AsRef<str> for Icon {
    fn as_ref(&self) -> &str {
        self.path()
    }
}

impl From<Icon> for String {
    fn from(icon: Icon) -> String {
        icon.path().to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconSpec {
    Handle(Icon),
    Path(String),

    Svg(String),
}

impl From<Icon> for IconSpec {
    fn from(icon: Icon) -> Self {
        IconSpec::Handle(icon)
    }
}

impl From<String> for IconSpec {
    fn from(path: String) -> Self {
        IconSpec::Path(path)
    }
}

impl From<&str> for IconSpec {
    fn from(path: &str) -> Self {
        IconSpec::Path(path.to_string())
    }
}

impl IconPath for IconSpec {
    fn art(&self) -> Option<rc::Rc<svg_icon::ParsedSvg>> {
        match self {
            IconSpec::Handle(icon) => icon.art(),
            IconSpec::Path(path) => svg_icon::from_path_data(path),
            IconSpec::Svg(svg) => svg_icon::parse(svg),
        }
    }
}

impl IconPath for &IconSpec {
    fn art(&self) -> Option<rc::Rc<svg_icon::ParsedSvg>> {
        (*self).art()
    }
}

pub trait IconPath {
    fn art(&self) -> Option<rc::Rc<svg_icon::ParsedSvg>>;
}

impl IconPath for Icon {
    fn art(&self) -> Option<rc::Rc<svg_icon::ParsedSvg>> {
        match self.svg() {
            Some(svg) => svg_icon::parse(svg),
            None => svg_icon::from_path_data(self.path()),
        }
    }
}

impl IconPath for &Icon {
    fn art(&self) -> Option<rc::Rc<svg_icon::ParsedSvg>> {
        (*self).art()
    }
}

impl IconPath for &str {
    fn art(&self) -> Option<rc::Rc<svg_icon::ParsedSvg>> {
        svg_icon::from_path_data(self)
    }
}

impl IconPath for String {
    fn art(&self) -> Option<rc::Rc<svg_icon::ParsedSvg>> {
        svg_icon::from_path_data(self)
    }
}

impl IconPath for &String {
    fn art(&self) -> Option<rc::Rc<svg_icon::ParsedSvg>> {
        svg_icon::from_path_data(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_icons_need_no_resolver_and_named_ones_degrade_to_empty() {
        assert_eq!(Icon::Local("M0 0h24").path(), "M0 0h24");
        assert_eq!(Icon::Named("icon/unknown").path(), "");
        assert_eq!(String::from(Icon::Local("M2 2")), "M2 2");
    }
}
