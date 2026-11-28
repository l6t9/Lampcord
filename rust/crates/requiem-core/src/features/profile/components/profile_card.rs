use crate::ui::author_style::TagPlace;
use crate::ui::author_style;
use crate::ui::classes::profile_popout;
use crate::ui::classes::profile_popout as cls;
use crate::ui::dim;
use crate::ui::emoji_text;
use crate::ui::twemoji;
use requiem_ui::components::AvatarStatusSize;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Align, Padding, StackProps};
use requiem_ui::components;
use requiem_ui::widget::primitives::MarkdownFeatures;
use requiem_ui::widget::primitives::StatusCutout;
use std::rc::Rc;

use skia_safe::gradient::shaders::linear_gradient;
use skia_safe::gradient::{Colors, Gradient, Interpolation};
use skia_safe::{Canvas, Color, Color4f, Paint, Point, RRect, Rect, TileMode};

use requiem_ui::components::{ButtonProps, TextProps, avatar_with_status, div};
use requiem_ui::node::Node;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::{
    ImageWidget, PresenceStatus, corner_radii, has_any_corner_radius,
};
use requiem_ui::widget::{AbsoluteRect, Widget};

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::UserMarker;
use crate::models::{Id, UserProfile};
use crate::presence::CustomStatus;

use crate::ui::custom_status::{BubbleActions, BubbleContent, bubble};
use crate::ui::profile_popout::{ColorBoxWidget, badges_row, separator_dot};
use crate::ui::{Fonts, TEXT_DEFAULT, TEXT_STRONG, user_avatar_props};

pub struct GradientBoxWidget {
    pub top: Color,
    pub stop: f32,
    pub bottom: Color,
}

impl Widget for GradientBoxWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let stop = (self.stop / rect.height.max(1.0)).clamp(0.0, 1.0);
        let colors = [
            Color4f::from(self.top),
            Color4f::from(self.top),
            Color4f::from(self.bottom),
        ];
        let positions = [0.0, stop, 1.0];
        let gradient = Gradient::new(
            Colors::new(&colors, Some(positions.as_slice()), TileMode::Clamp, None),
            Interpolation::default(),
        );
        let Some(shader) = linear_gradient(
            (
                Point::new(rect.x, rect.y),
                Point::new(rect.x, rect.y + rect.height),
            ),
            &gradient,
            None,
        ) else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_anti_alias(true);
        if has_any_corner_radius(style) {
            canvas.draw_rrect(RRect::new_rect_radii(bounds, &corner_radii(style)), &paint);
        } else {
            canvas.draw_rect(bounds, &paint);
        }
    }
}

pub struct ProfileCardView<'a> {
    pub user_id: Id<UserMarker>,
    pub discriminator: &'a str,
    pub username: &'a str,
    pub display_name: &'a str,

    pub pronouns: &'a str,
    pub bio: &'a str,
    pub avatar: Option<skia_safe::Image>,
    pub avatar_decoration: Option<skia_safe::Image>,
    pub banner_image: Option<skia_safe::Image>,
    pub banner_color: Color,

    pub theme: Option<(Color, Color)>,

    pub name_color: Option<Color>,

    pub status: Option<PresenceStatus>,

    pub bot: bool,

    pub badges_from: Option<&'a UserProfile>,
}

pub struct CardStatus<'a> {
    pub status: Option<&'a CustomStatus>,

    pub actions: Option<&'a BubbleActions>,
    pub prompt: &'a str,
}

#[derive(Default)]
pub struct CardParts {
    pub open_profile: Option<Rc<dyn Fn()>>,

    pub name_suffix: Vec<Node>,

    pub header_extra: Vec<Node>,

    /// Effect and frame layers that sit over the banner and avatar.
    pub header_layers: Vec<Node>,

    /// A nameplate at 40% opacity behind the whole card.
    pub background_layers: Vec<Node>,

    pub after_username: Vec<Node>,

    pub after_bio: Vec<Node>,

    pub roomy_body: bool,

    /// A callout above the name, e.g. that part of the profile failed to load.
    pub notice: Option<Node>,
}

pub enum CardContainer {
    Inline,

    Panel(Node),
}

pub fn footer_slot(child: Node) -> Node {
    div()
        .class(profile_popout::FOOTER)
        .child(child)
}

/// A user's bio with its inline markdown and emoji.
pub fn bio_text(
    bio: &str,
    size: f32,
    line_height: f32,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    components::message_text(components::MessageTextProps {
        content: bio.to_string(),
        size,
        color: TEXT_STRONG,
        line_height: Some(line_height),
        emoji_provider: Some(twemoji::tile),
        custom_emoji: Some(emoji_text::resolver(images, backend)),
        markdown: Some(MarkdownFeatures::BIO),
        ..Default::default()
    })
}

