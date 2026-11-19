use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};

struct Root {
    title: SharedString,
    detail: SharedString,
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size(px(640.))
            .justify_center()
            .items_center()
            .bg(rgb(0x313338))
            .text_color(rgb(0xf2f3f5))
            .text_xl()
            .child(self.title.clone())
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xb5bac1))
                    .child(self.detail.clone()),
            )
    }
}

fn main() {
    let model_ready = lampcord_model::channel::ChannelType::GUILD_VOICE.is_voice();
    let core_ready = lampcord_core::captcha::parse_captcha_challenge(
        r#"{"captcha_key":["x"],"captcha_service":"hcaptcha","captcha_sitekey":"site"}"#,
    )
    .is_some();

    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(640.), px(420.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|_| Root {
                    title: "Lampcord rewrite".into(),
                    detail: format!(
                        "P0 protocol models: {} | P1/P2 core: {}",
                        if model_ready { "ready" } else { "missing" },
                        if core_ready { "ready" } else { "missing" }
                    )
                    .into(),
                })
            },
        )
        .expect("failed to open Lampcord window");
        cx.activate(true);
    });
}
