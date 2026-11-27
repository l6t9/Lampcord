use gl::types;
use std::cmp::Reverse;
use std::env;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext,
    PossiblyCurrentGlContext,
};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::prelude::GlSurface;
use glutin::surface::{
    Surface as GlutinSurface, SurfaceAttributesBuilder, SwapInterval, WindowSurface,
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::gpu::{self, DirectContext, SurfaceOrigin, backend_render_targets, direct_contexts};
use skia_safe::{ColorType, Surface as SkSurface};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

pub struct GlWindow {
    pub window: Arc<Window>,
    pub gl_context: PossiblyCurrentContext,
    pub gl_surface: GlutinSurface<WindowSurface>,

    pub vsync: bool,

    vsync_relaxed: bool,
}

impl GlWindow {
    pub fn new(event_loop: &ActiveEventLoop, attrs: WindowAttributes) -> Self {
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(true);

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(attrs));
        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        let transparency_check = config.supports_transparency().unwrap_or(false)
                            & !accum.supports_transparency().unwrap_or(false);

                        if transparency_check || config.num_samples() < accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .expect("failed to build GL display/config");

        let window = window.expect("could not create window with OpenGL context");
        let raw_window_handle = window
            .window_handle()
            .expect("failed to retrieve window handle")
            .as_raw();

        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(raw_window_handle));

        // SAFETY: `raw_window_handle` belongs to the `window` that is kept alive for the lifetime
        // of the context, as glutin requires.
        let not_current_gl_context = unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_config
                        .display()
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create GL context")
                })
        };

        let (width, height): (u32, u32) = window.inner_size().into();
        let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width.max(1)).unwrap(),
            NonZeroU32::new(height.max(1)).unwrap(),
        );

        // SAFETY: `raw_window_handle` belongs to the `window` that is kept alive for the lifetime
        // of the surface, as glutin requires.
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &surface_attrs)
                .expect("could not create GL window surface")
        };

        let gl_context = not_current_gl_context
            .make_current(&gl_surface)
            .expect("could not make GL context current");

        let swap_interval = SwapInterval::Wait(NonZeroU32::new(1).expect("1 is non-zero"));
        let vsync = match gl_surface.set_swap_interval(&gl_context, swap_interval) {
            Ok(()) => true,
            Err(err) => {
                log::warn!(
                    "could not enable vsync ({swap_interval:?}): {err}; falling back to timer-paced frames"
                );
                false
            }
        };

        gl::load_with(|s| {
            gl_config
                .display()
                .get_proc_address(CString::new(s).unwrap().as_c_str())
        });

        Self {
            window: Arc::new(window),
            gl_context,
            gl_surface,
            vsync,
            vsync_relaxed: false,
        }
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.gl_surface.resize(
            &self.gl_context,
            NonZeroU32::new(width.max(1)).unwrap(),
            NonZeroU32::new(height.max(1)).unwrap(),
        );
    }

    pub fn vsync_active(&self) -> bool {
        self.vsync && !self.vsync_relaxed
    }

    pub fn set_live_resize(&mut self, resizing: bool) {
        if !self.vsync || self.vsync_relaxed == resizing {
            return;
        }
        let interval = if resizing {
            SwapInterval::DontWait
        } else {
            SwapInterval::Wait(NonZeroU32::new(1).expect("1 is non-zero"))
        };
        if self
            .gl_surface
            .set_swap_interval(&self.gl_context, interval)
            .is_ok()
        {
            self.vsync_relaxed = resizing;
        }
    }

    pub fn make_current(&self) {
        let _ = self.gl_context.make_current(&self.gl_surface);
    }

    pub fn swap_buffers(&self) {
        self.gl_surface
            .swap_buffers(&self.gl_context)
            .expect("failed to swap buffers");
    }
}

pub struct SkiaGl {
    pub gl: GlWindow,
    gr_context: DirectContext,
    surface: SkSurface,

    frame_cache: Option<SkSurface>,

