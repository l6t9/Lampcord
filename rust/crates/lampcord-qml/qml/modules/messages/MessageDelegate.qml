import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Item {
    id: root

    required property string author
    required property string avatar
    required property string decoration
    required property string content
    required property real time
    required property bool edited
    required property bool grouped
    required property string system
    required property int kind
    required property string attachments
    required property string replyAuthor
    required property string replyAvatar
    required property string replyContent

    property string systemPrefix: ""
    property bool showDate: false
    property string dateLabel: ""
    property bool hasNextSameUser: false
    property bool deleted: false
    property bool mentioned: false
    property bool pinned: false
    property string clanTag: ""
    property string clanBadge: ""
    property string reactions: "[]"
    property string stickers: "[]"
    property string embeds: "[]"
    property string forwarded: "[]"

    // Message identity comes from the model role in chat. Pins and inbox rows
    // pass empty ids, so their action menu stays unavailable.
    required property string messageId
    required property string authorId
    property bool mine: false

    signal actionReply(string id)
    signal actionEdit(string id, string content)
    signal actionDelete(string id)
    signal actionPin(string id, bool isPinned)
    signal actionReact(string id)
    signal actionToggleReaction(string id, string key, bool hasMe)

    readonly property bool isSystem: system !== ""
    readonly property var files: {
        try { return JSON.parse(attachments) } catch (e) { return [] }
    }
    readonly property var reactionList: {
        try { return JSON.parse(reactions) } catch (e) { return [] }
    }
    readonly property var stickerList: {
        try { return JSON.parse(stickers) } catch (e) { return [] }
    }
    readonly property var embedList: {
        try { return JSON.parse(embeds) } catch (e) { return [] }
    }
    readonly property var forwardedList: {
        try { return JSON.parse(forwarded) } catch (e) { return [] }
    }
    readonly property bool useBubbles: Theme.prefs.chatBubbles
    readonly property real dateOffset: (showDate && dateLabel !== "") ? 30 : 0
    readonly property real topPad: useBubbles
        ? (grouped ? 1.5 : 6)
        : (grouped ? (Theme.prefs.compactMessages ? 0.5 : 1.5) : (Theme.prefs.compactMessages ? 4 : 6))
    readonly property real bottomPad: useBubbles
        ? (hasNextSameUser ? 1.5 : 6)
        : 2

    implicitHeight: dateOffset + (isSystem ? systemRow.implicitHeight + 12 : body.implicitHeight + topPad + bottomPad)
    z: hover.hovered ? 10 : 1

    function copyText(value) {
        clipHelper.text = value
        clipHelper.selectAll()
        clipHelper.copy()
    }

    Item {
        id: dateSep
        visible: root.showDate && root.dateLabel !== ""
        x: 16
        y: 6
        width: parent.width - 32
        height: 20

        Rectangle {
            anchors.left: parent.left
            anchors.right: datePill.left
            anchors.rightMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            height: 1
            color: Theme.alpha(Theme.outlineVariant, 0.6)
        }
        Text {
            id: datePill
            anchors.centerIn: parent
            text: root.dateLabel
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 11
            font.weight: Font.DemiBold
        }
        Rectangle {
            anchors.left: datePill.right
            anchors.leftMargin: 10
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            height: 1
            color: Theme.alpha(Theme.outlineVariant, 0.6)
        }
    }

    Rectangle {
        y: root.dateOffset
        width: parent.width
        height: parent.height - root.dateOffset
        visible: root.deleted || root.mentioned || !root.useBubbles || root.isSystem
        color: root.deleted
            ? Theme.alpha(Theme.error, hover.hovered ? 0.19 : 0.13)
            : root.mentioned
                ? Theme.alpha(Theme.primary, hover.hovered ? 0.16 : 0.10)
                : Theme.alpha(Theme.surfaceVariant, hover.hovered ? 0.12 : 0)
        Behavior on color { ColorAnimation { duration: 90 } }

        Rectangle {
            visible: root.deleted || root.mentioned
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: 2
            color: root.deleted ? Theme.error : Theme.primary
        }
    }

    Row {
        id: systemRow
        visible: root.isSystem
        x: 8
        y: root.dateOffset + 6
        width: parent.width - 24
        spacing: 8

        Item {
            width: 40
            height: systemText.implicitHeight

            Icon {
                anchors.centerIn: parent
                size: 18
                name: ({ 1: "PersonAdd", 2: "PersonRemove", 3: "VolumeUp", 4: "Edit", 5: "Image",
                         6: "PushPin", 7: "ArrowForward", 12: "Campaign", 18: "Tag" })[root.kind] || "Info"
                color: ({ 1: "#23A559", 2: "#F23F43", 3: Theme.primary, 6: Theme.primary, 7: "#23A559",
                          8: "#EB459E", 9: "#EB459E", 10: "#EB459E", 11: "#EB459E", 12: Theme.primary,
                          18: Theme.primary })[root.kind] || Theme.surfaceVariantText
            }
        }

        Text {
            id: systemText
            width: parent.width - 48
            textFormat: Text.StyledText
            text: root.escapeHtml(root.systemPrefix)
                  + "<b><font color=\"" + Theme.surfaceText + "\">" + root.escapeHtml(root.author) + "</font></b>"
                  + root.escapeHtml(root.system) + "&nbsp;&nbsp;<font size=\"1\" color=\""
                  + Theme.alpha(Theme.surfaceVariantText, 0.6) + "\">" + root.stamp(root.time) + "</font>"
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 14
            font.letterSpacing: 0.25
            wrapMode: Text.Wrap
        }
    }

    Item {
        id: body
        visible: !root.isSystem
        x: 8
        y: root.dateOffset + root.topPad
        width: parent.width - 16
        implicitHeight: (reply.visible ? reply.height + 2 : 0) + bubble.height

        Item {
            id: reply
            visible: root.replyAuthor !== ""
            x: 48
            width: parent.width - 48
            height: visible ? 22 : 0

            Rectangle {
                x: -28
                y: 10
                width: 24
                height: 14
                color: "transparent"
                border.width: 2
                border.color: Theme.outlineVariant
                topLeftRadius: 6
                // Only the top and left edges should show.
                Rectangle { x: 2; y: 2; width: parent.width; height: parent.height; color: Theme.surface }
            }

            Row {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width
                spacing: 4

                Avatar {
                    anchors.verticalCenter: parent.verticalCenter
                    size: 16
                    name: root.replyAuthor
                    source: root.replyAvatar
                }
                Text {
                    id: replyName
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.replyAuthor
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 12
                    font.weight: Font.Bold
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - replyName.width - 28
                    text: root.replyContent !== "" ? root.replyContent.replace(/\n/g, " ") : "Click to see attachment"
                    color: Theme.surfaceVariantText
                    font.family: Theme.font
                    font.pixelSize: 12
                    font.italic: root.replyContent === ""
                    elide: Text.ElideRight
                }
            }
        }

        UserAvatar {
            id: authorAvatar
            visible: !root.grouped
            y: reply.visible ? reply.height + 2 : 0
            size: 40
            name: root.author
            source: root.avatar
            decoration: root.decoration

            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton
                cursorShape: Qt.PointingHandCursor
                onClicked: mouse => {
                    if (root.authorId !== "" && window.openProfile) {
                        const pt = mapToItem(Overlay.overlay, width, 0)
                        window.openProfile(root.authorId, root.author, root.avatar, root.decoration, "offline", "", pt.x, pt.y)
                    }
                    mouse.accepted = true
                }
            }
        }

        // Follow-ups show their time in the avatar gutter on hover.
        Text {
            id: gutterTime
            visible: root.grouped && hover.hovered
            width: 40
            y: (reply.visible ? reply.height + 2 : 0) + (root.useBubbles ? 12 : 3)
            text: Qt.formatTime(new Date(root.time), "h:mm")
            color: Theme.alpha(Theme.surfaceVariantText, 0.5)
            font.family: Theme.font
            font.pixelSize: 10
            horizontalAlignment: Text.AlignHCenter

            HoverHandler { id: gutterTimeHover }
            RichTooltip {
                anchor: gutterTime
                placement: "above"
                visible: gutterTimeHover.hovered
                text: root.fullStamp(root.time)
            }
        }

        Rectangle {
            id: bubble
            x: 48
            y: reply.visible ? reply.height + 2 : 0
            width: parent.width - 48
            height: column.implicitHeight + (root.useBubbles ? 20 : 0)
            readonly property real rOuter: 18
            readonly property real rInner: 4
            topLeftRadius: root.useBubbles ? (!root.grouped ? rOuter : rInner) : 0
            topRightRadius: root.useBubbles ? (!root.grouped ? rOuter : rInner) : 0
            bottomLeftRadius: root.useBubbles ? (!root.hasNextSameUser ? rOuter : rInner) : 0
            bottomRightRadius: root.useBubbles ? (!root.hasNextSameUser ? rOuter : rInner) : 0
            color: !root.useBubbles || root.deleted ? "transparent"
                 : hover.hovered ? Theme.surfaceContainerHighest
                 : Theme.surfaceContainerHigh
            Behavior on color { ColorAnimation { duration: 90 } }

            Column {
                id: column
                x: root.useBubbles ? 14 : 0
                y: root.useBubbles ? 10 : 0
                width: parent.width - (root.useBubbles ? 28 : 0)
                spacing: 2

                Row {
                    visible: !root.grouped
                    spacing: 8
                    bottomPadding: 1

                    Text {
                        id: nameText
                        text: root.author
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.weight: Font.Bold

                        MouseArea {
                            anchors.fill: parent
                            acceptedButtons: Qt.LeftButton
                            cursorShape: Qt.PointingHandCursor
                            onClicked: mouse => {
                                if (root.authorId !== "" && window.openProfile) {
                                    const pt = mapToItem(Overlay.overlay, width, 0)
                                    window.openProfile(root.authorId, root.author, root.avatar, root.decoration, "offline", "", pt.x, pt.y)
                                }
                                mouse.accepted = true
                            }
                        }
                    }
                    Rectangle {
                        anchors.verticalCenter: nameText.verticalCenter
                        visible: root.clanTag !== ""
                        height: 18
                        width: msgClanRow.implicitWidth + 10
                        radius: 6
                        color: Theme.surfaceContainerHighest
                        border.width: 1
                        border.color: Theme.alpha(Theme.outlineVariant, 0.5)

                        Row {
                            id: msgClanRow
                            anchors.centerIn: parent
                            spacing: 4
                            Image {
                                anchors.verticalCenter: parent.verticalCenter
                                visible: root.clanBadge !== ""
                                width: 12
                                height: 12
                                source: root.clanBadge
                                fillMode: Image.PreserveAspectFit
                                asynchronous: true
                            }
                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: root.clanTag
                                color: Theme.surfaceText
                                font.family: Theme.font
                                font.pixelSize: 10
                                font.weight: Font.Bold
                            }
                        }
                    }
                    Text {
                        id: stampText
                        anchors.baseline: nameText.baseline
                        text: root.stamp(root.time)
                        color: Theme.alpha(Theme.surfaceVariantText, 0.55)
                        font.family: Theme.font
                        font.pixelSize: 11
                        font.weight: Font.Medium

                        HoverHandler { id: stampHover }
                        RichTooltip {
                            anchor: stampText
                            placement: "above"
                            visible: stampHover.hovered
                            text: root.fullStamp(root.time)
                        }
                    }
                    Text {
                        anchors.baseline: nameText.baseline
                        visible: root.pinned
                        text: "Pinned"
                        color: Theme.primary
                        font.family: Theme.font
                        font.pixelSize: 10
                        font.weight: Font.Bold
                    }
                    Text {
                        anchors.baseline: nameText.baseline
                        visible: root.deleted
                        text: "(deleted)"
                        color: Theme.error
                        font.family: Theme.font
                        font.pixelSize: 11
                        font.weight: Font.Bold
                    }
                }

                Text {
                    width: parent.width
                    visible: text !== ""
                    text: root.content === "" ? "" : backend.renderMarkdown(
                        root.content,
                        String(Theme.surfaceContainerHighest),
                        String(Theme.primaryContainer),
                        String(Theme.primaryContainerText),
                        String(Theme.surfaceVariantText),
                        Theme.monoFont
                    ) + (root.edited ? " <font size=\"1\" color=\"" + Theme.alpha(Theme.surfaceVariantText, 0.55) + "\">(edited)</font>" : "")
                    textFormat: Text.RichText
                    color: root.deleted ? Theme.error : Theme.surfaceText
                    linkColor: Theme.primary
                    font.family: Theme.font
                    font.pixelSize: 15
                    lineHeight: 1.2
                    wrapMode: Text.Wrap
                    onLinkActivated: link => Qt.openUrlExternally(link)

                    HoverHandler {
                        enabled: parent.hoveredLink !== ""
                        cursorShape: Qt.PointingHandCursor
                    }
                }

                MessageForwarded {
                    width: parent.width
                    items: root.forwardedList
                }

                Row {
                    visible: root.stickerList.length > 0
                    spacing: 8
                    topPadding: 4

                    Repeater {
                        model: root.stickerList

                        delegate: Image {
                            required property var modelData
                            width: 140
                            height: 140
                            source: modelData.url
                            fillMode: Image.PreserveAspectFit
                            asynchronous: true
                        }
                    }
                }

                MessageEmbeds {
                    width: parent.width
                    items: root.embedList
                }

                MessageAttachments {
                    width: parent.width
                    files: root.files
                }

                MessageReactions {
                    width: parent.width
                    items: root.reactionList
                    canReact: root.messageId !== ""
                    onToggleReaction: (key, hasMe) => root.actionToggleReaction(root.messageId, key, hasMe)
                    onAddReaction: root.actionReact(root.messageId)
                }
            }
        }
    }

    MessageActionBar {
        visible: hover.hovered && root.messageId !== "" && !root.isSystem && !root.deleted
        anchors.right: parent.right
        anchors.rightMargin: 16
        y: root.dateOffset - 10
        z: 4
        mine: root.mine
        onReactClicked: root.actionReact(root.messageId)
        onReplyClicked: root.actionReply(root.messageId)
        onForwardClicked: root.copyText(root.content)
        onEditClicked: root.actionEdit(root.messageId, root.content)
        onCreateThreadClicked: backend.createThread(root.content.slice(0, 40) || "Thread", root.messageId)
        onMoreClicked: anchorItem => actions.openAt(anchorItem, anchorItem.width, anchorItem.height, true)
    }

    HoverHandler {
        id: hover
    }

    TextEdit {
        id: clipHelper
        visible: false
    }

    // Mobile long-press and desktop right-click open the same action menu.
    // z: -1 lets left-clicks on authorAvatar and nameText open ProfilePopup.
    MouseArea {
        anchors.fill: parent
        z: -1
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        propagateComposedEvents: true

        onClicked: mouse => {
            if (mouse.button === Qt.RightButton && root.messageId !== "" && !root.isSystem) {
                actions.openAt(this, mouse.x, mouse.y, false)
                mouse.accepted = true
            } else {
                mouse.accepted = false
            }
        }
        onPressAndHold: mouse => {
            if (root.messageId !== "" && !root.isSystem)
                actions.openAt(this, mouse.x, mouse.y, false)
        }
    }

    MessageContextMenu {
        id: actions
        enabled: root.messageId !== "" && !root.isSystem
        messageId: root.messageId
        authorId: root.authorId
        author: root.author
        avatar: root.avatar
        decoration: root.decoration
        content: root.content
        mine: root.mine
        files: root.files
        onEditRequested: root.actionEdit(root.messageId, root.content)
        onReplyRequested: root.actionReply(root.messageId)
        onDeleteRequested: root.actionDelete(root.messageId)
        onCopyTextRequested: value => root.copyText(value)
    }

    function fullStamp(ms) {
        return Qt.formatDateTime(new Date(ms), "dddd, MMMM d, yyyy 'at' h:mm AP")
    }

    function stamp(ms) {
        const date = new Date(ms)
        const today = new Date()
        const yesterday = new Date(today.getFullYear(), today.getMonth(), today.getDate() - 1)
        const time = Qt.formatTime(date, "h:mm AP")
        if (date.toDateString() === today.toDateString())
            return time
        if (date.toDateString() === yesterday.toDateString())
            return "Yesterday at " + time
        return Qt.formatDate(date, "M/d/yy") + ", " + time
    }

    function escapeHtml(text) {
        return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    }
}
