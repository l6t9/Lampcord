use requiem_core::ui;
use serde_json::json;

use super::Request;

const TEMPLATE: &str = include_str!("page.html");

fn theme_tokens() -> String {
    let theme = requiem_ui::theme_name();
    let class = format!(".theme-{theme}");
    let css = ui::THEME_CSS;
    let Some(start) = css.find(&class) else {
        return include_str!("../../../../../requiem-ui/src/tokens.css").to_string();
    };
    let body = &css[start + class.len()..];
    let Some(open) = body.find('{') else {
        return include_str!("../../../../../requiem-ui/src/tokens.css").to_string();
    };
    let mut depth = 0usize;
    let mut end = None;
    for (i, ch) in body[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(end) = end else {
        return include_str!("../../../../../requiem-ui/src/tokens.css").to_string();
    };
    format!(".mana-tokens {{\n{}\n}}", &body[open + 1..end])
}

pub fn html(request: &Request, standalone: bool) -> String {
    let host = standalone.then(|| {
        request
            .origin
            .split_once("://")
            .map_or(request.origin.as_str(), |(_, host)| host)
            .to_string()
    });
    let theme = if requiem_ui::theme_name() == "light" {
        "light"
    } else {
        "dark"
    };
    let config = json!({
        "service": request.challenge.service,
        "sitekey": request.challenge.sitekey,
        "rqdata": request.challenge.rqdata,
        "invisible": request.challenge.invisible,
        "language": request.language,
        "host": host,
        "graphic": request.graphic.is_some(),
        "theme": theme,
    });
    let fonts: String = request
        .fonts
        .iter()
        .map(|font| format!("@font-face {{ font-family: \"gg sans\"; font-weight: {}; src: url(data:font/woff2;base64,{}) format(\"woff2\"); }}\n", font.weight, font.woff2))
        .collect();
    let graphic = request.graphic.as_ref().map_or_else(String::new, |svg| {
        format!("data:image/svg+xml;base64,{svg}")
    });
    TEMPLATE
        .replace("@LANG@", &escape(&request.language))
        .replace("@TITLE@", &escape(&requiem_i18n::t!("captcha-title")))
        .replace("@SUBTITLE@", &escape(&requiem_i18n::t!("captcha-subtitle")))
        .replace("@ERROR@", &escape(&requiem_i18n::t!("captcha-error")))
        .replace("@RETRY@", &escape(&requiem_i18n::t!("captcha-retry")))
        .replace("@GRAPHIC@", &graphic)
        .replace("@TOKENS@", &theme_tokens())
        .replace("@SCHEME@", theme)
        .replace("@FONTS@", &fonts)
        .replace("@CONFIG@", &config.to_string().replace("</", "<\\/"))
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use requiem_core::auth::{CaptchaChallenge, CaptchaService};

    use super::*;

    fn request(rqdata: &str) -> Request {
        Request {
            challenge: CaptchaChallenge {
                service: CaptchaService::Hcaptcha,
                sitekey: Some("site".into()),
                rqdata: Some(rqdata.into()),
                rqtoken: None,
                session_id: None,
                invisible: false,
                reasons: vec![],
            },
            origin: "https://discord.com".into(),
            language: "en".into(),
            graphic: None,
            fonts: vec![],
        }
    }

    #[test]
    fn nothing_is_left_unfilled_and_server_text_cannot_break_out_of_the_script() {
        let page = html(
            &request("</script><script>alert(1)</script> @TOKENS@"),
            false,
        );
        for placeholder in [
            "@LANG@",
            "@TITLE@",
            "@SUBTITLE@",
            "@ERROR@",
            "@RETRY@",
            "@GRAPHIC@",
            "@SCHEME@",
            "@FONTS@",
            "@CONFIG@",
        ] {
            assert!(
                !page.contains(placeholder),
                "{placeholder} was not replaced"
            );
        }
        assert!(
            !page.contains("</script><script>alert"),
            "the challenge's text ended the script"
        );
    }

    #[test]
    fn the_standalone_page_names_the_host_for_hcaptcha() {
        assert!(html(&request("x"), true).contains(r#""host":"discord.com""#));
        assert!(html(&request("x"), false).contains(r#""host":null"#));
    }
}
