use crate::models;
use crate::ui::channel_sidebar;
use crate::ui::classes::emoji_popout as cls;
use crate::ui::emoji_names;
use crate::ui::emoji_text;
use crate::ui::twemoji;
use crate::ui;
use crate::utils::guild;
use requiem_ui::components::Anchor;
use requiem_ui::components::PopoverAlign;
use requiem_ui::components::PopoverProps;
use requiem_ui::widget::primitives::ImageWidget;
use std::rc::Rc;

use requiem_ui::components::{
    ButtonProps, ButtonSize, ButtonVariant, MessageTextProps, TextProps, TextVariant, div,
    message_text, popover_backdrop,
};
use requiem_ui::icons;
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::AbsoluteRect;
use requiem_ui::widget::primitives::{EmojiClick, EmojiKind};

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::state::AppState;
use crate::ui::{TEXT_DEFAULT, TEXT_STRONG, TEXT_SUBTLE};

const GAP: f32 = 8.0;

#[derive(Clone)]
struct Open {
    kind: EmojiKind,
    anchor: AbsoluteRect,

    guild_id: Option<Id<GuildMarker>>,
}

thread_local! {
    static OPEN: Signal<Option<Open>> = signal(None);
}

pub fn open(click: EmojiClick, guild_id: Option<Id<GuildMarker>>) {
    OPEN.with(|open| {
        open.set(Some(Open {
            kind: click.kind,
            anchor: click.rect,
            guild_id,
        }))
    });
}

pub fn close() {
    OPEN.with(|open| open.set(None));
}

#[derive(Debug, PartialEq, Eq)]
struct Info {
    description: &'static str,

    get_nitro: bool,
}

fn custom_info(joined: bool, current: bool, nitro: bool) -> Info {
    match (nitro, joined, current) {
        (true, true, true) => Info {
            description: "emoji-popout-this-server",
            get_nitro: false,
        },
        (true, true, false) => Info {
            description: "emoji-popout-your-servers",
            get_nitro: false,
        },
        (false, true, true) => Info {
            description: "emoji-popout-this-server-nitro",
            get_nitro: true,
        },
        (false, true, false) => Info {
            description: "emoji-popout-your-servers-nitro",
            get_nitro: true,
        },

        (_, false, _) => Info {
            description: "emoji-popout-unavailable",
            get_nitro: false,
        },
    }
}

pub fn build(
    state: &AppState,
    images: &ImageCache,
    backend: &BackendConfig,
    tick: &Signal<u64>,
    on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>,
) -> Node {
    let layer = div().class(cls::LAYER);
    let Some(open) = OPEN.with(Signal::get) else {
        return layer;
    };
    let nitro = state.users.current.with_untracked(|user| {
        user.as_ref()
            .and_then(|u| u.premium_type)
            .is_some_and(|t| t > 0)
    });

    let (content, name, info, source) = match &open.kind {
        EmojiKind::Unicode(emoji) => {
            let name = emoji_names::primary_name(emoji)
                .map(|n| format!(":{n}:"))
                .unwrap_or_else(|| emoji.clone());
            (
                emoji.clone(),
                name,
                Info {
                    description: "emoji-popout-default",
                    get_nitro: false,
                },
                None,
            )
        }
        EmojiKind::Custom { id, name, animated } => {
            let found = state.emojis.find(Id::new(*id));
            let joined = found.is_some();
            let current = found
                .as_ref()
                .is_some_and(|(guild, _)| Some(*guild) == open.guild_id);
            let guild = found.and_then(|(guild, _)| state.guilds.find(guild));
            (
                format!("<{}:{name}:{id}>", if *animated { "a" } else { "" }),
                format!(":{name}:"),
                custom_info(joined, current, nitro),
                guild,
            )
        }
    };

    let emoji = message_text(MessageTextProps {
        content,
        size: 16.0,
        color: TEXT_DEFAULT,
        emoji_provider: Some(twemoji::tile),
        custom_emoji: Some(emoji_text::resolver(images, backend)),
        emoji_scale: Some(emoji_text::JUMBO_SCALE),
        line_height: Some(48.0),
        emoji_in_box: true,
        ..Default::default()
    })
    .bind_image(tick.clone(), || None)
    .class(cls::EMOJI);
    let description = requiem_i18n::t!(info.description);
    let text = div()
        .class(cls::TEXT)
        .child(
            TextProps::styled(name, TextVariant::TEXT_MD_SEMIBOLD)
                .line_height(20.0_f32)
                .color(TEXT_DEFAULT)
                .line_clamp(1_u32)
                .build(),
        )
        .child(
            TextProps::styled(description.clone(), TextVariant::TEXT_SM_NORMAL)
                .line_height(18.0_f32)
                .color(TEXT_DEFAULT)
                .build(),
        );
    let mut section = div()
        .class(cls::SECTION)
        .child(div().class(cls::ROW).child(emoji).child(text));
    if info.get_nitro {
        section = section.child(
            div().class(cls::CTA).child(
                ButtonProps::default()
                    .variant(ButtonVariant::Expressive)
                    .size(ButtonSize::Small)
                    .icon(icons::NITRO_WHEEL)
                    .label(requiem_i18n::t!("emoji-popout-get-nitro"))
                    .on_click(close)
                    .build()
                    .class(cls::CTA_BUTTON),
            ),
        );
    }

    let anchor = Anchor::new(open.anchor)
        .align(PopoverAlign::Center)
        .spacing(GAP);
    let mut panel = PopoverProps::default()
        .anchor(anchor)
        .build()
        .class(cls::POPOUT)
        .child(section);
    if let Some(guild) = source {
        let (state, on_select) = (state.clone(), on_select_channel.clone());
        let guild_id = guild.id;
        let on_open: Rc<dyn Fn()> = Rc::new(move || {
            close();
            if let Some(channel) = ui::default_channel(&state, Some(guild_id)) {
                on_select(channel.id);
            }
        });
        panel = panel.child(guild_section(images, backend, &guild, on_open));
    }

    layer
        .class(cls::LAYER_OPEN)
        .child(popover_backdrop(close))
        .child(panel)
}

