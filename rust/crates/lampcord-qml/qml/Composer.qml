import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs

// ChatInputBar on desktop: the round add button, the rounded field with
// the emoji button inside it, and the send button, which only slides in
// once there is something to send. Picked files queue above the field.
// Enter sends, Shift+Enter breaks the line.
Item {
    id: root

    property string placeholder
    property real lastTyping: 0
    property var files: []

    readonly property bool canSend: input.text.trim() !== "" || files.length > 0

    implicitHeight: column.implicitHeight

    function reset() {
        input.clear()
        files = []
        lastTyping = 0
        emojiPicker.close()
    }

    function send() {
        if (!canSend)
            return
        if (files.length > 0)
            backend.sendFiles(input.text, JSON.stringify(files))
        else
            backend.sendMessage(input.text)
        input.clear()
        files = []
        lastTyping = 0
    }

    FileDialog {
        id: picker
        title: "Upload a file"
        fileMode: FileDialog.OpenFiles
        onAccepted: root.files = root.files.concat(selectedFiles.map(url => url.toString()))
    }

    Column {
        id: column
        width: parent.width

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
            x: 8
            topPadding: 2
            bottomPadding: 8
            width: parent.width - 16
            spacing: 0

            IconButton {
                id: add
                anchors.bottom: parent.bottom
                anchors.bottomMargin: 8
                width: 46
                height: 46
                glyph: "Add"
                iconColor: Theme.surfaceVariantText
                container: Theme.surfaceContainerHigh
                tip: "Upload a file"
                onClicked: picker.open()
            }

            Item { width: 4; height: 1 }

            Rectangle {
                id: field
                width: parent.width - add.width - 4 - sendSlot.width
                height: Math.max(46, Math.min(input.implicitHeight, 240))
                anchors.bottom: parent.bottom
                anchors.bottomMargin: 8
                radius: 16
                color: Theme.surfaceContainerHigh

                ScrollView {
                    anchors.fill: parent
                    anchors.leftMargin: 12
                    anchors.rightMargin: 44

                    TextArea {
                        id: input
                        topPadding: 13
                        bottomPadding: 13
                        leftPadding: 0
                        rightPadding: 0
                        wrapMode: TextEdit.Wrap
                        color: Theme.surfaceText
                        selectionColor: Theme.alpha(Theme.primary, 0.4)
                        selectedTextColor: Theme.surfaceText
                        placeholderText: root.placeholder
                        placeholderTextColor: Theme.alpha(Theme.surfaceVariantText, 0.6)
                        font.family: Theme.font
                        font.pixelSize: 16
                        background: null
                        focus: true

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
                    anchors.bottomMargin: 5
                    width: 36
                    height: 36
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
                anchors.bottomMargin: 8
                width: root.canSend ? 50 : 0
                height: 46
                clip: true

                Behavior on width { NumberAnimation { duration: Theme.medium; easing.type: Easing.OutCubic } }

                IconButton {
                    x: 4
                    width: 46
                    height: 46
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
