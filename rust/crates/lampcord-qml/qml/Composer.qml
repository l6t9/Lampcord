import QtQuick
import QtQuick.Controls.Basic

// ChatInputBar: a rounded field on surfaceContainerHigh and the round
// primary send button. Enter sends, Shift+Enter breaks the line.
Item {
    id: root

    property string placeholder
    property real lastTyping: 0

    implicitHeight: row.implicitHeight + 2 + 8

    function reset() {
        input.clear()
        lastTyping = 0
    }

    function send() {
        const text = input.text
        if (text.trim() === "")
            return
        backend.sendMessage(text)
        input.clear()
        lastTyping = 0
    }

    Row {
        id: row
        x: 8
        y: 2
        width: parent.width - 16
        spacing: 4

        Rectangle {
            id: field
            width: parent.width - send.width - parent.spacing
            height: Math.max(46, Math.min(input.implicitHeight, 240))
            anchors.bottom: parent.bottom
            radius: 16
            color: Theme.surfaceContainerHigh

            ScrollView {
                anchors.fill: parent
                anchors.leftMargin: 12
                anchors.rightMargin: 8

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
        }

        IconButton {
            id: send
            anchors.bottom: parent.bottom
            width: 46
            height: 46
            glyph: "send"
            iconSize: 24
            iconColor: Theme.primaryText
            container: Theme.primary
            enabled: input.text.trim() !== ""
            opacity: enabled ? 1 : 0.6
            tip: "Send"
            onClicked: root.send()
        }
    }
}
