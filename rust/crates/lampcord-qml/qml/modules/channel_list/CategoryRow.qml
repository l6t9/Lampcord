import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Item {
    id: root

    property string categoryId: ""
    property string name
    property bool collapsed: false
    property bool mobile: false
    signal toggled()

    implicitHeight: root.mobile ? 32 : 12 + 20 + 4

    readonly property color tint: hover.hovered ? Theme.surfaceText : Theme.surfaceVariantText

    Row {
        x: 16
        y: root.mobile ? 6 : 12
        height: 20
        spacing: 4

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.name
            color: root.tint
            font.family: Theme.font
            font.pixelSize: 12
            font.weight: Font.Bold
            font.letterSpacing: 0.25
        }
        Icon {
            anchors.verticalCenter: parent.verticalCenter
            name: root.collapsed ? "ChevronRight" : "KeyboardArrowDown"
            size: 16
            color: root.tint
        }
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.PointingHandCursor
    }
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor
        onClicked: mouse => {
            if (mouse.button === Qt.RightButton)
                catMenu.popup(mouse.x, mouse.y)
            else
                root.toggled()
        }
    }

    LampMenu {
        id: catMenu

        LampMenuItem {
            text: "Mark As Read"
            leading: Component { Icon { name: "Check"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: {
                if (backend.selectedGuildId !== "")
                    backend.markGuildRead(backend.selectedGuildId)
            }
        }

        MenuSeparator {
            padding: 4
            contentItem: Rectangle {
                implicitHeight: 1
                color: Theme.outlineVariant
            }
        }

        LampMenuItem {
            text: "Copy ID"
            leading: Component { Icon { name: "Dns"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: if (root.categoryId !== "" && window.copyText) window.copyText(root.categoryId)
        }
    }
}
