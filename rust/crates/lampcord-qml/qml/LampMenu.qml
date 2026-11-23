import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects

// M3 DropdownMenu as Compose draws it: surfaceContainer, extra-small
// corners, 220px wide, a short scale-and-fade in.
Menu {
    id: root

    width: 220
    padding: 0
    topPadding: 8
    bottomPadding: 8

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 120 }
        NumberAnimation { property: "scale"; from: 0.85; to: 1; duration: 180; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; to: 0; duration: 90 }
    }

    background: Item {
        implicitWidth: 220
        RectangularShadow {
            anchors.fill: box
            radius: box.radius
            blur: 12
            offset.y: 3
            color: Qt.rgba(0, 0, 0, 0.3)
        }
        Rectangle {
            id: box
            anchors.fill: parent
            radius: 4
            color: Theme.surfaceContainer
        }
    }
}
