use std::cell::RefCell;
use std::collections::HashMap;

use super::values::parse_length_px;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Linear {
    pub pct: f32,
    pub px: f32,
}

#[derive(Clone, Copy)]
struct Val {
    lin: Linear,
    unitless: bool,
}

impl Val {
    fn scale(self, k: f32) -> Val {
        Val {
            lin: Linear {
                pct: self.lin.pct * k,
                px: self.lin.px * k,
            },
            unitless: self.unitless,
        }
    }
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
}

impl Parser<'_> {
    fn peek(&mut self) -> Option<char> {
        while self.s[self.i..].starts_with(char::is_whitespace) {
            self.i += 1;
        }
        self.s[self.i..].chars().next()
    }

    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.i += c.len_utf8();
            true
        } else {
            false
        }
    }

    fn expr(&mut self) -> Option<Val> {
        let mut acc = self.term()?;
        loop {
            let sign = match self.peek() {
                Some('+') => 1.0,
                Some('-') => -1.0,
                _ => return Some(acc),
            };
            self.i += 1;
            let rhs = self.term()?;
            if acc.unitless != rhs.unitless {
                return None;
            }
            acc = Val {
                lin: Linear {
                    pct: acc.lin.pct + sign * rhs.lin.pct,
                    px: acc.lin.px + sign * rhs.lin.px,
                },
                unitless: acc.unitless,
            };
        }
    }

    fn term(&mut self) -> Option<Val> {
        let mut acc = self.factor()?;
        loop {
            match self.peek() {
                Some('*') => {
                    self.i += 1;
                    let rhs = self.factor()?;
                    acc = match (acc.unitless, rhs.unitless) {
                        (_, true) => acc.scale(rhs.lin.px),
                        (true, false) => rhs.scale(acc.lin.px),
                        (false, false) => return None,
                    };
                }
                Some('/') => {
                    self.i += 1;
                    let rhs = self.factor()?;
                    if !rhs.unitless || rhs.lin.px == 0.0 {
                        return None;
                    }
                    acc = acc.scale(1.0 / rhs.lin.px);
                }
                _ => return Some(acc),
            }
        }
    }

    fn factor(&mut self) -> Option<Val> {
        self.peek();
        if self.s[self.i..].starts_with("calc(") {
            self.i += "calc(".len();
            return self.group();
        }
        if self.eat('(') {
            return self.group();
        }
        let start = self.i;
        while let Some(c) = self.s[self.i..].chars().next() {
            let sign = self.i == start && matches!(c, '+' | '-');
            if !(sign || c.is_ascii_alphanumeric() || c == '.' || c == '%') {
                break;
            }
            self.i += c.len_utf8();
        }
        let token = &self.s[start..self.i];
        if let Some(pct) = token.strip_suffix('%') {
            let pct = pct.parse::<f32>().ok()?;
            return Some(Val {
                lin: Linear {
                    pct: pct / 100.0,
                    px: 0.0,
                },
                unitless: false,
            });
        }
        if let Ok(n) = token.parse::<f32>() {
            return Some(Val {
                lin: Linear { pct: 0.0, px: n },
                unitless: true,
            });
        }
        parse_length_px(token).map(|px| Val {
            lin: Linear { pct: 0.0, px },
            unitless: false,
        })
    }

    fn group(&mut self) -> Option<Val> {
        let inner = self.expr()?;
        self.eat(')').then_some(inner)
    }
}

pub fn parse_calc(value: &str) -> Option<Linear> {
    let value = value.trim();
    if !value.starts_with("calc(") {
        return None;
    }
    let mut parser = Parser { s: value, i: 0 };
    let result = parser.factor()?;
    (parser.peek().is_none() && !result.unitless).then_some(result.lin)
}

thread_local! {
    static INTERNED: RefCell<HashMap<(u32, u32), &'static Linear>> = RefCell::new(HashMap::new());
}

pub fn intern(lin: Linear) -> *const () {
    INTERNED.with(|map| {
        let stored: &'static Linear = map
            .borrow_mut()
            .entry((lin.pct.to_bits(), lin.px.to_bits()))
            .or_insert_with(|| Box::leak(Box::new(lin)));
        stored as *const Linear as *const ()
    })
}

pub fn resolve(handle: *const (), basis: f32) -> f32 {
    // SAFETY: `handle` was produced by `intern`, which leaks a `Linear` and returns a pointer to
    // it, so it is valid, aligned and never freed.
    let lin = unsafe { &*(handle as *const Linear) };
    lin.px + lin.pct * basis
}

pub fn split_tokens(value: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let (mut depth, mut start) = (0usize, None);
    for (i, c) in value.char_indices() {
        match c {
            '(' => {
                depth += 1;
                start.get_or_insert(i);
            }
            ')' => depth = depth.saturating_sub(1),
            c if c.is_whitespace() && depth == 0 => {
                if let Some(s) = start.take() {
                    tokens.push(&value[s..i]);
                }
            }
            _ => {
                start.get_or_insert(i);
            }
        }
    }
    if let Some(s) = start {
        tokens.push(&value[s..]);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lin(pct: f32, px: f32) -> Option<Linear> {
        Some(Linear { pct, px })
    }

    #[test]
    fn evaluates_mixed_expressions() {
        assert_eq!(parse_calc("calc(100% - 20px)"), lin(1.0, -20.0));
        assert_eq!(parse_calc("calc(4px - 2px)"), lin(0.0, 2.0));
        assert_eq!(parse_calc("calc(2 * (10px + 5px))"), lin(0.0, 30.0));
        assert_eq!(parse_calc("calc((100% - 10px) / 2)"), lin(0.5, -5.0));
        assert_eq!(parse_calc("calc(50% + calc(1px * 3))"), lin(0.5, 3.0));
        assert_eq!(parse_calc("calc(-10px + 100%)"), lin(1.0, -10.0));
    }

    #[test]
    fn rejects_non_linear_and_malformed_values() {
        assert_eq!(parse_calc("calc(1px * 2px)"), None);
        assert_eq!(parse_calc("calc(10px / 0)"), None);
        assert_eq!(
            parse_calc("calc(10px + 2)"),
            None,
            "a bare number is not a length"
        );
        assert_eq!(parse_calc("calc(10)"), None);
        assert_eq!(parse_calc("calc(10px"), None);
        assert_eq!(parse_calc("calc(1px) calc(2px)"), None);
        assert_eq!(parse_calc("10px"), None);
        assert_eq!(parse_calc("calc(min(1px, 2px))"), None);
    }

    #[test]
    fn handles_resolve_against_the_basis() {
        let handle = intern(Linear {
            pct: 0.5,
            px: -10.0,
        });
        assert_eq!(resolve(handle, 200.0), 90.0);
        assert_eq!(
            intern(Linear {
                pct: 0.5,
                px: -10.0
            }),
            handle,
            "deduplicated"
        );
    }

    #[test]
    fn splits_tokens_outside_parentheses_only() {
        assert_eq!(
            split_tokens("0  calc(1px + 2px) auto"),
            ["0", "calc(1px + 2px)", "auto"]
        );
        assert_eq!(split_tokens("  1px "), ["1px"]);
        assert!(split_tokens("").is_empty());
    }
}
