import QtQuick
import QtQuick.Controls.Basic
import Lampcord

LampMenu {
    id: root
    property string messageId: ""
    property string authorId: ""
    property string author: ""
    property string avatar: ""
    property string decoration: ""
    property string content: ""
    property bool mine: false
    property var files: []

    signal editRequested()
    signal replyRequested()
    signal deleteRequested()
    signal copyTextRequested(string text)

    width: 192

    function openAt(sourceItem, mx, my, alignRight) {
        const overlay = Overlay.overlay
        if (overlay) {
            const pt = sourceItem.mapToItem(overlay, mx, my)
            parent = overlay
            const w = width
            const h = implicitHeight
            let px = alignRight
                ? Math.max(8, Math.min(pt.x - w, overlay.width - w - 8))
                : Math.max(8, Math.min(pt.x, overlay.width - w - 8))
            let py = alignRight ? pt.y + 4 : pt.y
            if (py + h > overlay.height - 8)
                py = Math.max(8, pt.y - h - (alignRight ? 4 : 0))
            popup(px, py)
        } else {
            popup(mx, my)
        }
    }

    LampMenuItem {
        visible: root.mine
        text: "Edit Message"
        leading: Component { Icon { name: "Edit"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: root.editRequested()
    }
    LampMenuItem {
        text: "Reply"
        leading: Component { Icon { name: "Rounded.Reply"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: root.replyRequested()
    }
    LampMenuItem {
        text: "Forward"
        leading: Component { Icon { name: "Forward"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: root.copyTextRequested(root.content)
    }
    LampMenuItem {
        visible: root.authorId !== ""
        text: "Profile"
        leading: Component { Icon { name: "AccountCircle"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: {
            if (window.openProfile)
                window.openProfile(root.authorId, root.author, root.avatar, root.decoration, "offline", "")
        }
    }
    MenuSeparator {
        implicitHeight: 9
        height: 9
        padding: 4
        contentItem: Rectangle { implicitHeight: 1; color: Theme.alpha(Theme.outlineVariant, 0.5) }
    }
    LampMenuItem {
        text: "Mark Unread"
        leading: Component { Icon { name: "VisibilityOff"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: backend.ackLatest()
    }
    MenuSeparator {
        implicitHeight: 9
        height: 9
        padding: 4
        contentItem: Rectangle { implicitHeight: 1; color: Theme.alpha(Theme.outlineVariant, 0.5) }
    }
    LampMenuItem {
        visible: root.content !== ""
        text: "Copy Text"
        leading: Component { Icon { name: "ContentCopy"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: root.copyTextRequested(root.content)
    }
    LampMenuItem {
        text: "Copy Link"
        leading: Component { Icon { name: "Link"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: {
            const guild = backend.selectedGuildId !== "" ? backend.selectedGuildId : "@me"
            root.copyTextRequested("https://discord.com/channels/" + guild + "/" + backend.selectedChannelId + "/" + root.messageId)
        }
    }
    LampMenuItem {
        visible: root.files.length > 0
        text: "Save Files"
        leading: Component { Icon { name: "Download"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: {
            for (const f of root.files)
                Qt.openUrlExternally(f.url)
        }
    }
    MenuSeparator {
        visible: root.mine
        implicitHeight: visible ? 9 : 0
        height: visible ? 9 : 0
        padding: visible ? 4 : 0
        contentItem: Rectangle { visible: parent.visible; implicitHeight: 1; color: Theme.alpha(Theme.outlineVariant, 0.5) }
    }
    LampMenuItem {
        visible: root.mine
        text: "Delete Message"
        danger: true
        leading: Component { Icon { name: "Delete"; size: 17; color: Theme.error } }
        onTriggered: root.deleteRequested()
    }
    MenuSeparator {
        implicitHeight: 9
        height: 9
        padding: 4
        contentItem: Rectangle { implicitHeight: 1; color: Theme.alpha(Theme.outlineVariant, 0.5) }
    }
    LampMenuItem {
        text: "Copy Message ID"
        leading: Component { Icon { name: "Dns"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: root.copyTextRequested(root.messageId)
    }
    LampMenuItem {
        visible: root.authorId !== ""
        text: "Copy Author ID"
        leading: Component { Icon { name: "Dns"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: root.copyTextRequested(root.authorId)
    }
}
