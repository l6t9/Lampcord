//! Start-up wiring for the desktop shell. `run` in `main.rs` calls these in order: the shared
//! services and channels, the root signals and their callbacks, the session lifecycle, and finally
//! the event loop's hooks and user-event handler.

use requiem_core::ui::emoji_anim;
use requiem_core::ui::requiem_emoji;
use requiem_core::video::VideoCache;
use requiem_ui::perf;
use requiem_ui::text_cache::MeasureCache;
use requiem_ui::waker::FrameWaker;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as SyncMutex;
use tokio::sync::mpsc;

use requiem_ui::components::toast;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use requiem_core::images::ImageCache;
use requiem_core::models::id::marker::ChannelMarker;
use requiem_core::models::Id;
use requiem_core::presence::PresenceUpdate;
use requiem_core::query::QueryClient;
use requiem_core::rest;
use winit::event_loop::EventLoopProxy;

use crate::{AppEvent, BackgroundReceivers, GifQuery, UploadCommand};


mod events;
mod hooks;
mod session;
mod signals;
mod wiring;

pub(crate) use events::EventHandler;
pub(crate) use hooks::{
    AppEnv, build_app, make_modal_open, register_popouts,
};
pub(crate) use session::{LogoutEnv, SessionEnv, make_end_session, make_start_session};
pub(crate) use signals::build_root_signals;
pub(crate) use wiring::{wire_callbacks, wire_search_jump};

/// The sending halves of the channels the background tasks listen on.
#[derive(Clone)]
pub(crate) struct Senders {
    pub compose_tx: mpsc::UnboundedSender<(Id<ChannelMarker>, String)>,
    pub upload_tx: mpsc::UnboundedSender<UploadCommand>,
    pub select_tx: mpsc::UnboundedSender<Id<ChannelMarker>>,
    pub gif_query_tx: mpsc::UnboundedSender<GifQuery>,
    pub status_tx: mpsc::UnboundedSender<PresenceUpdate>,
}

/// Creates the channels the UI thread uses to talk to the background tasks.
pub(crate) fn channels() -> (Senders, BackgroundReceivers) {
    let (compose_tx, compose_rx) =
        mpsc::unbounded_channel::<(Id<ChannelMarker>, String)>();
    let (upload_tx, upload_rx) = mpsc::unbounded_channel::<UploadCommand>();
    let (select_tx, select_rx) = mpsc::unbounded_channel::<Id<ChannelMarker>>();
    let (gif_query_tx, gif_query_rx) = mpsc::unbounded_channel::<GifQuery>();
    let (status_tx, status_rx) = mpsc::unbounded_channel::<PresenceUpdate>();
    let senders = Senders {
        compose_tx,
        upload_tx,
        select_tx,
        gif_query_tx,
        status_tx,
    };
    let receivers = BackgroundReceivers {
        compose_rx: Arc::new(SyncMutex::new(compose_rx)),
        upload_rx: Arc::new(SyncMutex::new(upload_rx)),
        select_rx: Arc::new(SyncMutex::new(select_rx)),
        gif_query_rx: Arc::new(SyncMutex::new(gif_query_rx)),
        status_rx: Arc::new(SyncMutex::new(status_rx)),
    };
    (senders, receivers)
}

/// Long-lived handles shared by the UI and the background tasks.
#[derive(Clone)]
pub(crate) struct Services {
    pub proxy_slot: Arc<Mutex<Option<EventLoopProxy<AppEvent>>>>,
    pub images: Rc<RefCell<Option<ImageCache>>>,
    pub queries: Rc<RefCell<Option<QueryClient>>>,
    pub emoji_insert_proxy: Rc<RefCell<Option<EventLoopProxy<AppEvent>>>>,
    pub measure_cache: Rc<MeasureCache>,
    pub frame_waker: FrameWaker,
    pub videos: VideoCache,
    pub toasts: toast::ToastState,
}

impl Services {
    pub(crate) fn new(runtime: &Runtime) -> Self {
        let proxy_slot: Arc<Mutex<Option<EventLoopProxy<AppEvent>>>> = Arc::new(Mutex::new(None));
        let wake = {
            let proxy_slot = proxy_slot.clone();
            move || {
                if let Some(proxy) = proxy_slot.lock().expect("proxy slot poisoned").as_ref() {
                    let _ = proxy.send_event(AppEvent::Wake);
                }
            }
        };
        let images: Rc<RefCell<Option<ImageCache>>> = Rc::new(RefCell::new(Some(ImageCache::new(
            runtime.handle().clone(),
            rest::build_image_client().expect("failed to build reqwest client"),
            wake.clone(),
        ))));

        requiem_emoji::init(images.borrow().clone());
        if let Some(images) = images.borrow().as_ref() {
            images.register_mem_probe();
        }
        let queries: Rc<RefCell<Option<QueryClient>>> = Rc::new(RefCell::new(Some(QueryClient::new(
            runtime.handle().clone(),
            wake,
        ))));

        let emoji_insert_proxy: Rc<RefCell<Option<EventLoopProxy<AppEvent>>>> =
            Rc::new(RefCell::new(None));

        let measure_cache = Rc::new(MeasureCache::default());

        let frame_waker = FrameWaker::default();
        let videos = VideoCache::new(
            runtime.handle().clone(),
            rest::build_video_client().expect("failed to build reqwest client"),
        )
        .with_frame_waker(frame_waker.clone());
        {
            let videos = videos.clone();
            perf::register_mem_probe(move || {
                let (anims, frames, bytes) = emoji_anim::stats();
                format!(
                    "video players: {} | animated emoji: {anims} ({frames} frames, {})",
                    videos.open_players(),
                    perf::mb(bytes)
                )
            });
        }
        let toasts = toast::ToastState::new();
        Self {
            proxy_slot,
            images,
            queries,
            emoji_insert_proxy,
            measure_cache,
            frame_waker,
            videos,
            toasts,
        }
    }
}
