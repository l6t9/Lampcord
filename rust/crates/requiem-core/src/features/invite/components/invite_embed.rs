use crate::models::marker;
use crate::models;
use crate::ui::channel_sidebar;
use crate::ui::classes::invite_embed as cls;
use crate::ui;
use crate::utils::time;
use requiem_ui::colors;
use std::rc;
use std::time::Duration;

use requiem_ui::components::{
    AvatarProps, ButtonProps, ButtonSize, ButtonVariant, TextProps, TextVariant, div,
};
use requiem_ui::node::Node;
use requiem_ui::widget::primitives::{AvatarShape, ImageWidget};

use crate::rest::endpoints::invites::{self, InvitePreview};
use crate::ui::context_menus::MenuEnv;

pub fn cards(
    env: &MenuEnv,
    fonts: &ui::Fonts,
    content: &str,
    open_guild: &rc::Rc<dyn Fn(models::Id<marker::GuildMarker>)>,
) -> Vec<Node> {
    let codes = invites::codes_in(content);
    codes
        .into_iter()
        .filter_map(|code| card(env, fonts, &code, open_guild))
        .collect()
}

fn card(
    env: &MenuEnv,
    fonts: &ui::Fonts,
    code: &str,
    open_guild: &rc::Rc<dyn Fn(models::Id<marker::GuildMarker>)>,
) -> Option<Node> {
    let http = env.http.clone()?;
    let fetch_code = code.to_string();
    let result = env.queries.query::<InvitePreview, _, String>(
        &format!("invite:{code}"),
        Duration::from_secs(300),
        move || async move { invites::fetch_preview(&http, &fetch_code).await },
    );
    let preview = result.data?;

    let mut banner = div().class(cls::INVITE_BANNER);
    if let Some(image) = preview.banner.as_deref().and_then(|hash| {
        env.images
            .get_or_decode_image(&env.backend.guild_banner_url(preview.guild_id, hash))
    }) {
        banner = Node::new(
            "div",
            ImageWidget::new(Some(image))
                .with_cover()
                .with_cover_centered(),
        )
        .class(cls::INVITE_BANNER);
    }

    let acronym: String = preview
        .name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect();
    let mut icon = AvatarProps::default()
        .fallback_initial(acronym)
        .fallback_background(colors::SURFACE_HIGH)
        .shape(AvatarShape::RoundedSquare(0.25));
    if let Some(image) = preview.icon.as_deref().and_then(|hash| {
        env.images
            .get_or_decode_image(&env.backend.guild_icon_url(preview.guild_id, hash))
    }) {
        icon = icon.image(image);
    }
    let avatar_row = div().class(cls::INVITE_AVATAR_ROW).child(
        div().class(cls::INVITE_AVATAR_RING).child(
            icon.typeface(fonts.typeface.clone())
                .build()
                .class(cls::INVITE_AVATAR),
        ),
    );

    let mut name_row = div().class(cls::INVITE_NAME_ROW).child(
        TextProps::styled(preview.name.clone(), TextVariant::TEXT_MD_SEMIBOLD)
            .color(ui::TEXT_STRONG)
            .line_clamp(1_u32)
            .build(),
    );
    if preview.partnered {
        name_row = name_row.child(channel_sidebar::partnered_badge());
    } else if preview.verified {
        name_row = name_row.child(channel_sidebar::verified_badge());
    }
    let stat = |dot: &'static str, text: String| {
        div()
            .class(cls::INVITE_STAT)
            .child(div().class(cls::INVITE_DOT).class(dot))
            .child(
                TextProps::styled(text, TextVariant::TEXT_SM_NORMAL)
                    .color(ui::TEXT_SUBTLE)
                    .build(),
            )
    };
    let mut stats = div().class(cls::INVITE_STATS);
    if let Some(online) = preview.online {
        stats = stats.child(stat(
            cls::INVITE_DOT_ONLINE,
            requiem_i18n::t!("invite-embed-online", count = online).to_string(),
        ));
    }
    if let Some(members) = preview.members {
        stats = stats.child(stat(
            cls::INVITE_DOT_TOTAL,
            requiem_i18n::t!("invite-embed-members", count = members).to_string(),
        ));
    }
    let established = TextProps::styled(
        requiem_i18n::t!(
            "invite-embed-established",
            date = time::snowflake_month_year(preview.guild_id.get())
        )
        .to_string(),
        TextVariant::TEXT_SM_NORMAL,
    )
    .color(ui::TEXT_SUBTLE)
    .build();
    let header = div()
        .class(cls::INVITE_HEADER)
        .children([name_row, stats, established]);

    let is_member = env
        .state
        .guilds
        .list
        .with_untracked(|guilds| guilds.iter().any(|guild| guild.id == preview.guild_id));
    let button = if is_member {
        let (open_guild, guild_id) = (open_guild.clone(), preview.guild_id);
        ButtonProps::default()
            .variant(ButtonVariant::Active)
            .size(ButtonSize::Small)
            .full_width(true)
            .label(requiem_i18n::t!("invite-embed-go-to-server").to_string())
            .on_click(move || open_guild(guild_id))
            .build()
    } else {
        let (http, queries, code, resolved) = (
            env.http.clone(),
            env.queries.clone(),
            code.to_string(),
            preview.resolved.clone(),
        );
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Small)
            .full_width(true)
            .label(requiem_i18n::t!("invite-embed-join").to_string())
            .on_click(move || {
                let Some(http) = http.clone() else { return };
                let (code, resolved) = (code.clone(), resolved.clone());
                queries.mutate(Vec::new(), move || async move {
                    invites::accept_invite(&http, &code, &resolved)
                        .await
                        .map(|_| ())
                });
            })
            .build()
    };
    Some(div().class(cls::INVITE_CARD).children([
        banner,
        avatar_row,
        header,
        div().class(cls::INVITE_BUTTON_ROW).child(button),
    ]))
}
