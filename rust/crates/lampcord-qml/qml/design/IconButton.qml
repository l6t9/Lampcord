import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// Circular icon button. `container` set to a colour makes it a filled one.
AbstractButton {
    id: control

    property string glyph
    property real iconSize: 24
    property color iconColor: Theme.surfaceVariantText
    property color container: "transparent"
    property string tip

    implicitWidth: 40
    implicitHeight: 40
    hoverEnabled: true

    contentItem: Item {
        Icon {
            anchors.centerIn: parent
            name: control.glyph
            size: control.iconSize
            color: control.enabled ? control.iconColor : Theme.alpha(control.iconColor, 0.38)
        }
    }

    background: Rectangle {
        radius: width / 2
        color: control.container

        Rectangle {
            anchors.fill: parent
            radius: parent.radius
            color: control.iconColor
            opacity: !control.enabled ? 0 : control.down ? 0.12 : control.hovered ? 0.08 : 0
            Behavior on opacity { NumberAnimation { duration: Theme.quick } }
        }
    }

    RichTooltip {
        anchor: control
        placement: "above"
        visible: control.tip !== "" && control.hovered
        text: control.tip
    }

    HoverHandler { cursorShape: control.enabled ? Qt.PointingHandCursor : Qt.ArrowCursor }
}
