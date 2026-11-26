import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// RegularGuildItem: 48px squircle that tightens from 24 to 16 radius on
// hover/selection, with the left pill indicator.
Item {
    id: root

    property string guildId: ""
    property bool selected: false
    property bool unread: false
    property int mentions: 0
    property bool mobile: false
    property string icon
    property string label
    property string glyph       // Icon name used instead of initials
    property string tip

    signal clicked()

    implicitWidth: 72
    implicitHeight: 48

    readonly property bool hovered: hover.hovered
    readonly property bool active: selected || hovered
    readonly property bool monogram: icon === "" || !image.ready

    Rectangle {
        id: indicator
        anchors.left: parent.left
        anchors.leftMargin: root.mobile ? -4 : 0
        anchors.verticalCenter: parent.verticalCenter
        width: root.mobile ? 8 : 4
        height: root.mobile
            ? (root.selected || root.unread || root.hovered ? 8 : 0)
            : parent.height * (root.selected ? 0.8 : root.hovered ? 0.4 : root.unread ? 0.15 : 0)
        radius: root.mobile ? 4 : 0
        topRightRadius: root.mobile ? 4 : 4
        bottomRightRadius: root.mobile ? 4 : 4
        color: Theme.surfaceText
        opacity: root.selected || root.hovered || root.unread ? 1 : 0

        Behavior on height { NumberAnimation { duration: Theme.quick; easing.type: Easing.OutCubic } }
        Behavior on opacity { NumberAnimation { duration: Theme.quick } }
    }

    Item {
        id: tile
        anchors.centerIn: parent
        width: 48
        height: 48

        // Phoenix visual refresh settles guild tiles at radius 12. Desktop
        // keeps the older squircle-to-rounded transition.
        property real radius: root.mobile ? 12 : (root.active ? 16 : 24)
        Behavior on radius {
            NumberAnimation {
                duration: root.mobile ? 100 : Theme.medium
                easing.type: Easing.OutCubic
            }
        }

        Rectangle {
            anchors.fill: parent
            visible: root.monogram
            radius: tile.radius
            color: root.active ? Theme.primary : Theme.surfaceVariant
            Behavior on color { ColorAnimation { duration: 100 } }

            Icon {
                anchors.centerIn: parent
                visible: root.glyph !== ""
                name: root.glyph
                size: root.mobile ? 24 : 28
                color: root.active ? Theme.primaryText : Theme.primary
            }

            Text {
                anchors.centerIn: parent
                width: parent.width - 6
                visible: root.glyph === ""
                text: root.label
                color: root.active ? Theme.primaryText : Theme.primary
                font.family: Theme.font
                font.pixelSize: text.length > 2 ? 13 : 16
                font.weight: Font.DemiBold
                horizontalAlignment: Text.AlignHCenter
                elide: Text.ElideRight
            }
        }

        RoundedImage {
            id: image
            anchors.fill: parent
            radius: tile.radius
            source: root.icon
        }

        MentionBadge {
            x: parent.width - width + 3
            y: parent.height - height + 3
            visible: root.mentions > 0
            count: root.mentions
        }
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.PointingHandCursor
    }

    TextEdit {
        id: clipHelper
        visible: false
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor
        onClicked: mouse => {
            if (mouse.button === Qt.RightButton && root.guildId !== "") {
                guildMenu.popup(mouse.x, mouse.y)
            } else if (mouse.button === Qt.LeftButton) {
                root.clicked()
            }
        }
    }

    LampMenu {
        id: guildMenu

        LampMenuItem {
            text: "Mute Server"
            leading: Component { Icon { name: "VolumeOff"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: if (window.openServerSettings) window.openServerSettings(root.guildId, root.tip)
        }
        LampMenuItem {
            text: "Mark as Read"
            leading: Component { Icon { name: "Check"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: backend.markGuildRead(root.guildId)
        }
        LampMenuItem {
            text: "Edit Profile"
            leading: Component { Icon { name: "AccountCircle"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: if (window.openSettings) window.openSettings("profile")
        }
        MenuSeparator {
            padding: 4
            contentItem: Rectangle { implicitHeight: 1; color: Theme.alpha(Theme.outlineVariant, 0.5) }
        }
        LampMenuItem {
            text: "Leave Server"
            danger: true
            leading: Component { Icon { name: "Logout"; size: 17; color: Theme.error } }
            onTriggered: backend.leaveGuild(root.guildId)
        }
        MenuSeparator {
            padding: 4
            contentItem: Rectangle { implicitHeight: 1; color: Theme.alpha(Theme.outlineVariant, 0.5) }
        }
        LampMenuItem {
            text: "Copy ID"
            leading: Component { Icon { name: "Dns"; size: 17; color: Theme.surfaceVariantText } }
            onTriggered: {
                clipHelper.text = root.guildId
                clipHelper.selectAll()
                clipHelper.copy()
            }
        }
    }

    RichTooltip {
        anchor: root
        x: tile.x + tile.width + 12
        visible: root.hovered && root.tip !== ""
        text: root.tip
    }
}
