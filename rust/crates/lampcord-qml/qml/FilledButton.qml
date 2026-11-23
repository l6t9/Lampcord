import QtQuick
import QtQuick.Controls.Basic

// M3 filled button at the login screen's 56dp height and large shape.
Button {
    id: control

    property bool loading: false

    implicitHeight: 56
    hoverEnabled: true
    font.family: Theme.font
    font.pixelSize: 16
    font.weight: Font.DemiBold
    font.letterSpacing: 0.15

    contentItem: Item {
        Text {
            anchors.centerIn: parent
            visible: !control.loading
            text: control.text
            font: control.font
            color: control.enabled ? Theme.primaryText : Theme.alpha(Theme.surfaceText, 0.38)
        }
        LoadingIndicator {
            anchors.centerIn: parent
            visible: control.loading
            size: 24
            contained: true
            containerColor: Theme.primaryText
            color: Theme.primary
        }
    }

    background: Rectangle {
        radius: 18
        color: control.enabled || control.loading ? Theme.primary : Theme.alpha(Theme.surfaceText, 0.12)

        Rectangle {
            anchors.fill: parent
            radius: parent.radius
            color: Theme.primaryText
            opacity: control.down ? 0.12 : control.hovered ? 0.08 : 0
            Behavior on opacity { NumberAnimation { duration: Theme.quick } }
        }
    }

    HoverHandler { cursorShape: Qt.PointingHandCursor }
}