    last_present: Option<Instant>,
    fb_info: FramebufferInfo,
    num_samples: usize,
    stencil_size: usize,
}

impl Drop for SkiaGl {
    fn drop(&mut self) {
        self.gl.make_current();
        self.frame_cache = None;
        self.gr_context.release_resources_and_abandon();
    }
}

impl SkiaGl {
    pub fn new(event_loop: &ActiveEventLoop, attrs: WindowAttributes) -> Self {
        let gl = GlWindow::new(event_loop, attrs);

        let interface = gpu::gl::Interface::new_load_with(|name| {
            if name == "eglGetCurrentDisplay" {
                return ptr::null();
            }
            gl.gl_context
                .display()
                .get_proc_address(CString::new(name).unwrap().as_c_str())
        })
        .expect("could not create Skia GL interface");

        let mut gr_context = {
            let mut options = gpu::ContextOptions::default();
            if let Some(kb) = env::var("UI_GLYPH_ATLAS_KB")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
            {
                options.glyph_cache_texture_maximum_bytes = kb * 1024;
            }
            direct_contexts::make_gl(interface, &options)
                .expect("could not create Skia DirectContext")
        };

        let limit_mb = env::var("UI_GPU_CACHE_MB")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(64);
        gr_context.set_resource_cache_limit(limit_mb * 1024 * 1024);

        let fb_info = {
            let mut fboid: types::GLint = 0;
            // SAFETY: a GL context is current on this thread, and `fboid` is a valid out-pointer
            // for one GLint.
            unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };
            FramebufferInfo {
                fboid: fboid.try_into().unwrap(),
                format: gpu::gl::Format::RGBA8.into(),
                ..Default::default()
            }
        };

        let num_samples = 0;
        let stencil_size = 8;

        let surface = Self::create_surface(
            &gl.window,
            fb_info,
            &mut gr_context,
            num_samples,
            stencil_size,
        );

