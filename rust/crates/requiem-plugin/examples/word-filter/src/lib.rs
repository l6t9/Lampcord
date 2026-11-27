use requiem_plugin::prelude::*;
use serde::{Deserialize, Serialize};

requiem_plugin::classes!();
use classes::settings as cls;

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    words: String,
    block_outgoing: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            words: String::new(),
            block_outgoing: true,
        }
    }
}

impl Settings {
    fn blocked_words(&self) -> Vec<String> {
        self.words
            .split(',')
            .map(|w| w.trim().to_lowercase())
            .filter(|w| !w.is_empty())
            .collect()
    }
}

fn mask(text: &str, words: &[String]) -> Option<String> {
    let mut changed = false;
    let masked = text
        .split(' ')
        .map(|token| {
            let bare = token
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if words.contains(&bare) {
                changed = true;
                "*".repeat(token.chars().count())
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    changed.then_some(masked)
}

fn masked_count() -> u64 {
    storage::get_string("masked-count")
        .and_then(|c| c.parse().ok())
        .unwrap_or(0)
}

struct WordFilter;

impl Plugin for WordFilter {
    fn init(previous_version: Option<String>) -> Result<(), Error> {
        let words = settings::load::<Settings>().blocked_words().len();
        log::info(&format!(
            "word filter started (previous version: {previous_version:?}, {words} word(s))"
        ));
        Ok(())
    }

    fn before_send(message: &OutgoingMessage) -> Verdict {
        let settings = settings::load::<Settings>();
        if !settings.block_outgoing {
            return Verdict::Pass;
        }
        match mask(&message.content(), &settings.blocked_words()) {
            Some(_) => Verdict::Cancel("Your message contains a blocked word".to_string()),
            None => Verdict::Pass,
        }
    }

    fn before_edit(edit: &OutgoingEdit) -> Verdict {
        if let Some(masked) = mask(
            &edit.content(),
            &settings::load::<Settings>().blocked_words(),
        ) {
            edit.set_content(&masked);
        }
        Verdict::Pass
    }

    fn transform(batch: &[IncomingMessage]) -> Vec<Verdict> {
        let words = settings::load::<Settings>().blocked_words();
        let mut masked = 0;
        let verdicts = batch
            .iter()
            .map(|incoming| {
                let content = incoming.message().content();
                if words.contains(&content.trim().to_lowercase()) {
                    return Verdict::Cancel("only a blocked word".to_string());
                }
                if let Some(text) = mask(&content, &words) {
                    incoming.set_display_content(&text);
                    incoming.set_annotation("Blocked words hidden");
                    masked += 1;
                }
                Verdict::Pass
            })
            .collect();
        if masked > 0 {
            let _ = storage::set(
                "masked-count",
                (masked_count() + masked).to_string().as_bytes(),
            );
        }
        verdicts
    }

    fn settings_page() -> Node {
        let current = settings::load::<Settings>();
        div()
            .class(cls::PAGE)
            .child(form::text_row(
                "Blocked words",
                Some("Separate words with commas."),
                &current.words,
                "spoiler, ...",
                |words| settings::update::<Settings>(|s| s.words = words),
            ))
            .child(form::switch_row(
                "Block sending",
                Some("Keep messages containing a blocked word from being sent."),
                current.block_outgoing,
                |on| settings::update::<Settings>(|s| s.block_outgoing = on),
            ))
            .child(
                div().class(cls::STATS).child(
                    StackProps::horizontal()
                        .justify(Justify::SpaceBetween)
                        .align(Align::Center)
                        .build()
                        .child(
                            TextProps::styled(
                                format!("{} message(s) masked so far", masked_count()),
                                TextVariant::TEXT_SM_NORMAL,
                            )
                            .color("var(--text-muted)")
                            .build(),
                        )
                        .child(
                            ButtonProps::new(ButtonVariant::Secondary, "Reset", || {
                                let _ = storage::set("masked-count", b"0");
                            })
                            .size(ButtonSize::Small)
                            .build(),
                        ),
                ),
            )
    }
}

export_plugin!(WordFilter);
