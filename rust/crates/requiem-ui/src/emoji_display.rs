use std::borrow::Cow;
use std::sync::OnceLock;

pub type Filter = for<'a> fn(&'a str) -> Cow<'a, str>;

static FILTER: OnceLock<Filter> = OnceLock::new();

pub fn install(filter: Filter) {
    let _ = FILTER.set(filter);
}

pub fn apply(text: &str) -> Cow<'_, str> {
    match FILTER.get() {
        Some(filter) => filter(text),
        None => Cow::Borrowed(text),
    }
}
