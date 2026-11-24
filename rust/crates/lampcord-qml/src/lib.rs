pub mod backend;
pub mod emoji;
pub mod icons;
pub mod palette;
pub mod surface;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

/// Entry point shared by the desktop binary and the Android Qt activity.
///
/// Qt's Android template calls the native `main()` from its loaded library;
/// the C++ wrapper in `android/src/main.cpp` calls this function instead.
#[unsafe(no_mangle)]
pub extern "C" fn lampcord_qml_main() -> i32 {
    surface::ffi::request_true_color_surfaces();

    let mut app = QGuiApplication::new();
    if let Some(mut app) = app.as_mut() {
        app.as_mut()
            .set_application_name(&QString::from("Lampcord"));
        app.as_mut()
            .set_organization_name(&QString::from("Lampcord"));
    }

    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(qml_entry()));
    }

    app.as_mut().map(|app| app.exec()).unwrap_or(1)
}

fn qml_entry() -> &'static str {
    if cfg!(target_os = "android") || std::env::var_os("LAMPCORD_QML_MOBILE").is_some() {
        "qrc:/qt/qml/Lampcord/qml/MobileMain.qml"
    } else {
        "qrc:/qt/qml/Lampcord/qml/Main.qml"
    }
}
