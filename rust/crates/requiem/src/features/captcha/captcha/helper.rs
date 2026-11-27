#![allow(clippy::print_stdout, clippy::print_stderr)]

use requiem_core::utils::prefs;
use std::io::{BufRead, Write};
use std::io;
use std::process;
use std::thread;
use std::time::Duration;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use tao::platform::unix::WindowExtUnix;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use wry::WebViewBuilderExtUnix;

use serde::Deserialize;
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::{Theme, WindowBuilder};
use wry::{PageLoadEvent, WebContext, WebViewBuilder};

use super::{EXIT_UNAVAILABLE, Request, Response, page};

const WINDOW_SIZE: (f64, f64) = (480.0, 720.0);
const MIN_SIZE: (f64, f64) = (440.0, 560.0);

const BACKGROUND: (u8, u8, u8, u8) = (0x12, 0x12, 0x14, 0xff);

const ORIGIN_TIMEOUT: Duration = Duration::from_secs(8);

enum Msg {
    PageLoaded(String),
    OriginTimeout,
    Page(String),

    ParentGone,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum PageMessage {
    Solved { token: String },
    Cancel,
    Failed { code: String },
}

pub fn run() -> i32 {
    let stdin = io::stdin();
    let mut line = String::new();
    let request: Request = match stdin
        .lock()
        .read_line(&mut line)
        .ok()
        .and_then(|_| serde_json::from_str(line.trim()).ok())
    {
        Some(request) => request,
        None => {
            eprintln!("captcha: no valid request on stdin");
            return EXIT_UNAVAILABLE;
        }
    };

    let event_loop = EventLoopBuilder::<Msg>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    {
        let proxy = proxy.clone();
        thread::spawn(move || {
            let mut sink = String::new();
            while stdin
                .lock()
                .read_line(&mut sink)
                .is_ok_and(|read| read > 0)
            {
                sink.clear();
            }
            let _ = proxy.send_event(Msg::ParentGone);
        });
    }

    let (width, height) = event_loop.primary_monitor().map_or(WINDOW_SIZE, |monitor| {
        let scale = monitor.scale_factor();
        let screen = monitor.size();
        (
            WINDOW_SIZE.0.min(screen.width as f64 / scale - 48.0),
            WINDOW_SIZE.1.min(screen.height as f64 / scale - 96.0),
        )
    });
    let mut window = WindowBuilder::new()
        .with_title(requiem_i18n::t!("captcha-title"))
        .with_inner_size(LogicalSize::new(width, height))
        .with_min_inner_size(LogicalSize::new(
            MIN_SIZE.0.min(width),
            MIN_SIZE.1.min(height),
        ))
        .with_theme(Some(Theme::Dark))
        .with_visible(false);
    if let Some(monitor) = event_loop.primary_monitor() {
        let scale = monitor.scale_factor();
        let screen = monitor.size();
        window = window.with_position(LogicalPosition::new(
            (screen.width as f64 / scale - width) / 2.0,
            (screen.height as f64 / scale - height) / 2.0,
        ));
    }
    let window = match window.build(&event_loop) {
        Ok(window) => window,
        Err(err) => return unavailable(format!("no window: {err}")),
    };

    let mut context = WebContext::new(
        prefs::config_dir().map(|dir| dir.join("captcha-webview")),
    );
    let origin_url = format!("{}/robots.txt", request.origin.trim_end_matches('/'));
    let builder = WebViewBuilder::new_with_web_context(&mut context)
        .with_background_color(BACKGROUND)
        .with_url(origin_url.clone())
        .with_on_page_load_handler({
            let proxy = proxy.clone();
            move |event, url| {
                if matches!(event, PageLoadEvent::Finished) {
                    let _ = proxy.send_event(Msg::PageLoaded(url));
                }
            }
        })
        .with_ipc_handler({
            let proxy = proxy.clone();
            move |message| {
                let _ = proxy.send_event(Msg::Page(message.into_body()));
            }
        });
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let webview = builder.build(&window);
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let webview = {
        builder.build_gtk(window.default_vbox().expect("tao windows have a vbox"))
    };
    let webview = match webview {
        Ok(webview) => webview,
        Err(err) => return unavailable(format!("no webview: {err}")),
    };

    thread::spawn({
        let proxy = proxy.clone();
        move || {
            thread::sleep(ORIGIN_TIMEOUT);
            let _ = proxy.send_event(Msg::OriginTimeout);
        }
    });

    let mut shown = false;
    let mut page_written = false;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let _ = &context;
        let finish = |response: Response| -> ! {
            let mut stdout = io::stdout().lock();
            let _ = writeln!(
                stdout,
                "{}",
                serde_json::to_string(&response).expect("a response serializes")
            );
            let _ = stdout.flush();

            process::exit(0);
        };
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::UserEvent(Msg::ParentGone) => finish(Response::Cancelled),
            Event::UserEvent(Msg::PageLoaded(url)) if !page_written => {
                page_written = true;
                let on_origin = url.starts_with(request.origin.trim_end_matches('/'));
                let result = if on_origin {
                    let document = serde_json::to_string(&page::html(&request, false))
                        .expect("a string serializes");
                    webview.evaluate_script(&format!(
                        "document.open();document.write({document});document.close();"
                    ))
                } else {
                    log::warn!(
                        "captcha: {} did not load (got {url}); using the standalone page",
                        request.origin
                    );
                    webview.load_html(&page::html(&request, true))
                };
                if let Err(err) = result {
                    log::warn!("captcha: could not show the page: {err}");
                }
                if !shown {
                    shown = true;
                    window.set_visible(true);
                    window.set_focus();
                }
            }
            Event::UserEvent(Msg::OriginTimeout) if !page_written => {
                page_written = true;
                log::warn!(
                    "captcha: {} did not load in time; using the standalone page",
                    request.origin
                );
                let _ = webview.load_html(&page::html(&request, true));
                window.set_visible(true);
                window.set_focus();
                shown = true;
            }
            Event::UserEvent(Msg::Page(body)) => match serde_json::from_str(&body) {
                Ok(PageMessage::Solved { token }) => finish(Response::Solved { key: token }),
                Ok(PageMessage::Cancel) => finish(Response::Cancelled),
                Ok(PageMessage::Failed { code }) => finish(Response::Failed { code }),
                Err(err) => log::warn!("captcha: unreadable message from the page: {err}"),
            },
            _ => {}
        }
    })
}

fn unavailable(reason: String) -> i32 {
    eprintln!("captcha: {reason}");
    println!(
        "{}",
        serde_json::to_string(&Response::Unavailable { reason }).expect("a response serializes")
    );
    EXIT_UNAVAILABLE
}
