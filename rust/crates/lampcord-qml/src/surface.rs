#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("surface_format.h");

        /// Must run before the QGuiApplication exists.
        #[cxx_name = "requestTrueColorSurfaces"]
        fn request_true_color_surfaces();
    }
}
