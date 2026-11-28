use crate::models;
use requiem_ui::components::{TextProps, TextVariant, div};
use requiem_ui::node::Node;
use requiem_ui::widget::primitives::ImageWidget;
use skia_safe::Color;
use std::collections::hash_map::DefaultHasher;
use std::collections;
use std::env;
use std::hash::Hash;
use std::hash::Hasher;
use super::classes::author_style as cls;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::{GuildMarker, RoleMarker};
use crate::models::user::DisplayNameStyles;
use crate::models::{Id, Role, User};

use crate::state::AppState;

#[derive(Clone, Debug, PartialEq)]
pub enum NameFill {
    Solid(Color),

    Gradient(Vec<Color>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RoleIcon {
    Emoji(String),
    Image {
        role_id: Id<RoleMarker>,
        hash: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct GuildTag {
    pub guild_id: Id<GuildMarker>,
    pub tag: String,
    pub badge: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuthorStyle {
    pub fill: Option<NameFill>,
    pub icon: Option<RoleIcon>,

    pub icon_role_name: Option<String>,
    pub tag: Option<GuildTag>,
}

pub fn guild_roles(state: &AppState, guild_id: Id<GuildMarker>) -> Vec<Role> {
    spoofed_roles(guild_id, state.permissions.guild_roles(guild_id))
}

fn spoof_style() -> bool {
    cfg!(debug_assertions) && env::var_os("REQUIEM_SPOOF_STYLE").is_some()
}

fn spoofed_roles(guild_id: Id<GuildMarker>, roles: Vec<Role>) -> Vec<Role> {
    if !spoof_style() {
        return roles;
    }
    roles
        .into_iter()
        .map(|mut role| {
            if role.id.get() != guild_id.get() {
                role.color = 0xe91e63;
                role.colors = models::RoleColors {
                    primary_color: 0xe91e63,
                    secondary_color: (role.position % 2 == 0).then_some(0x3498db),
                    tertiary_color: None,
                };
                role.unicode_emoji = Some("\u{1f525}".to_string());
            }
            role
        })
        .collect()
}

fn rgb(value: i64) -> Color {
    Color::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

/// `None` for a role with no color: 0 is "unset", not black.
pub fn role_fill(role: &Role) -> Option<NameFill> {
    let primary = if role.colors.primary_color != 0 {
        role.colors.primary_color
    } else {
        role.color
    };
    if primary == 0 {
        return None;
    }
    let primary = rgb(primary);
    let rest: Vec<Color> = [role.colors.secondary_color, role.colors.tertiary_color]
        .into_iter()
        .flatten()
        .map(rgb)
        .collect();
    if rest.is_empty() {
        return Some(NameFill::Solid(primary));
    }
    let mut colors = vec![primary];
    colors.extend(rest);
    colors.push(primary);
    Some(NameFill::Gradient(colors))
}

/// Nitro display-name colors, used when no role color wins.
pub fn display_name_fill(styles: Option<&DisplayNameStyles>) -> Option<NameFill> {
    let colors: Vec<Color> = styles
        .and_then(|styles| styles.colors.as_ref())?
        .iter()
        .copied()
        .filter(|value| *value != 0)
        .map(|value| rgb(value & 0xff_ffff))
        .collect();
    match colors.as_slice() {
        [] => None,
        [only] => Some(NameFill::Solid(*only)),
        _ => {
            let mut colors = colors;
            let first = colors[0];
            colors.push(first);
            Some(NameFill::Gradient(colors))
        }
    }
}

fn role_has_color(role: &Role) -> bool {
    role.color != 0 || role.colors.primary_color != 0
}

pub fn role_icon_of(role: &Role) -> Option<RoleIcon> {
    match (&role.icon, &role.unicode_emoji) {
        (Some(hash), _) if !hash.is_empty() => Some(RoleIcon::Image {
            role_id: role.id,
            hash: hash.clone(),
        }),
        (_, Some(emoji)) if !emoji.is_empty() => Some(RoleIcon::Emoji(emoji.clone())),
        _ => None,
    }
}

/// What `<@&id>` mentions resolve to: name and color, `None` color when the
/// role has none so the mention keeps its normal color.
pub fn role_mention_map(roles: &[Role]) -> collections::HashMap<u64, (String, Option<Color>)> {
    roles
        .iter()
        .map(|role| {
            let color = match role_fill(role) {
                Some(NameFill::Solid(color)) => Some(color),
                Some(NameFill::Gradient(colors)) => colors.first().copied(),
                None => None,
            };
            (role.id.get(), (role.name.clone(), color))
        })
        .collect()
}

pub fn guild_tag(user: &User) -> Option<GuildTag> {
    if spoof_style() {
        return Some(GuildTag {
            guild_id: Id::new(1),
            tag: "TEST".to_string(),
            badge: None,
        });
    }
    tag_of(user.primary_guild.as_ref()?)
}

pub fn tag_of(primary_guild: &models::PrimaryGuild) -> Option<GuildTag> {
    let (guild_id, tag, badge) = primary_guild.shown()?;
    Some(GuildTag {
        guild_id,
        tag: tag.to_string(),
        badge: badge.map(str::to_string),
    })
}

pub fn resolve(
    state: &AppState,
    roles: &[Role],
    guild_id: Option<Id<GuildMarker>>,
    user: &User,
) -> AuthorStyle {
    let member_styles = guild_id.and_then(|guild_id| {
        state
            .members
            .by_guild
            .with_untracked(|all| {
                all.get(&guild_id)
                    .and_then(|members| members.get(&user.id))
                    .and_then(|member| member.display_name_styles.clone())
            })
    });
    let fallback_fill = member_styles
        .as_ref()
        .or_else(|| user.display_name_styles.as_ref())
        .and_then(|styles| display_name_fill(Some(styles)));
    let mut style = AuthorStyle {
        tag: guild_tag(user),
        fill: fallback_fill.clone(),
        ..Default::default()
    };
    let Some(guild_id) = guild_id else {
        return style;
    };
    let member_roles = state
        .members
        .by_guild
        .with_untracked(|all| {
            all.get(&guild_id)
                .and_then(|members| members.get(&user.id))
                .map(|m| m.roles.clone())
        })
        .unwrap_or_default();
    if spoof_style() && user.id.get().is_multiple_of(2) {
        style.fill = Some(NameFill::Solid(rgb(0xe91e63)));
        style.icon = Some(RoleIcon::Emoji("\u{1f525}".to_string()));
        style.icon_role_name = Some("test role".to_string());
    } else if spoof_style() {
        style.fill = Some(NameFill::Gradient(vec![
            rgb(0xe91e63),
            rgb(0x3498db),
            rgb(0xe91e63),
        ]));
        style.icon = Some(RoleIcon::Emoji("\u{1f525}".to_string()));
        style.icon_role_name = Some("test role".to_string());
    }
    if member_roles.is_empty() {
        return style;
    }
    let held = || roles.iter().filter(|r| member_roles.contains(&r.id));
    style.fill = held()
        .filter(|r| role_has_color(r))
        .max_by_key(|r| r.position)
        .and_then(role_fill)
        .or(fallback_fill);
    let icon_role = held()
        .filter(|r| role_icon_of(r).is_some())
        .max_by_key(|r| r.position);
    style.icon = icon_role.and_then(role_icon_of);
    style.icon_role_name = icon_role.map(|r| r.name.clone());
    style
}

impl AuthorStyle {
    pub fn version(&self) -> u64 {
        let mut h = DefaultHasher::new();
        format!("{self:?}").hash(&mut h);
        h.finish()
    }
}

impl AuthorStyle {
    pub fn loaded(&self, images: &ImageCache, backend: &BackendConfig) -> u64 {
        let badge = self
            .tag
            .as_ref()
            .and_then(|t| {
                Some(
                    images
                        .get_or_decode_image(
                            &backend.guild_tag_badge_url(t.guild_id, t.badge.as_deref()?),
                        )
                        .is_some(),
                )
            })
            .unwrap_or(false);
        let icon = match &self.icon {
            Some(RoleIcon::Image { role_id, hash }) => images
                .get_or_decode_image(&backend.role_icon_url(*role_id, hash))
                .is_some(),
            _ => false,
        };
        u64::from(badge) | u64::from(icon) << 1
    }
}

/// One color for a name drawn inside running text, such as a system
/// message's author: the role color, or a gradient's first stop.
pub fn inline_name_color(fill: Option<&NameFill>, default: Color) -> Color {
    match fill {
        Some(NameFill::Solid(color)) => *color,
        Some(NameFill::Gradient(colors)) => colors.first().copied().unwrap_or(default),
        None => default,
    }
}

pub fn name_node(
    text: String,
    variant: TextVariant,
    line_height: f32,
    fill: Option<&NameFill>,
    default: Color,
) -> Node {
    let props = TextProps::styled(text, variant).line_height(line_height);
    match fill {
        Some(NameFill::Solid(color)) => props.color(*color).build(),
        Some(NameFill::Gradient(colors)) => props
            .color(Color::WHITE)
            .gradient(colors.clone())
            .gradient_tile(100.0_f32)
            .build(),
        None => props.color(default).build(),
    }
}

#[derive(Clone, Copy)]
pub enum TagPlace {
    Message,
    Reply,
    List,
    Pill,
}

pub fn tag_chip(
    tag: &GuildTag,
    place: TagPlace,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    let (place_class, variant, badge_class) = match place {
        TagPlace::Message => (
            cls::TAG_CHIP_MESSAGE,
            TextVariant::TEXT_XS_SEMIBOLD,
            Some(cls::TAG_BADGE_14),
        ),
        TagPlace::Reply => (
            cls::TAG_CHIP_REPLY,
            TextVariant::TEXT_XS_SEMIBOLD,
            Some(cls::TAG_BADGE_12),
        ),
        TagPlace::List => (
            cls::TAG_CHIP_LIST,
            TextVariant::TEXT_XS_SEMIBOLD,
            Some(cls::TAG_BADGE_14),
        ),
        TagPlace::Pill => (
            cls::TAG_CHIP_PILL,
            TextVariant::TEXT_XS_NORMAL,
            Some(cls::TAG_BADGE_14),
        ),
    };
    let mut chip = div().class(cls::TAG_CHIP).class(place_class);
    let badge = tag
        .badge
        .as_deref()
        .zip(badge_class)
        .and_then(|(hash, class)| {
            Some((
                images.get_or_decode_image(&backend.guild_tag_badge_url(tag.guild_id, hash))?,
                class,
            ))
        });
    if let Some((image, class)) = badge {
        chip = chip.child(Node::new("div", ImageWidget::new(Some(image))).class(class));
    }
    chip.child(
        TextProps::styled(tag.tag.clone(), variant)
            .color(super::TEXT_STRONG)
            .line_clamp(1_u32)
            .build(),
    )
}

pub fn role_icon_node(
    icon: &RoleIcon,
    class: &'static str,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Option<Node> {
    match icon {
        RoleIcon::Emoji(glyph) => {
            let (image, (x, y, w, h)) = super::twemoji::tile(glyph)?;
            Some(
                Node::new(
                    "div",
                    ImageWidget::new(Some(image)).with_src_rect(x, y, w, h),
                )
                .class(class),
            )
        }
        RoleIcon::Image { role_id, hash } => {
            let image = images.get_or_decode_image(&backend.role_icon_url(*role_id, hash))?;
            Some(Node::new("div", ImageWidget::new(Some(image))).class(class))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_styles_become_solid_or_gradient_fills() {
        let solid = DisplayNameStyles {
            colors: Some(vec![0x33_66_99]),
            ..Default::default()
        };
        assert_eq!(
            display_name_fill(Some(&solid)),
            Some(NameFill::Solid(rgb(0x33_66_99)))
        );

        let gradient = DisplayNameStyles {
            colors: Some(vec![0xff_00_00, 0x00_ff_00]),
            ..Default::default()
        };
        assert_eq!(
            display_name_fill(Some(&gradient)),
            Some(NameFill::Gradient(vec![
                rgb(0xff_00_00),
                rgb(0x00_ff_00),
                rgb(0xff_00_00),
            ]))
        );
        assert_eq!(display_name_fill(None), None);
    }
}
