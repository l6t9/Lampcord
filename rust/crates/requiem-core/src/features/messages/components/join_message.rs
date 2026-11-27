use crate::ui::author_style;
use crate::ui::classes::pins as cls;
use crate::ui::profile_popout;
use std::rc::Rc;

use requiem_ui::components::{MessageTextProps, TextProps, TextVariant, div, icon, message_text};
use requiem_ui::node::Node;
use requiem_ui::widget::AbsoluteRect;
use skia_safe::Color;

use crate::models::Message;
use crate::ui::TEXT_STRONG;
use crate::ui::author_style::NameFill;
use requiem_ui::icons::ARROW_SMALL_RIGHT;

const JOIN_GREEN: Color = Color::from_rgb(0x3d, 0x9e, 0x60);

const WELCOMES: i64 = 13;

pub fn system_icon() -> Node {
    div()
        .class(cls::SYSTEM_ICON_SLOT)
        .child(icon(ARROW_SMALL_RIGHT, JOIN_GREEN).class(cls::SYSTEM_ICON))
}

fn welcome(timestamp_ms: i64, user: &str) -> String {
    let index = timestamp_ms.rem_euclid(WELCOMES) + 1;
    requiem_i18n::t!(&format!("join-message-{index}"), user = user).to_string()
}

pub fn system_body(
    message: &Message,
    name: &str,
    fill: Option<&NameFill>,
    muted: Color,
    profile_popout: &profile_popout::ProfilePopoutState,
) -> Node {
    const USER: &str = "https://join.invalid/user";
    let escaped = name.replace(['[', ']', '*', '_', '`', '~', '|', '\\'], "");
    let content = welcome(
        message.timestamp_millis().unwrap_or_default(),
        &format!("[{escaped}]({USER})"),
    );
    let (row_id, author) = (message.id, message.author.id);
    let popout = profile_popout.clone();
    let on_link: Rc<dyn Fn(&str, AbsoluteRect)> = Rc::new(move |url: &str, link: AbsoluteRect| {
        if url == USER {
            profile_popout::toggle_for(row_id, Some(author), link, &popout);
        }
    });
    let name_color = author_style::inline_name_color(fill, TEXT_STRONG);
    let text = message_text(MessageTextProps {
        content,
        size: 16.0,
        color: muted,
        line_height: Some(22.0),
        link_style: Some((name_color, 500)),
        on_link: Some(on_link),
        ..Default::default()
    });
    let mut body = div().class(cls::SYSTEM_BODY).child(text);
    if let Some(time) = message.format_header_timestamp() {
        body = body.child(
            TextProps::styled(time, TextVariant::TEXT_XS_MEDIUM)
                .line_height(22.0_f32)
                .color(muted)
                .build()
                .class(cls::SYSTEM_TIME),
        );
    }
    body
}
