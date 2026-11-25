import QtQuick

// Default (non-bubble) MessageItem: 40px avatar, name + timestamp header,
// follow-ups from the same author folded under it, reply previews,
// attachments, and SystemMessage rows for calls, pins, joins and the like.
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

    // Message identity comes from the model role in chat. Pins and inbox rows
    // pass empty ids, so their action menu stays unavailable.
    required property string messageId
    required property string authorId
    property bool mine: false

    signal actionReply(string id)
    signal actionEdit(string id, string content)
    signal actionDelete(string id)

    readonly property bool isSystem: system !== ""
    readonly property var files: JSON.parse(attachments)

    implicitHeight: (isSystem ? systemRow.implicitHeight + 12 : body.implicitHeight + (grouped ? 1.5 : 6) + 1.5)

    Rectangle {
        anchors.fill: parent
        color: Theme.alpha(Theme.surfaceVariant, hover.hovered ? 0.12 : 0)
    }

    // SystemMessage: tinted icon in the avatar gutter, bold name, text, time.
    Row {
        id: systemRow
        visible: root.isSystem
        x: 8
        y: 6
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
            text: "<b><font color=\"" + Theme.surfaceText + "\">" + root.escapeHtml(root.author) + "</font></b> "
                  + root.system + "&nbsp;&nbsp;<font size=\"1\" color=\""
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
        y: root.grouped ? 1.5 : 6
        width: parent.width - 16
        implicitHeight: column.implicitHeight

        UserAvatar {
            visible: !root.grouped
            y: reply.visible ? reply.height : 0
            size: 40
            name: root.author
            source: root.avatar
            decoration: root.decoration
        }

        // Follow-ups show their time in the avatar gutter on hover.
        Text {
            visible: root.grouped && hover.hovered
            width: 40
            y: 3
            text: Qt.formatTime(new Date(root.time), "h:mm")
            color: Theme.alpha(Theme.surfaceVariantText, 0.5)
            font.family: Theme.font
            font.pixelSize: 10
            horizontalAlignment: Text.AlignHCenter
        }

        Column {
            id: column
            x: 48
            width: parent.width - 48
            spacing: 0

            // Reply preview, hooked to the avatar with a bent line.
            Item {
                id: reply
                visible: root.replyAuthor !== ""
                width: parent.width
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

            Row {
                visible: !root.grouped
                spacing: 8
                bottomPadding: 2

                Text {
                    id: nameText
                    text: root.author
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 14
                    font.weight: Font.Bold
                }
                Text {
                    anchors.baseline: nameText.baseline
                    text: root.stamp(root.time)
                    color: Theme.alpha(Theme.surfaceVariantText, 0.5)
                    font.family: Theme.font
                    font.pixelSize: 11
                    font.weight: Font.Medium
                }
            }

            Text {
                width: parent.width
                visible: text !== ""
                text: root.content === "" ? "" : root.render(root.content) + (root.edited ? " <font size=\"1\" color=\"" + Theme.alpha(Theme.surfaceVariantText, 0.5) + "\">(edited)</font>" : "")
                textFormat: Text.StyledText
                color: Theme.surfaceText
                linkColor: Theme.primary
                font.family: Theme.font
                font.pixelSize: 16
                lineHeight: 1.15
                wrapMode: Text.Wrap
                onLinkActivated: link => Qt.openUrlExternally(link)

                HoverHandler {
                    enabled: parent.hoveredLink !== ""
                    cursorShape: Qt.PointingHandCursor
                }
            }

            Repeater {
                model: root.files

                delegate: Item {
                    required property var modelData
                    width: column.width
                    height: (modelData.image ? image.height : card.height) + 4

                    // Images keep their aspect inside 400x300.
                    readonly property real fit: modelData.width > 0
                        ? Math.min(1, 400 / modelData.width, 300 / modelData.height) : 1

                    RoundedImage {
                        id: image
                        visible: modelData.image
                        y: 4
                        width: modelData.width > 0 ? modelData.width * parent.fit : 300
                        height: modelData.height > 0 ? modelData.height * parent.fit : 200
                        radius: 12
                        source: modelData.image ? modelData.preview : ""

                        Rectangle {
                            anchors.fill: parent
                            radius: 12
                            visible: !image.ready
                            color: Theme.surfaceContainerHigh
                        }
                        TapHandler { onTapped: Qt.openUrlExternally(modelData.url) }
                        HoverHandler { cursorShape: Qt.PointingHandCursor }
                    }

                    Rectangle {
                        id: card
                        visible: !modelData.image
                        y: 4
                        width: Math.min(column.width, 400)
                        height: 64
                        radius: 12
                        color: Theme.surfaceContainerHigh

                        Row {
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 12

                            Icon {
                                anchors.verticalCenter: parent.verticalCenter
                                name: "Description"
                                size: 32
                                color: Theme.primary
                            }
                            Column {
                                anchors.verticalCenter: parent.verticalCenter
                                width: parent.width - 32 - 40 - 24

                                Text {
                                    width: parent.width
                                    text: modelData.filename
                                    color: Theme.primary
                                    font.family: Theme.font
                                    font.pixelSize: 14
                                    elide: Text.ElideMiddle
                                }
                                Text {
                                    text: root.size(modelData.size)
                                    color: Theme.surfaceVariantText
                                    font.family: Theme.font
                                    font.pixelSize: 12
                                }
                            }
                            IconButton {
                                anchors.verticalCenter: parent.verticalCenter
                                glyph: "Download"
                                tip: "Download"
                                onClicked: Qt.openUrlExternally(modelData.url)
                            }
                        }
                    }
                }
            }
        }
    }

    HoverHandler {
        id: hover
    }

    // Mobile long-press and desktop right-click open the same action menu.
    // Left clicks propagate so links in the message stay tappable.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        propagateComposedEvents: true

        onClicked: mouse => {
            if (mouse.button === Qt.RightButton) {
                actions.open()
                mouse.accepted = true
            } else {
                mouse.accepted = false
            }
        }
        onPressAndHold: actions.open()
    }

    LampMenu {
        id: actions
        x: Math.max(4, root.width - width - 12)
        y: 4
        enabled: root.messageId !== "" && !root.isSystem

        LampMenuItem {
            text: "Reply"
            onTriggered: {
                root.actionReply(root.messageId)
                actions.close()
            }
        }
        LampMenuItem {
            visible: root.mine
            text: "Edit"
            onTriggered: {
                root.actionEdit(root.messageId, root.content)
                actions.close()
            }
        }
        LampMenuItem {
            visible: root.mine
            text: "Delete"
            danger: true
            onTriggered: {
                root.actionDelete(root.messageId)
                actions.close()
            }
        }
    }

    // DateTimeUtils.formatTimestamp from the Compose client.
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

    function size(bytes) {
        if (bytes < 1024)
            return bytes + " B"
        if (bytes < 1024 * 1024)
            return (bytes / 1024).toFixed(1) + " KB"
        return (bytes / 1024 / 1024).toFixed(1) + " MB"
    }

    function escapeHtml(text) {
        return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    }

    // Just enough Discord markdown for plain chat: links, bold, italics,
    // strikethrough and inline code. Everything is escaped first.
    function render(text) {
        let out = escapeHtml(text)
        const codes = []
        out = out.replace(/`([^`\n]+)`/g, (m, code) => {
            codes.push(code)
            return "\u0000" + (codes.length - 1) + "\u0000"
        })
        out = out.replace(/(https?:\/\/[^\s<]+[^\s<.,:;"')\]])/g, "<a href=\"$1\">$1</a>")
        out = out.replace(/\*\*(.+?)\*\*/g, "<b>$1</b>")
        out = out.replace(/__(.+?)__/g, "<u>$1</u>")
        out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, "$1<i>$2</i>")
        out = out.replace(/(^|\W)_([^_\n]+)_(?=\W|$)/g, "$1<i>$2</i>")
        out = out.replace(/~~(.+?)~~/g, "<s>$1</s>")
        out = out.replace(/\u0000(\d+)\u0000/g, (m, n) =>
            "<code style=\"background-color:" + Theme.surfaceContainerHighest + "\">&nbsp;" + codes[n] + "&nbsp;</code>")
        return out.replace(/\n/g, "<br>")
    }
}
