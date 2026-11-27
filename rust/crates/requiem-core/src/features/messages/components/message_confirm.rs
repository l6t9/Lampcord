use crate::ui::classes::message_confirm as cls;
use crate::ui::emoji_text;
use crate::ui::message_list;
use crate::ui::twemoji;
use crate::ui;
use requiem_ui::colors;
use requiem_ui::components;
use requiem_ui::widget::primitives::MarkdownFeatures;
use std::cell::RefCell;
use std::rc::Rc;

use requiem_ui::components::{MessageTextProps, TextProps, TextVariant, div};
use requiem_ui::node::Node;

use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::ui::home::HomeEnv;
use crate::ui::{TEXT_DEFAULT, TEXT_STRONG};

thread_local! {
    static ENV: RefCell<Option<Rc<dyn Fn() -> HomeEnv>>> = const { RefCell::new(None) };
}

pub fn install(env: Rc<dyn Fn() -> HomeEnv>) {
    ENV.with(|e| *e.borrow_mut() = Some(env));
}

pub fn env() -> Option<HomeEnv> {
    ENV.with(|e| e.borrow().clone()).map(|factory| factory())
}

pub fn preview(
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    tip: Option<String>,
) -> Node {
    let Some(env) = env() else { return div() };
    let message = env
        .state
        .messages
        .in_channel(channel_id)
        .iter()
        .find(|m| m.id == message_id)
        .cloned()
        .or_else(|| {
            env.state.pins.by_channel.with_untracked(|all| {
                all.get(&channel_id).and_then(|p| {
                    p.items
                        .iter()
                        .find(|pin| pin.message.id == message_id)
                        .map(|pin| pin.message.clone())
                })
            })
        });
    let Some(message) = message else { return div() };
    let author = env.state.users.resolve(&message.author);
    let name = match ui::find_channel(&env.state, channel_id) {
        Some((_, channel)) => {
            message_list::author_display_name(&env.state, &channel, &author)
        }
        None => env.state.users.display_name(&author),
    };
    let image = author.avatar.as_deref().and_then(|hash| {
        env.images
            .get_or_decode_image(&env.backend.avatar_url(author.id, hash))
    });
    let text = MessageTextProps {
        content: message.content.clone(),
        size: 16.0,
        color: TEXT_DEFAULT,
        emoji_provider: Some(twemoji::tile),
        custom_emoji: Some(emoji_text::resolver(&env.images, &env.backend)),
        line_height: Some(22.0),
        markdown: Some(MarkdownFeatures::CHAT),
        ..Default::default()
    };
    let mut header = div().class(cls::HEADER).child(
        TextProps::styled(name, TextVariant::TEXT_MD_MEDIUM)
            .line_height(22.0_f32)
            .color(TEXT_STRONG)
            .line_clamp(1_u32)
            .build(),
    );
    if let Some(time) = message.format_header_timestamp() {
        header = header.child(
            TextProps::styled(time, TextVariant::TEXT_XS_MEDIUM)
                .line_height(22.0_f32)
                .color(colors::TEXT_SUBTLE)
                .build(),
        );
    }
    let mut avatar = ui::user_avatar_props(author.id, &author.discriminator, &env.fonts);
    if let Some(image) = image {
        avatar = avatar.image(image);
    }
    let row = div()
        .class(cls::ROW)
        .child(avatar.build().class(cls::AVATAR))
        .child(
            div()
                .class(cls::BODY)
                .child(header)
                .child(components::message_text(text)),
        );
    let mut out = div()
        .class(cls::WRAP)
        .child(div().class(cls::MESSAGE).child(row));
    if let Some(tip) = tip {
        out = out.child(
            div()
                .class(cls::TIP)
                .child(
                    TextProps::styled(
                        format!("{}:", requiem_i18n::t!("inbox-protip")),
                        TextVariant::TEXT_SM_BOLD,
                    )
                    .color(colors::TEXT_POSITIVE)
                    .uppercase(true)
                    .build(),
                )
                .child(components::message_text(MessageTextProps {
                    content: tip,
                    size: 14.0,
                    color: TEXT_DEFAULT,
                    line_height: Some(16.0),
                    ..Default::default()
                })),
        );
    }
    out
}
