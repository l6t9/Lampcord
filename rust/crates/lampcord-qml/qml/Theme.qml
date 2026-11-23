pragma Singleton
import QtQuick
import Lampcord

// Material 3 tokens, generated in Rust from the wallpaper like the Compose
// desktop theme (see src/palette.rs). Light/dark follows the system.
// M3's "onX" roles are spelled "xText": QML would read a property named
// onSomething as a signal handler.
QtObject {
    readonly property bool dark: Application.styleHints.colorScheme !== Qt.ColorScheme.Light
    readonly property var scheme: JSON.parse(Palette.schemeJson)[dark ? "dark" : "light"]

    readonly property color primary: scheme.primary
    readonly property color primaryText: scheme.onPrimary
    readonly property color primaryContainer: scheme.primaryContainer
    readonly property color primaryContainerText: scheme.onPrimaryContainer
    readonly property color secondaryContainer: scheme.secondaryContainer
    readonly property color secondaryContainerText: scheme.onSecondaryContainer
    readonly property color error: scheme.error
    readonly property color errorText: scheme.onError
    readonly property color errorContainer: scheme.errorContainer
    readonly property color errorContainerText: scheme.onErrorContainer

    readonly property color background: scheme.background
    readonly property color surface: scheme.surface
    readonly property color surfaceContainerLowest: scheme.surfaceContainerLowest
    readonly property color surfaceContainerLow: scheme.surfaceContainerLow
    readonly property color surfaceContainer: scheme.surfaceContainer
    readonly property color surfaceContainerHigh: scheme.surfaceContainerHigh
    readonly property color surfaceContainerHighest: scheme.surfaceContainerHighest
    readonly property color surfaceVariant: scheme.surfaceVariant
    readonly property color surfaceText: scheme.onSurface
    readonly property color surfaceVariantText: scheme.onSurfaceVariant
    readonly property color outline: scheme.outline
    readonly property color outlineVariant: scheme.outlineVariant

    // Compose's FontOption.SYSTEM resolves through fontconfig, like this.
    readonly property string font: "sans-serif"

    // Motion: the Compose client's "default spatial" spring, approximated.
    readonly property int quick: 160
    readonly property int medium: 260

    // The wallpaper can change while we run; the Compose client watches
    // for that too.
    property Timer wallpaperWatch: Timer {
        interval: 2000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: Palette.refresh()
    }

    function alpha(c, a) {
        return Qt.rgba(c.r, c.g, c.b, a)
    }
}
