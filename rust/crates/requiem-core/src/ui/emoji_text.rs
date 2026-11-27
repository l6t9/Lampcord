use std::rc::Rc;

use requiem_ui::widget::primitives::CustomEmojiResolver;

use super::twemoji;
use crate::config::BackendConfig;
use crate::images::ImageCache;

const MAX_JUMBO_EMOJI: usize = 30;

pub const JUMBO_SCALE: f32 = 3.0;

pub fn resolver(images: &ImageCache, backend: &BackendConfig) -> CustomEmojiResolver {
    let (images, backend) = (images.clone(), backend.clone());
    Rc::new(move |id, animated| super::emoji_anim::frame(&images, &backend, id, animated))
}

pub fn is_jumbo(content: &str) -> bool {
    let mut count = 0;
    let mut rest = content.trim();
    if rest.is_empty() {
        return false;
    }
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if let Some(len) = custom_token_len(rest) {
            rest = &rest[len..];
        } else {
            let grapheme = unicode_segmentation::UnicodeSegmentation::graphemes(rest, true)
                .next()
                .unwrap_or("");
            if grapheme.is_empty() || twemoji::tile(grapheme).is_none() {
                return false;
            }
            rest = &rest[grapheme.len()..];
        }
        count += 1;
        if count > MAX_JUMBO_EMOJI {
            return false;
        }
    }
    count > 0
}

pub fn flatten_custom_emoji(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        match custom_token_len(rest) {
            Some(len) => {
                let token = &rest[..len];
                let name = token
                    .trim_start_matches("<a")
                    .trim_start_matches('<')
                    .trim_start_matches(':');
                out.push(':');
                out.push_str(name.split(':').next().unwrap_or(""));
                out.push(':');
                rest = &rest[len..];
            }
            None => {
                out.push('<');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn custom_token_len(text: &str) -> Option<usize> {
    let body = text.strip_prefix('<')?;
    let body = body.strip_prefix('a').unwrap_or(body);
    let body = body.strip_prefix(':')?;
    let name_len = body
        .bytes()
        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
        .count();
    if !(2..=32).contains(&name_len) {
        return None;
    }
    let body = body[name_len..].strip_prefix(':')?;
    let id_len = body.bytes().take_while(u8::is_ascii_digit).count();
    if id_len == 0 || !body[id_len..].starts_with('>') {
        return None;
    }
    Some(text.len() - body.len() + id_len + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emoji_only_messages_are_jumbo() {
        assert!(is_jumbo("<:happy:123456>"));
        assert!(is_jumbo("<:happy:123456> <a:wiggle:789>  <:happy:123456>"));
        assert!(is_jumbo("<:happy:123456><:happy:123456>"));
    }

    #[test]
    fn text_or_nothing_is_not() {
        assert!(!is_jumbo(""));
        assert!(!is_jumbo("hi <:happy:123456>"));
        assert!(!is_jumbo("<:happy:123456> hi"));
        assert!(!is_jumbo("<:h:123456>"), "names are at least 2 characters");
        assert!(!is_jumbo("<:happy:12x>"));
    }

    #[test]
    fn too_many_stay_small() {
        let many = "<:happy:1>".repeat(MAX_JUMBO_EMOJI + 1);
        assert!(!is_jumbo(&many));
        assert!(is_jumbo(&"<:happy:1>".repeat(MAX_JUMBO_EMOJI)));
    }

    #[test]
    fn previews_show_custom_emoji_by_name() {
        assert_eq!(
            flatten_custom_emoji("hi <:happy:123> and <a:wiggle:9> <@5> <"),
            "hi :happy: and :wiggle: <@5> <"
        );
        assert_eq!(flatten_custom_emoji("plain"), "plain");
    }

    #[test]
    fn token_length_covers_the_whole_token() {
        assert_eq!(custom_token_len("<:ab:12>rest"), Some(8));
        assert_eq!(custom_token_len("<a:ab:12>"), Some(9));
        assert_eq!(custom_token_len("<@12>"), None);
    }
}
