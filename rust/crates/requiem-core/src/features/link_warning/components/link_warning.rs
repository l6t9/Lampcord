use crate::ui::classes::link_warning as cls;
use std::rc::Rc;

use requiem_ui::components::{CheckboxProps, TextProps, TextVariant, div};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::ToggleState;
use skia_safe::font_style::Weight;

use crate::ui::context_menu::{ConfirmRequest, confirm_labeled};
use crate::ui::{TEXT_DEFAULT, TEXT_SECONDARY, TEXT_SUBTLE};
use crate::utils::link::open_url;
use crate::utils::prefs;

/// Hosts that never need the warning.
const ALWAYS_TRUSTED: [&str; 3] = ["discord.com", "discordapp.com", "discord.gg"];

thread_local! {
    static TRUST: Signal<ToggleState> = signal(ToggleState::new(false));
}

/// Opens a link, first asking when it is a masked link (its label hides where it goes)
/// to a host the user hasn't trusted.
pub fn open_link(url: &str, masked: bool) {
    let Some(host) = host_of(url) else {
        return open_url(url);
    };
    if !masked || is_trusted(&host) {
        return open_url(url);
    }
    TRUST.with(|trust| trust.set(ToggleState::new(false)));
    let (target, trusting) = (url.to_string(), host.clone());
    let (shown, shown_host) = (url.to_string(), host);
    confirm_labeled(
        ConfirmRequest {
            title: requiem_i18n::t!("link-warning-title").to_string(),
            body: requiem_i18n::t!("link-warning-body").to_string(),
            confirm_label: requiem_i18n::t!("link-warning-visit").to_string(),
            on_confirm: Rc::new(move || {
                if TRUST.with(|trust| trust.peek().on) {
                    trust_host(&trusting);
                }
                open_url(&target);
            }),
            critical: false,
            content: Some(Rc::new(move || content(&shown, &shown_host))),
        },
        requiem_i18n::t!("link-warning-go-back").to_string(),
    );
}

/// The host of an `http(s)` address, lowercase, without credentials or port.
fn host_of(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit('@').next()?;
    let host = host.split(':').next()?.to_ascii_lowercase();
    (!host.is_empty()).then_some(host)
}

fn is_trusted(host: &str) -> bool {
    let under = |trusted: &str| host == trusted || host.ends_with(&format!(".{trusted}"));
    ALWAYS_TRUSTED.iter().any(|trusted| under(trusted))
        || prefs::load()
            .trusted_link_domains
            .unwrap_or_default()
            .iter()
            .any(|trusted| under(trusted))
}

fn trust_host(host: &str) {
    let mut saved = prefs::load();
    let trusted = saved.trusted_link_domains.get_or_insert_with(Vec::new);
    if !trusted.iter().any(|known| known == host) {
        trusted.push(host.to_string());
        prefs::save(&saved);
    }
}

fn piece(text: String, weight: Weight, color: skia_safe::Color) -> Node {
    let mut props = TextProps::styled(text, TextVariant::TEXT_MD_NORMAL)
        .line_height(20.0_f32)
        .color(color);
    props.weight = weight.into();
    props.build()
}

/// The address with its host picked out, then the trust option.
fn content(url: &str, host: &str) -> Node {
    let scheme_end = url.find("://").map_or(0, |at| at + 3);
    let host_end = (scheme_end + url[scheme_end..].find(host).unwrap_or(0) + host.len()).min(url.len());
    let (scheme, rest) = url.split_at(scheme_end);
    let (before, after) = (
        &rest[..host_end.saturating_sub(scheme_end).min(rest.len())],
        &url[host_end..],
    );
    let host_text = before.rsplit('@').next().unwrap_or(before).to_string();
    let credentials = before[..before.len() - host_text.len()].to_string();
    let address = div().class(cls::URL).children([
        piece(format!("{scheme}{credentials}"), Weight::NORMAL, TEXT_SUBTLE),
        piece(host_text, Weight::SEMI_BOLD, TEXT_DEFAULT),
        piece(after.to_string(), Weight::NORMAL, TEXT_SUBTLE),
    ]);
    let host = host.to_string();
    let trust = Node::reactive(move || {
        let label = div().class(cls::TRUST).children([
            piece(
                requiem_i18n::t!("link-warning-trust-before").to_string(),
                Weight::NORMAL,
                TEXT_SECONDARY,
            ),
            piece(host.clone(), Weight::SEMI_BOLD, TEXT_SECONDARY),
            piece(
                requiem_i18n::t!("link-warning-trust-after").to_string(),
                Weight::NORMAL,
                TEXT_SECONDARY,
            ),
        ]);
        CheckboxProps::new(TRUST.with(Signal::get), |on: bool| {
            TRUST.with(|trust| trust.set(ToggleState::new(on)))
        })
        .content(label)
        .build()
    });
    div().class(cls::CONTENT).children([address, trust])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_host_is_read_without_credentials_port_or_case() {
        assert_eq!(host_of("https://Example.COM/a?b#c").as_deref(), Some("example.com"));
        assert_eq!(host_of("http://user:pw@example.com:8080/x").as_deref(), Some("example.com"));
        assert_eq!(host_of("https://example.com?x=1").as_deref(), Some("example.com"));
    }

    #[test]
    fn only_web_addresses_are_warned_about() {
        assert_eq!(host_of("mailto:someone@example.com"), None);
        assert_eq!(host_of("ms-settings:privacy"), None);
        assert_eq!(host_of("https://"), None);
    }

    #[test]
    fn discord_and_its_subdomains_are_always_trusted() {
        assert!(is_trusted("discord.com"));
        assert!(is_trusted("cdn.discordapp.com"));
        assert!(!is_trusted("notdiscord.com"));
        assert!(!is_trusted("discord.com.example.org"));
    }
}
