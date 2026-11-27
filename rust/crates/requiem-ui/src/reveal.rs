use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    static REQUESTED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

pub fn request(key: &str) {
    REQUESTED.with(|requested| requested.borrow_mut().insert(key.to_string()));
}

pub(crate) fn take(key: &str) -> bool {
    REQUESTED.with(|requested| requested.borrow_mut().remove(key))
}

pub(crate) fn nearest_offset(offset: f32, top: f32, height: f32, view: f32, padding: f32) -> f32 {
    if height > view - padding * 2.0 || top < padding {
        offset + top - padding
    } else if top + height > view - padding {
        offset + top + height - (view - padding)
    } else {
        offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolls_the_least_that_shows_the_span() {
        assert_eq!(
            nearest_offset(100.0, 200.0, 50.0, 400.0, 12.0),
            100.0,
            "already in view"
        );
        assert_eq!(
            nearest_offset(100.0, 380.0, 50.0, 400.0, 12.0),
            142.0,
            "below: bottom lands 12px up from the edge"
        );
        assert_eq!(
            nearest_offset(100.0, -30.0, 50.0, 400.0, 12.0),
            58.0,
            "above: top lands 12px down"
        );
        assert_eq!(
            nearest_offset(0.0, 100.0, 500.0, 400.0, 12.0),
            88.0,
            "taller than the view: top aligned"
        );
    }

    #[test]
    fn a_request_is_consumed() {
        request("a-key");
        assert!(take("a-key"));
        assert!(!take("a-key"));
    }
}
