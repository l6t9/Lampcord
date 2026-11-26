import QtQuick
import QtQuick.Controls.Basic
import Lampcord

LampMenu {
    id: root

    property string userId: ""
    property string userName: ""
    property string userAvatar: ""
    property string userDecoration: ""
    property string userStatus: "offline"
    property string userActivity: ""

    readonly property bool isMe: userId !== "" && userId === backend.currentUserId
    readonly property var relationship: window.relationships
        ? (window.relationships.find(r => r.id === root.userId) || null) : null
    readonly property bool isFriend: relationship !== null && relationship.kind === 1
    readonly property bool isBlocked: relationship !== null && relationship.kind === 2

    LampMenuItem {
        text: "Profile"
        leading: Component { Icon { name: "AccountCircle"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: {
            if (window.openProfile && root.userId !== "")
                window.openProfile(root.userId, root.userName, root.userAvatar, root.userDecoration, root.userStatus, root.userActivity)
        }
    }
    LampMenuItem {
        text: "Mention"
        leading: Component { Icon { name: "AlternateEmail"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: {
            if (window.mentionUser && root.userId !== "")
                window.mentionUser(root.userId)
        }
    }
    LampMenuItem {
        visible: !root.isMe
        height: visible ? implicitHeight : 0
        text: "Message"
        leading: Component { Icon { name: "ChatBubble"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: backend.openDm(root.userId)
    }

    MenuSeparator {
        visible: !root.isMe
        implicitHeight: visible ? 9 : 0
        height: visible ? 9 : 0
        padding: visible ? 4 : 0
        contentItem: Rectangle { visible: parent.visible; implicitHeight: 1; color: Theme.alpha(Theme.outlineVariant, 0.5) }
    }

    LampMenuItem {
        visible: !root.isMe && root.isFriend
        height: visible ? implicitHeight : 0
        text: "Remove Friend"
        leading: Component { Icon { name: "PersonRemove"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: backend.removeRelationship(root.userId)
    }
    LampMenuItem {
        visible: !root.isMe && !root.isFriend && !root.isBlocked
        height: visible ? implicitHeight : 0
        text: "Add Friend"
        leading: Component { Icon { name: "PersonAdd"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: backend.sendFriendRequest(root.userName)
    }
    LampMenuItem {
        visible: !root.isMe && !root.isBlocked
        height: visible ? implicitHeight : 0
        text: "Block"
        danger: true
        leading: Component { Icon { name: "Block"; size: 17; color: Theme.error } }
        onTriggered: backend.blockUser(root.userId)
    }
    LampMenuItem {
        visible: !root.isMe && root.isBlocked
        height: visible ? implicitHeight : 0
        text: "Unblock"
        leading: Component { Icon { name: "Block"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: backend.removeRelationship(root.userId)
    }

    MenuSeparator {
        padding: 4
        contentItem: Rectangle { implicitHeight: 1; color: Theme.alpha(Theme.outlineVariant, 0.5) }
    }

    LampMenuItem {
        text: "Copy User ID"
        leading: Component { Icon { name: "Dns"; size: 17; color: Theme.surfaceVariantText } }
        onTriggered: {
            if (window.copyText)
                window.copyText(root.userId)
        }
    }
}
