//! # gpui-material
//!
//! Material 3 Expressive UI components for GPUI.
//!
//! ## Overview
//!
//! `gpui-material` provides modular, high-performance UI components adhering to
//! Material 3 Expressive design, built for GPUI applications.
//!
//! Key design characteristics:
//! - **Uniform API**: builder methods (`size`, `variant`, `shape`, `colors`, `metrics`, `spring`).
//! - **Spring Physics**: harmonic oscillator physics for responsive corner morphing and state layers.
//! - **Zero Heap Allocations**: frame evaluation uses stack-allocated Copy types and `SharedString`.
//! - **Settled Idle**: animation loops halt immediately when springs rest, dropping idle CPU/GPU usage to zero.
//! - **Theming**: standard light and dark palettes with full per-component color role overrides.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use gpui_material::button::{ButtonModel, ButtonSize, ButtonVariant};
//! use gpui_material::theme::light_scheme;
//!
//! let btn = ButtonModel::new(0, "Action")
//!     .size(ButtonSize::Medium)
//!     .variant(ButtonVariant::Filled);
//!
//! let palette = light_scheme();
//! let element = btn.render(&palette);
//! ```
//!
//! ## Components
//!
//! - [`button`]: Five sizes, five emphasis styles, round/square shapes, spring corner morphing.
//! - [`blob`]: Liquid surfaces, rounded rectangles that smooth-merge and squash and stretch.
//! - [`card`]: Filled, elevated, and outlined cards with spring corner morphing.
//! - [`carousel`]: M3 center-aligned multi-browse carousel with masking and snapping.
//! - [`icon`]: Material Symbols icons for menus and buttons.
//! - [`menu`]: M3 Expressive menus with connected corner shaping, item pills, group gap separation, and nested submenus.
//! - [`progress_ring`]: Circular progress indicator, the ring form of the progress bar.
//! - [`shape`]: 35 Material 3 Expressive shapes, point-sampled morphing, and harmonic spring bounce.
//! - [`slider`]: Continuous and discrete sliders with stepped snap intervals.
//! - [`split_button`]: Primary action button with a morphing contextual trailing button.
//! - [`switch`]: Pill toggle with animated thumb expansion and morphing check/cross icons.
//! - [`motion`]: Damped harmonic oscillator spring engine (`expressive` and `standard` profiles).
//! - [`theme`]: M3 Expressive color tokens, baseline palette roles, light and dark schemes.
//!
//! ## Interactive Showcase
//!
//! The repository includes an interactive showcase previewing all components.
//! Run it locally with:
//!
//! ```bash
//! cargo run --example showcase
//! ```

mod components;
pub mod motion;
pub mod theme;

// Top-level module exports
pub use components::blob;
pub use components::button;
pub use components::card;
pub use components::carousel;
pub use components::icon;
pub use components::menu;
pub use components::progress_bar;
pub use components::progress_ring;
pub use components::shape;
pub use components::slider;
pub use components::split_button;
pub use components::switch;

#[doc(inline)]
pub use components::blob::{
    BlobRect, BlobShape, Deform, DeformSpring, Edge, LavaBlob, LavaLamp, blob_contours,
    blob_shape_contours, blob_shape_contours_colored, paint_blob_shadow, paint_blobs,
    paint_contours, paint_shape_blobs, paint_shape_blobs_colored,
};
#[doc(inline)]
pub use components::button::{
    ButtonColors, ButtonMetrics, ButtonModel, ButtonShape, ButtonSize, ButtonVariant,
};
#[doc(inline)]
pub use components::card::{CardColors, CardMetrics, CardModel, CardVariant};
#[doc(inline)]
pub use components::icon::M3Icon;
#[doc(inline)]
pub use components::menu::{
    MenuColors, MenuGroup, MenuItem, MenuItemTrailing, MenuMetrics, MenuModel, MenuVariant,
};
#[doc(inline)]
pub use components::progress_bar::{ProgressBar, ProgressBarColors, ProgressBarMetrics};
pub use components::progress_ring::{ProgressRing, ProgressRingMetrics};
#[doc(inline)]
pub use components::shape::{
    MaterialShape, SHAPE_SAMPLE_COUNT, damping_from_ratio, draw_morph, draw_morph_points,
    draw_morph_with_bounce, draw_points, draw_shape, draw_shape_morph_with_bounce,
    lerp_and_rotate_points,
};
#[doc(inline)]
pub use components::slider::{
    SliderColors, SliderConfiguration, SliderMetrics, SliderModel, SliderSpec,
};
#[doc(inline)]
pub use components::split_button::{
    SplitButtonColors, SplitButtonMetrics, SplitButtonModel, SplitButtonSize, SplitButtonVariant,
    SplitIconKind, draw_chevron_icon, draw_split_icon,
};
#[doc(inline)]
pub use components::switch::{
    IconSegment, SwitchColors, SwitchMetrics, SwitchModel, animated_icon_segments,
    draw_switch_icon, icon_segments, thumb_geometry,
};
#[doc(inline)]
pub use motion::{SpringParams, SpringState, spring_step};
#[doc(inline)]
pub use theme::{M3Palette, dark_scheme, light_scheme};

/// Convenient prelude module for importing common models, enums, and theme helpers.
pub mod prelude {
    pub use crate::blob::{BlobRect, BlobShape, LavaLamp};
    pub use crate::button::{ButtonModel, ButtonShape, ButtonSize, ButtonVariant};
    pub use crate::card::{CardColors, CardMetrics, CardModel, CardVariant};
    pub use crate::icon::M3Icon;
    pub use crate::menu::{
        MenuColors, MenuGroup, MenuItem, MenuItemTrailing, MenuMetrics, MenuModel, MenuVariant,
    };
    pub use crate::motion::{SpringParams, SpringState};
    pub use crate::progress_bar::{ProgressBar, ProgressBarColors, ProgressBarMetrics};
    pub use crate::shape::MaterialShape;
    pub use crate::slider::{SliderColors, SliderMetrics, SliderSpec};
    pub use crate::split_button::{
        SplitButtonColors, SplitButtonMetrics, SplitButtonModel, SplitButtonSize,
        SplitButtonVariant, SplitIconKind,
    };
    pub use crate::switch::{SwitchColors, SwitchMetrics, SwitchModel};
    pub use crate::theme::{M3Palette, dark_scheme, light_scheme};
}
