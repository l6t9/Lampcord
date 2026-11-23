import QtQuick

// Default (non-bubble) MessageItem: 40px avatar, name + timestamp header,
// and follow-ups from the same author folded under it.
Item {
    id: root

    required property string author
    required property string avatar
    required property string content
    required property real time
    required property bool edited
    required property bool grouped

    implicitHeight: body.implicitHeight + (grouped ? 1.5 : 6) + 1.5

    Rectangle {
        anchors.fill: parent
        color: Theme.alpha(Theme.surfaceVariant, hover.hovered ? 0.12 : 0)
    }

    Item {
        id: body
        x: 8
        y: root.grouped ? 1.5 : 6
        width: parent.width - 16
        implicitHeight: Math.max(column.implicitHeight, root.grouped ? 0 : 40)

        Avatar {
            visible: !root.grouped
            size: 40
            name: root.author
            source: root.avatar
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

            Row {
                visible: !root.grouped
                spacing: 8
                bottomPadding: 2

                Text {
                    text: root.author
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 14
                    font.weight: Font.Bold
                }
                Text {
                    anchors.baseline: parent.children[0].baseline
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
                text: root.render(root.content) + (root.edited ? " <font size=\"1\" color=\"" + Theme.alpha(Theme.surfaceVariantText, 0.5) + "\">(edited)</font>" : "")
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
        }
    }

    HoverHandler {
        id: hover
    }

    function stamp(ms) {
        const date = new Date(ms)
        const today = new Date()
        const yesterday = new Date(today.getFullYear(), today.getMonth(), today.getDate() - 1)
        const time = Qt.formatTime(date, "h:mm AP")
        if (date.toDateString() === today.toDateString())
            return "Today at " + time
        if (date.toDateString() === yesterday.toDateString())
            return "Yesterday at " + time
        return Qt.formatDate(date, "MM/dd/yyyy") + " " + time
    }

    // Just enough Discord markdown for plain chat: links, bold, italics,
    // strikethrough and inline code. Everything is escaped first.
    function render(text) {
        let out = text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
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
