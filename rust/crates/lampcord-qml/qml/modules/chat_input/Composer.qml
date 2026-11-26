import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import QtQuick.Effects
import Lampcord

// ChatInputBar on desktop: the round add button, the rounded field with
// the emoji button inside it, and the send button, which only slides in
// once there is something to send. Picked files queue above the field.
// Enter sends, Shift+Enter breaks the line.
Item {
    id: root

    property string placeholder
    property bool floating: false
    property real lastTyping: 0
    property var files: []
    property string replyId: ""
    property string replyAuthor: ""
    property string replyPreview: ""
    property string editId: ""

    readonly property bool canSend: input.text.trim() !== "" || files.length > 0
    readonly property var slashCommands: [
        { name: "/shrug", desc: "Append ¯\\_(ツ)_/¯ to your message", insert: "¯\\_(ツ)_/¯" },
        { name: "/tableflip", desc: "Append (╯°□°)╯︵ ┻━┻", insert: "(╯°□°)╯︵ ┻━┻" },
        { name: "/unflip", desc: "Append ┬─┬ノ( º _ ºノ)", insert: "┬─┬ノ( º _ ºノ)" },
        { name: "/me", desc: "Send italicized action text", insert: "*...*" },
        { name: "/spoiler", desc: "Wrap text in a spoiler tag", insert: "||...||" },
        { name: "/code", desc: "Insert a fenced code block", insert: "```\n\n```" }
    ]
    readonly property var matchingCommands: {
        const t = input.text
        if (!t.startsWith("/") || t.includes(" "))
            return []
        const q = t.toLowerCase()
        return slashCommands.filter(cmd => cmd.name.startsWith(q))
    }

    implicitHeight: column.implicitHeight

    function clearContext() {
        replyId = ""
        replyAuthor = ""
        replyPreview = ""
        editId = ""
    }

    function reset() {
        input.clear()
        files = []
        lastTyping = 0
        clearContext()
        emojiPicker.close()
    }

    function replyTo(id, author, content) {
        clearContext()
        replyId = id
        replyAuthor = author
        replyPreview = content.replace(/\s+/g, " ").slice(0, 120)
        input.forceActiveFocus()
    }

    function editMessage(id, content) {
        clearContext()
        editId = id
        input.text = content
        input.cursorPosition = content.length
        input.forceActiveFocus()
    }

    function insertMention(userId) {
        const prefix = input.text.length > 0 && !input.text.endsWith(" ") ? " " : ""
        input.text = input.text + prefix + "<@" + userId + "> "
        input.cursorPosition = input.text.length
        input.forceActiveFocus()
    }

    function handleBack() {
        if (emojiPicker.opened) {
            emojiPicker.close()
            return true
        }
        if (replyId !== "" || editId !== "") {
            clearContext()
            return true
        }
        return false
    }

    function send() {
        if (!canSend)
            return
        let text = input.text
        const trimmed = text.trim()
        if (trimmed === "/shrug") text = "¯\\_(ツ)_/¯"
        else if (trimmed.startsWith("/shrug ")) text = trimmed.slice(7) + " ¯\\_(ツ)_/¯"
        else if (trimmed === "/tableflip") text = "(╯°□°)╯︵ ┻━┻"
        else if (trimmed.startsWith("/tableflip ")) text = trimmed.slice(11) + " (╯°□°)╯︵ ┻━┻"
        else if (trimmed === "/unflip") text = "┬─┬ノ( º _ ºノ)"
        else if (trimmed.startsWith("/me ")) text = "*" + trimmed.slice(4) + "*"
        else if (trimmed.startsWith("/spoiler ")) text = "||" + trimmed.slice(9) + "||"

        if (editId !== "")
            backend.editMessage(editId, text)
        else if (replyId !== "" && files.length === 0)
            backend.sendReply(text, replyId)
        else if (files.length > 0)
            backend.sendFiles(text, JSON.stringify(files))
        else
            backend.sendMessage(text)
        input.clear()
        files = []
        lastTyping = 0
        clearContext()
    }

    FileDialog {
        id: picker
        title: "Upload a file"
        fileMode: FileDialog.OpenFiles
        onAccepted: root.files = root.files.concat(selectedFiles.map(url => url.toString()))
    }

    // Phoenix floating inputs sit over the message list with a short scrim,
    // instead of pushing the list up with a solid app bar.
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: row.top
        height: 56
        visible: root.floating
        z: -1
        gradient: Gradient {
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 1; color: Theme.surfaceContainerLow }
        }
    }

    Column {
        id: column
        width: parent.width

        // Slash command autocomplete popup
        Rectangle {
            visible: root.matchingCommands.length > 0
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: 8
            anchors.rightMargin: 8
            height: visible ? slashCol.implicitHeight + 12 : 0
            radius: 14
            color: Theme.surfaceContainerHigh
            border.width: 1
            border.color: Theme.outlineVariant

            Column {
                id: slashCol
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: 6
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2

                Repeater {
                    model: root.matchingCommands

                    delegate: Rectangle {
                        required property var modelData
                        width: parent.width
                        height: 36
                        radius: 8
                        color: cmdHover.hovered ? Theme.secondaryContainer : "transparent"

                        Row {
                            anchors.fill: parent
                            anchors.leftMargin: 10
                            anchors.rightMargin: 10
                            spacing: 10

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: modelData.name
                                color: Theme.primary
                                font.family: Theme.monoFont
                                font.pixelSize: 13
                                font.weight: Font.Bold
                            }
                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: modelData.desc
                                color: Theme.surfaceVariantText
                                font.family: Theme.font
                                font.pixelSize: 12
                                elide: Text.ElideRight
                            }
                        }

                        HoverHandler { id: cmdHover; cursorShape: Qt.PointingHandCursor }
                        TapHandler {
                            onTapped: {
                                input.text = modelData.name + " "
                                input.cursorPosition = input.text.length
                                input.forceActiveFocus()
                            }
                        }
                    }
                }
            }
        }

        // Reply/edit banner, like ChatInputBar's reply banner.
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: 8
            anchors.rightMargin: 8
            height: visible ? 52 : 0
            radius: root.floating ? 12 : 10
            color: Theme.surfaceContainerHigh
            visible: root.replyId !== "" || root.editId !== ""

            Rectangle {
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: 3
                radius: 1
                color: Theme.primary
            }

            Row {
                anchors.fill: parent
                anchors.leftMargin: 12
                anchors.rightMargin: 4
                spacing: 8

                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 44
                    spacing: 1

                    Text {
                        width: parent.width
                        text: root.editId !== "" ? "Editing message" : "Replying to " + root.replyAuthor
                        color: Theme.primary
                        font.family: Theme.font
                        font.pixelSize: 12
                        font.weight: Font.DemiBold
                        elide: Text.ElideRight
                    }
                    Text {
                        width: parent.width
                        text: root.editId !== "" ? "Save your changes below" : root.replyPreview
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 11
                        elide: Text.ElideRight
                    }
                }

                IconButton {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 36
                    height: 36
                    glyph: "Rounded.Close"
                    iconSize: 18
                    tip: "Cancel"
                    onClicked: root.clearContext()
                }
            }
        }

        // Queued uploads.
        Row {
            visible: root.files.length > 0
            padding: 8
            spacing: 8
            height: visible ? 120 : 0

            Repeater {
                model: root.files

                delegate: Rectangle {
                    required property string modelData
                    required property int index
                    readonly property string name: decodeURIComponent(modelData.split("/").pop())
                    readonly property bool image: /\.(png|jpe?g|gif|webp)$/i.test(name)
                    width: 100
                    height: 100
                    radius: 8
                    color: Theme.surfaceContainerHigh

                    RoundedImage {
                        anchors.fill: parent
                        radius: 8
                        visible: parent.image
                        source: parent.image ? parent.modelData : ""
                    }
                    Text {
                        anchors.centerIn: parent
                        width: parent.width - 8
                        visible: !parent.image
                        text: parent.name
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 11
                        horizontalAlignment: Text.AlignHCenter
                        wrapMode: Text.WrapAnywhere
                        maximumLineCount: 2
                        elide: Text.ElideRight
                    }
                    Rectangle {
                        anchors.top: parent.top
                        anchors.right: parent.right
                        width: 24
                        height: 24
                        radius: 12
                        color: Qt.rgba(0, 0, 0, 0.4)

                        Icon {
                            anchors.centerIn: parent
                            name: "Close"
                            size: 16
                            color: "white"
                        }
                        TapHandler {
                            onTapped: root.files = root.files.filter((_, i) => i !== index)
                        }
                        HoverHandler { cursorShape: Qt.PointingHandCursor }
                    }
                }
            }
        }

        Row {
            id: row
            x: root.floating ? 16 : 8
            topPadding: 2
            bottomPadding: (root.floating ? 16 : 8) + (root.floating ? window.navBarInset : 0)
            width: parent.width - (root.floating ? 32 : 16)
            spacing: 0

            IconButton {
                id: add
                anchors.bottom: parent.bottom
                anchors.bottomMargin: root.floating ? 0 : 8
                width: root.floating ? 40 : 46
                height: root.floating ? 40 : 46
                iconSize: root.floating ? 20 : 24
                glyph: "Add"
                iconColor: Theme.surfaceVariantText
                container: Theme.surfaceContainerHigh
                tip: "Upload a file"
                onClicked: picker.open()
            }

            Item { width: 4; height: 1 }

            Item {
                id: field
                width: parent.width - add.width - 4 - sendSlot.width
                height: root.floating
                    ? Math.max(54, Math.min(input.implicitHeight + 20, 200))
                    : Math.max(46, Math.min(input.implicitHeight, 240))
                anchors.bottom: parent.bottom
                anchors.bottomMargin: root.floating ? 0 : 8

                RectangularShadow {
                    anchors.fill: fieldSurface
                    radius: fieldSurface.radius
                    blur: root.floating ? 4 : 12
                    offset.y: root.floating ? -1 : 3
                    color: Qt.rgba(0, 0, 0, 0.24)
                    visible: root.floating
                }

                Rectangle {
                    id: fieldSurface
                    anchors.fill: parent
                    radius: 16
                    color: Theme.surfaceContainerHigh
                    border.width: root.floating ? 1 : 0
                    border.color: Theme.alpha(Theme.outlineVariant, 0.5)
                }

                ScrollView {
                    anchors.fill: parent
                    anchors.leftMargin: 12
                    anchors.rightMargin: root.floating ? 40 : 44

                    TextArea {
                        id: input
                        topPadding: root.floating ? 10 : 13
                        bottomPadding: root.floating ? 10 : 13
                        leftPadding: 0
                        rightPadding: 0
                        wrapMode: TextEdit.Wrap
                        color: Theme.surfaceText
                        selectionColor: Theme.alpha(Theme.primary, 0.4)
                        selectedTextColor: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 16
                        background: null
                        focus: true

                        Text {
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            visible: input.text.length === 0
                            text: root.placeholder
                            color: Theme.alpha(Theme.surfaceVariantText, 0.6)
                            font: input.font
                            elide: Text.ElideRight
                            maximumLineCount: 1
                        }

                        Keys.onPressed: event => {
                            if (event.key !== Qt.Key_Return && event.key !== Qt.Key_Enter)
                                return
                            if (event.modifiers & Qt.ShiftModifier)
                                return
                            event.accepted = true
                            root.send()
                        }

                        onTextChanged: {
                            const now = Date.now()
                            if (text !== "" && now - root.lastTyping > 8000) {
                                root.lastTyping = now
                                backend.sendTyping()
                            }
                        }
                    }
                }

                IconButton {
                    id: emoji
                    anchors.right: parent.right
                    anchors.rightMargin: 4
                    anchors.bottom: parent.bottom
                    anchors.bottomMargin: root.floating ? 11 : 5
                    width: root.floating ? 32 : 36
                    height: root.floating ? 32 : 36
                    iconSize: root.floating ? 18 : 24
                    glyph: "SentimentSatisfied"
                    iconColor: emojiPicker.opened ? Theme.primary : Theme.surfaceVariantText
                    tip: "Emoji"
                    onClicked: emojiPicker.opened ? emojiPicker.close() : emojiPicker.open()

                    EmojiPicker {
                        id: emojiPicker
                        x: emoji.width - width
                        y: -height - 16
                        transformOrigin: Item.BottomRight
                        onPicked: emojiText => {
                            input.insert(input.cursorPosition, emojiText)
                            input.forceActiveFocus()
                        }
                    }
                }
            }

            // Send slides in from zero width (expand + fade + scale from 0.8).
            Item {
                id: sendSlot
                anchors.bottom: parent.bottom
                anchors.bottomMargin: root.floating ? 0 : 8
                width: root.canSend ? (root.floating ? 44 : 50) : 0
                height: root.floating ? 54 : 46
                clip: true

                Behavior on width {
                    NumberAnimation {
                        duration: root.floating ? 100 : Theme.medium
                        easing.type: Easing.OutCubic
                    }
                }

                IconButton {
                    x: 4
                    anchors.verticalCenter: parent.verticalCenter
                    width: root.floating ? 36 : 46
                    height: root.floating ? 36 : 46
                    iconSize: root.floating ? 20 : 24
                    glyph: "Rounded.Send"
                    iconColor: Theme.primaryText
                    container: Theme.primary
                    tip: "Send"
                    opacity: root.canSend ? 1 : 0
                    scale: root.canSend ? 1 : 0.8
                    onClicked: root.send()

                    Behavior on opacity { NumberAnimation { duration: Theme.medium } }
                    Behavior on scale { NumberAnimation { duration: Theme.medium; easing.type: Easing.OutCubic } }
                }
            }
        }
    }
}
