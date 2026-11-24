cmake_minimum_required(VERSION 3.24)

project(lampcord_qml_android LANGUAGES CXX)

# androiddeployqt generates the Gradle wrapper but not a settings file. Add one
# so Gradle treats the generated project as its own build instead of walking up
# into the repository root settings.gradle.kts.
file(MAKE_DIRECTORY "${CMAKE_BINARY_DIR}/android-build")
configure_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/settings.gradle"
    "${CMAKE_BINARY_DIR}/android-build/settings.gradle"
    COPYONLY
)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2 QuickDialogs2)

qt_standard_project_setup()

add_library(lampcord_rust STATIC IMPORTED)
set_target_properties(lampcord_rust PROPERTIES
    IMPORTED_LOCATION "${LAMPCORD_RUST_LIB}"
)

qt_add_executable(lampcord_qml MANUAL_FINALIZATION
    src/main.cpp
)

target_link_libraries(lampcord_qml PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Qml
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::QuickDialogs2
    # The staticlib contains QML cache and Qt resource initializers in object
    # files that nothing references directly. Pull in the whole archive so the
    # Android shared library actually registers the QML module.
    "$<LINK_LIBRARY:WHOLE_ARCHIVE,lampcord_rust>"
)

set_target_properties(lampcord_qml PROPERTIES
    QT_ANDROID_APPLICATION_ID "me.lampu.lampcord.qml"
    QT_ANDROID_APP_NAME "Lampcord"
    QT_ANDROID_VERSION_CODE 1
    QT_ANDROID_VERSION_NAME "0.1.0"
    QT_ANDROID_MIN_SDK_VERSION 24
    QT_ANDROID_TARGET_SDK_VERSION 34
)

qt_finalize_executable(lampcord_qml)
