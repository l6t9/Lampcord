import QtQuick

// A nameplate collectible behind a user row: cropped, at 40% opacity.
RoundedImage {
    property bool shown: true

    radius: 8
    opacity: shown && ready ? 0.4 : 0
    Behavior on opacity { NumberAnimation { duration: Theme.quick } }
}