        Self {
            gl,
            gr_context,
            surface,
            frame_cache: None,
            last_present: None,
            fb_info,
            num_samples,
            stencil_size,
        }
    }

    fn create_surface(
        window: &Window,
        fb_info: FramebufferInfo,
        gr_context: &mut DirectContext,
        num_samples: usize,
        stencil_size: usize,
    ) -> SkSurface {
        let size = window.inner_size();
        let size = (
            size.width.try_into().expect("could not convert width"),
            size.height.try_into().expect("could not convert height"),
        );
        let backend_render_target =
            backend_render_targets::make_gl(size, num_samples, stencil_size, fb_info);

        gpu::surfaces::wrap_backend_render_target(
            gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        )
        .expect("could not create Skia surface")
    }

    pub fn begin(&mut self) {
        self.gl.make_current();
        self.gr_context.reset(None);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.begin();
        self.gl.resize(width, height);
        self.surface = Self::create_surface(
            &self.gl.window,
            self.fb_info,
            &mut self.gr_context,
            self.num_samples,
            self.stencil_size,
        );
    }

    pub fn canvas(&mut self) -> &skia_safe::Canvas {
        self.surface.canvas()
    }

    pub fn frame_cache_canvas(&mut self) -> (&skia_safe::Canvas, bool) {
        let (width, height) = (self.surface.width(), self.surface.height());
        let stale = self
            .frame_cache
            .as_ref()
            .is_none_or(|cache| cache.width() != width || cache.height() != height);
        if stale {
            let info = skia_safe::ImageInfo::new(
                (width, height),
                ColorType::RGBA8888,
                skia_safe::AlphaType::Premul,
                None,
            );
            self.frame_cache = gpu::surfaces::render_target(
                &mut self.gr_context,
                gpu::Budgeted::Yes,
                &info,
                None,
                SurfaceOrigin::BottomLeft,
                None,
                false,
                None,
            );
        }
        let cache = self
            .frame_cache
            .as_mut()
            .expect("could not create the offscreen frame cache");
        (cache.canvas(), stale)
    }

    pub fn blit_frame_cache(&mut self) {
        if let Some(cache) = self.frame_cache.as_mut() {
            cache.draw(
                self.surface.canvas(),
                (0, 0),
                skia_safe::SamplingOptions::default(),
                None,
            );
        }
    }

    pub fn gpu_cache_stats(&self) -> (usize, usize, usize) {
        let usage = self.gr_context.resource_cache_usage();
        (
            usage.resource_count,
            usage.resource_bytes,
            self.gr_context.resource_cache_limit(),
        )
    }

    pub fn gpu_dump(&mut self) -> String {
        self.begin();
        let mut found: Vec<(usize, String)> = Vec::new();
        let (mut total, mut count) = (0usize, 0usize);

        // SAFETY: a GL context is current on this thread; every call below passes valid
        // out-pointers or ids that were just checked with `IsTexture`.
        unsafe {
            let mut prev: types::GLint = 0;
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut prev);
            for id in 1..8192u32 {
                if gl::IsTexture(id) == gl::FALSE {
                    continue;
                }
                gl::BindTexture(gl::TEXTURE_2D, id);
                let (mut bytes, mut levels, mut w0, mut h0, mut fmt) = (0usize, 0, 0, 0, 0);
                for level in 0..16 {
                    let (mut w, mut h): (types::GLint, types::GLint) = (0, 0);
                    gl::GetTexLevelParameteriv(gl::TEXTURE_2D, level, gl::TEXTURE_WIDTH, &mut w);
                    gl::GetTexLevelParameteriv(gl::TEXTURE_2D, level, gl::TEXTURE_HEIGHT, &mut h);
                    if w <= 0 || h <= 0 {
                        break;
                    }
                    if level == 0 {
                        w0 = w;
                        h0 = h;
                        gl::GetTexLevelParameteriv(
                            gl::TEXTURE_2D,
                            0,
                            gl::TEXTURE_INTERNAL_FORMAT,
                            &mut fmt,
                        );
                    }
                    levels += 1;
                    bytes += w as usize * h as usize * 4;
                }
                if bytes > 0 {
                    total += bytes;
                    count += 1;
                    found.push((bytes, format!("{w0}x{h0} fmt {fmt:#x} {levels} lvl")));
                }
            }
            gl::BindTexture(gl::TEXTURE_2D, prev as u32);
        }
        self.gr_context.reset(None);
        found.sort_unstable_by_key(|a| Reverse(a.0));
        let top: Vec<String> = found
            .iter()
            .take(12)
            .map(|(b, d)| format!("{d} {:.1}MB", *b as f64 / 1048576.0))
            .collect();
        format!(
            "gl textures: {count} ~{:.1}MB (as 4 B/px) | top: {}",
            total as f64 / 1048576.0,
            top.join("; ")
        )
    }

    pub fn gpu_cache_purgeable(&self) -> usize {
        self.gr_context.resource_cache_purgeable_bytes()
    }

    const IDLE_CLEANUP: Duration = Duration::from_secs(10);

    pub fn maintain_gpu_cache(&mut self) -> Option<Instant> {
        let due = self.last_present? + Self::IDLE_CLEANUP;
        if Instant::now() < due {
            return Some(due);
        }
        self.last_present = None;
        self.begin();
        self.gr_context
            .perform_deferred_cleanup(Self::IDLE_CLEANUP, None);
        None
    }

    pub fn present(&mut self) {
        self.last_present = Some(Instant::now());
        self.gr_context.flush_and_submit();
        self.gl.swap_buffers();
    }

    #[cfg(debug_assertions)]
    pub fn screenshot_png(&mut self) -> Option<Vec<u8>> {
        let image = self.surface.image_snapshot();
        let data = image.encode(
            &mut self.gr_context,
            skia_safe::EncodedImageFormat::PNG,
            None,
        )?;
        Some(data.as_bytes().to_vec())
    }
}