fn guild_section(
    images: &ImageCache,
    backend: &BackendConfig,
    guild: &models::Guild,
    on_open: Rc<dyn Fn()>,
) -> Node {
    let icon = match guild
        .icon
        .as_deref()
        .and_then(|hash| images.get_or_decode_image(&backend.guild_icon_url(guild.id, hash)))
    {
        Some(image) => Node::new(
            "div",
            ImageWidget::new(Some(image)),
        )
        .class(cls::GUILD_ICON),
        None => div()
            .class(cls::GUILD_ICON)
            .class(cls::GUILD_ICON_TILE)
            .child(
                TextProps::styled(
                    guild::acronym(&guild.name),
                    TextVariant::TEXT_XS_MEDIUM,
                )
                .color(TEXT_DEFAULT)
                .center(true)
                .line_clamp(1_u32)
                .build(),
            ),
    };
    let open = on_open.clone();
    let icon = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::GUILD_ICON_BUTTON.to_string()])
        .on_click(move || open())
        .build()
        .child(icon);
    let mut name = div().class(cls::GUILD_NAME);
    if guild.features.iter().any(|f| f == "VERIFIED") {
        name = name.child(
            div()
                .class(cls::GUILD_BADGE)
                .child(channel_sidebar::verified_badge()),
        );
    }
    let title = TextProps::styled(guild.name.clone(), TextVariant::HEADING_MD_SEMIBOLD)
        .line_height(20.0_f32)
        .color(TEXT_STRONG)
        .line_clamp(1_u32)
        .build();
    name = name.child(
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::GUILD_NAME_BUTTON.to_string()])
            .on_click(move || on_open())
            .build()
            .child(title),
    );
    let mut text = div().class(cls::GUILD_TEXT).child(name);
    if !guild.features.iter().any(|f| f == "DISCOVERABLE") {
        text = text.child(
            TextProps::styled(
                requiem_i18n::t!("emoji-popout-invite-only"),
                TextVariant::TEXT_XS_NORMAL,
            )
            .line_height(16.0_f32)
            .color(TEXT_DEFAULT)
            .build(),
        );
    }
    div()
        .class(cls::GUILD_SECTION)
        .child(
            TextProps::styled(
                requiem_i18n::t!("emoji-popout-from"),
                TextVariant::TEXT_SM_MEDIUM,
            )
            .line_height(18.0_f32)
            .color(TEXT_SUBTLE)
            .build()
            .class(cls::GUILD_TITLE),
        )
        .child(div().class(cls::GUILD_ROW).child(icon).child(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_and_button_follow_where_the_emoji_is_from_and_whether_you_have_nitro() {
        assert_eq!(
            custom_info(true, true, true),
            Info {
                description: "emoji-popout-this-server",
                get_nitro: false
            }
        );
        assert_eq!(
            custom_info(true, false, true),
            Info {
                description: "emoji-popout-your-servers",
                get_nitro: false
            }
        );

        assert_eq!(
            custom_info(true, true, false),
            Info {
                description: "emoji-popout-this-server-nitro",
                get_nitro: true
            }
        );
        assert_eq!(
            custom_info(true, false, false),
            Info {
                description: "emoji-popout-your-servers-nitro",
                get_nitro: true
            }
        );

        assert_eq!(
            custom_info(false, false, false).description,
            "emoji-popout-unavailable"
        );
        assert!(!custom_info(false, false, true).get_nitro);
    }

    #[test]
    fn default_emoji_have_their_discord_names() {
        for (emoji, name) in [
            ("\u{1F604}", "smile"),
            ("\u{1FAE3}", "face_with_peeking_eye"),
            ("\u{1F633}", "flushed"),
        ] {
            assert_eq!(
                emoji_names::primary_name(emoji),
                Some(name),
                "{emoji}"
            );
        }
    }
}