#[allow(clippy::too_many_arguments)]
pub fn profile_card(
    view: ProfileCardView<'_>,
    status: CardStatus<'_>,
    mut parts: CardParts,
    footer: Option<Node>,
    container: CardContainer,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    let themed = matches!(container, CardContainer::Inline)
        .then_some(view.theme)
        .flatten();
    let background_layers = std::mem::take(&mut parts.background_layers);

    let banner = match &view.banner_image {
        Some(image) => Node::new(
            "div",
            ImageWidget::new(Some(image.clone())).with_cover_centered(),
        )
        .class(cls::BANNER),
        None => Node::new(
            "div",
            ColorBoxWidget {
                color: view.banner_color,
            },
        )
        .class(cls::BANNER),
    };

    let mut avatar_props = user_avatar_props(view.user_id, view.discriminator, fonts);
    if let Some(image) = &view.avatar {
        avatar_props = avatar_props.image(image.clone());
    }
    if let Some(image) = &view.avatar_decoration {
        avatar_props = avatar_props.decoration(image.clone());
    }
    let avatar = avatar_with_status(avatar_props, AvatarStatusSize::Px80, view.status);

    let avatar_wrap = match &parts.open_profile {
        Some(open) => {
            let open = open.clone();
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![cls::AVATAR_WRAP.to_string()])
                .on_click(move || open())
                .build()
                .child(avatar)
                .child(
                    Node::new(
                        "div",
                        dim::DimWidget {
                            cutout: view
                                .status
                                .map(|_| StatusCutout::ROUND_80),
                        },
                    )
                    .class(cls::AVATAR_OVERLAY),
                )
        }
        None => div().class(cls::AVATAR_WRAP).child(avatar),
    };

    let mut ring = div().class(cls::AVATAR_RING);
    if let Some((primary, _)) = themed {
        let dim = |c: u8| (c as u16 * 40 / 100) as u8;
        ring = ring.style(format!(
            "background-color: #{:02x}{:02x}{:02x}",
            dim(primary.r()),
            dim(primary.g()),
            dim(primary.b())
        ));
    }

    let mut header = div()
        .class(cls::HEADER)
        .children([banner, ring, avatar_wrap]);
    header = header.children(std::mem::take(&mut parts.header_layers));
    if status.actions.is_some() || status.status.is_some() {
        let content = match status.status {
            Some(custom) => BubbleContent::Status(custom),
            None => BubbleContent::Prompt(status.prompt),
        };
        header =
            header.child(bubble(content, status.actions, images, backend).class(cls::BUBBLE_SLOT));
    }
    header = header.children(parts.header_extra);

    let linkable = |inner: Node| match &parts.open_profile {
        Some(open) => {
            let open = open.clone();
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![cls::HOVER_UNDERLINE.to_string()])
                .on_click(move || open())
                .build()
                .child(inner)
        }
        None => inner,
    };
    let name = TextProps::styled(view.display_name.to_string(), TextVariant::TEXT_LG_BOLD)
        .color(view.name_color.unwrap_or(TEXT_DEFAULT))
        .build()
        .class(cls::DISPLAY_NAME);
    let mut name_row = StackProps::horizontal()
        .align(Align::Center)
        .full_width(false)
        .build()
        .child(linkable(name));
    if view.bot {
        name_row = name_row.child(
            div().class(cls::BOT_BADGE).child(
                TextProps::styled(
                    requiem_i18n::t!("common-app-tag"),
                    TextVariant::TEXT_XS_SEMIBOLD,
                )
                .color(Color::WHITE)
                .build()
                .class(cls::BOT_BADGE_TEXT),
            ),
        );
    }
    name_row = name_row.children(parts.name_suffix);

    let username_text = TextProps::styled(view.username.to_string(), TextVariant::TEXT_SM_NORMAL)
        .color(TEXT_STRONG)
        .build()
        .class(cls::USERNAME_TEXT);
    let mut username_row = div()
        .class(cls::USERNAME_ROW)
        .child(linkable(username_text));
    let pronouns = view.pronouns.trim();
    if pronouns.is_empty() {
        username_row = username_row.class(cls::USERNAME_ROW_NO_PRONOUNS);
    } else {
        username_row = username_row.child(separator_dot(TEXT_DEFAULT)).child(
            TextProps::styled(pronouns.to_string(), TextVariant::TEXT_SM_NORMAL)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
        );
    }
    if let Some(tag) = view
        .badges_from
        .and_then(|p| author_style::tag_of(p.user.primary_guild.as_ref()?))
    {
        username_row = username_row.child(author_style::tag_chip(
            &tag,
            TagPlace::Pill,
            images,
            backend,
        ));
    }
    if let Some(badges) = view
        .badges_from
        .and_then(|p| badges_row(p, images, backend))
    {
        username_row = username_row.child(badges);
    }

    let mut body = StackProps::vertical()
        .gap(12)
        .full_width(false)
        .padding(Padding::top(4).with_right(16).with_bottom(8).with_left(16))
        .build();
    if parts.roomy_body {
        body = body.class(cls::BODY_ROOMY);
    }

    if let Some(notice) = parts.notice {
        body = body.child(notice);
    }
    body = body
        .child(
            StackProps::vertical()
                .gap(0)
                .full_width(false)
                .build()
                .children([name_row, username_row]),
        )
        .children(parts.after_username);
    if !view.bio.trim().is_empty() {
        body = body.child(
            div()
                .class(cls::BIO)
                .class(components::line_clamp_class(3))
                .child(bio_text(view.bio, 14.0, 18.0, images, backend)),
        );
    }
    body = body.children(parts.after_bio);

    match container {
        CardContainer::Panel(panel) => {
            let mut panel = panel;
            for layer in background_layers {
                panel = panel.child(layer);
            }
            let panel = panel.child(header).child(body);
            match footer {
                Some(footer) => panel.child(footer),
                None => panel,
            }
        }
        CardContainer::Inline => {
            let mut content = background_layers;
            content.push(header);
            content.push(body);
            content.extend(footer);
            match themed {
                None => div()
                    .class(cls::PANEL)
                    .class(cls::PANEL_INLINE)
                    .children(content),
                Some((primary, accent)) => {
                    let inner = div().class(cls::THEMED_INNER).children(content);
                    Node::new(
                        "div",
                        GradientBoxWidget {
                            top: primary,
                            stop: 105.0,
                            bottom: accent,
                        },
                    )
                    .class(cls::THEMED_OUTER)
                    .child(inner)
                }
            }
        }
    }
}
