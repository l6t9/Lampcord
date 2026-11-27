use crate::ui::classes::dm_start as cls;
use crate::ui::profile_popout::ProfileSurface;
use requiem_ui::components::{
    AvatarStatusSize, ButtonProps, ButtonSize, ButtonVariant, MessageTextProps, TextProps,
    TextVariant, avatar_with_status, div, message_text,
};
use requiem_ui::node::Node;
use skia_safe::font_style::Weight;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::query::QueryClient;
use crate::rest::HttpClient;
use crate::rest::endpoints::relationships as rels;
use crate::state::{AppState, DmChannel};
use crate::ui::home::dm_title;
use crate::ui::message_list::ChatColors;
use crate::ui::profile_popout::{ProfilePopoutState, load_profile, separator_dot};
use crate::ui::{Fonts, user_avatar_props};

#[allow(clippy::too_many_arguments)]
pub fn build(
    state: &AppState,
    dm: &DmChannel,
    colors: &ChatColors,
    popout: &ProfilePopoutState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) -> Node {
    let title = dm_title(state, dm);
    let mut root = div().class(cls::CONTAINER);

    let person = dm.recipients.first().copied().filter(|_| !dm.is_group());
    let user = person.and_then(|id| state.users.get(id));
    if let Some(user) = &user {
        let mut props = user_avatar_props(user.id, &user.discriminator, fonts);
        if let Some(image) = user
            .avatar
            .as_deref()
            .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(user.id, hash)))
        {
            props = props.image(image);
        }
        root =
            root.child(avatar_with_status(props, AvatarStatusSize::Px80, None).class(cls::AVATAR));
    }

    root = root.child(
        TextProps::new(title.clone(), 32.0_f32)
            .weight(Weight::BOLD)
            .color(colors.text_strong)
            .build()
            .class(cls::NAME),
    );
    if let Some(user) = &user {
        root = root.child(
            TextProps::new(user.username.clone(), 24.0_f32)
                .weight(Weight::MEDIUM)
                .color(colors.text_strong)
                .build()
                .class(cls::USERNAME),
        );
    }
    let description = requiem_i18n::md!("dm-start-description", name = title.clone());
    root = root.child(
        message_text(MessageTextProps {
            content: description,
            size: 16.0,
            color: colors.text_default,
            line_height: Some(22.0),
            ..Default::default()
        })
        .class(cls::DESCRIPTION),
    );

    let (Some(user_id), Some(user)) = (person, user) else {
        return root;
    };
    let mut actions = div().class(cls::ACTIONS);
    let mutual = load_profile(
        queries,
        http,
        &state.users,
        user_id,
        None,
        ProfileSurface::Modal,
    )
    .and_then(|p| p.mutual_guilds.as_ref().map(|g| g.len()))
    .unwrap_or(0);
    if mutual > 0 {
        actions = actions
            .child(
                TextProps::styled(
                    requiem_i18n::t!("mutual-servers", count = mutual),
                    TextVariant::TEXT_SM_NORMAL,
                )
                .color(colors.text_default)
                .build(),
            )
            .child(separator_dot(colors.text_default));
    }
    let button = |label: String, on_click: Box<dyn Fn()>| {
        ButtonProps::default()
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Small)
            .label(label)
            .on_click(on_click)
            .build()
    };
    let (queries, http) = (queries.clone(), http.cloned());
    match state.relationships.kind_of(user_id) {
        Some(1) => {
            actions = actions.child(button(
                requiem_i18n::t!("home-remove-friend").to_string(),
                Box::new(move || {
                    let Some(http) = http.clone() else { return };
                    queries.mutate(Vec::new(), move || async move {
                        rels::remove(&http, user_id, rels::location::DM_CHANNEL).await
                    });
                }),
            ))
        }
        None => {
            actions = actions.child(button(
                requiem_i18n::t!("home-add-friend").to_string(),
                Box::new(move || {
                    let Some(http) = http.clone() else { return };
                    queries.mutate(Vec::new(), move || async move {
                        rels::send_friend_request(&http, user_id, rels::location::DM_CHANNEL).await
                    });
                }),
            ))
        }
        _ => {}
    }
    if state.relationships.kind_of(user_id) != Some(2) {
        let (overlays, name) = (popout.overlays.clone(), state.users.display_name(&user));
        actions = actions.child(button(
            requiem_i18n::t!("home-block").to_string(),
            Box::new(move || {
                overlays.confirm_block(user_id, name.clone(), rels::location::DM_CHANNEL)
            }),
        ));
    }
    root.child(actions)
}
