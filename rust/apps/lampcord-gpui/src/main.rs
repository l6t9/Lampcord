use std::sync::{Arc, Mutex};

use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use lampcord_core::{
    event_dispatch::Dispatcher,
    state_cache::{StateCache, StateCacheHandler},
};

struct Root {
    title: SharedString,
    detail: SharedString,
    cache: Arc<Mutex<StateCache>>,
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let (guild_count, channel_count, message_count) = {
            let cache = self.cache.lock().expect("state cache lock");
            (
                cache.guild_count(),
                cache.channel_count(),
                cache.cached_message_count(),
            )
        };

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
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x949ba4))
                    .child(SharedString::from(format!(
                        "State cache: {guild_count} guilds | {channel_count} channels | {message_count} messages"
                    ))),
            )
    }
}

fn main() {
    let model_ready = lampcord_model::channel::ChannelType::GUILD_VOICE.is_voice();
    let core_ready = lampcord_core::captcha::parse_captcha_challenge(
        r#"{"captcha_key":["x"],"captcha_service":"hcaptcha","captcha_sitekey":"site"}"#,
    )
    .is_some();
    let state_cache = Arc::new(Mutex::new(StateCache::new()));
    let mut dispatcher = Dispatcher::new();
    dispatcher.register(Arc::new(StateCacheHandler::new(state_cache.clone())));
    let _ = dispatcher;
    let cache_for_app = state_cache.clone();

    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(640.), px(420.)), cx);
        let cache_for_window = cache_for_app.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| {
                let cache = cache_for_window.clone();
                cx.new(|_| Root {
                    title: "Lampcord rewrite".into(),
                    detail: format!(
                        "P0 protocol models: {} | P1/P2 core: {}",
                        if model_ready { "ready" } else { "missing" },
                        if core_ready { "ready" } else { "missing" }
                    )
                    .into(),
                    cache,
                })
            },
        )
        .expect("failed to open Lampcord window");
        cx.activate(true);
    });
}
