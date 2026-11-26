import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import Lampcord

// M3 Expressive ContextMenu / DropdownMenu matching Compose:
// 16px rounded corners, compact width, subtle elevation shadow.
Menu {
    id: root

    width: 156
    padding: 0
    topPadding: 6
    bottomPadding: 6

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 120 }
        NumberAnimation { property: "scale"; from: 0.9; to: 1; duration: 160; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; to: 0; duration: 90 }
    }

    background: Item {
        implicitWidth: 156
        RectangularShadow {
            anchors.fill: box
            radius: box.radius
            blur: 16
            offset.y: 4
            color: Qt.rgba(0, 0, 0, 0.35)
        }
        Rectangle {
            id: box
            anchors.fill: parent
            radius: 16
            color: Theme.surfaceContainer
        }
    }
}
