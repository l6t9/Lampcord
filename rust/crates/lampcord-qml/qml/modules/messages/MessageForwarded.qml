import QtQuick
import Lampcord

Column {
    id: root
    property var items: []
    visible: items.length > 0
    spacing: 4

    Repeater {
        model: root.items

        delegate: Item {
            required property var modelData
            width: root.width
            height: fwdCol.implicitHeight + 12

            Rectangle {
                x: 0
                y: 4
                width: 3
                height: parent.height - 8
                radius: 1.5
                color: Theme.outline
            }

            Column {
                id: fwdCol
                x: 12
                y: 4
                width: parent.width - 12
                spacing: 4

                Row {
                    spacing: 5
                    Icon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: "Forward"
                        size: 14
                        color: Theme.surfaceVariantText
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Forwarded"
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 12
                        font.italic: true
                    }
                }

                Text {
                    width: parent.width
                    visible: !!(modelData.content && modelData.content !== "")
                    text: modelData.content ? backend.renderMarkdown(
                        modelData.content,
                        String(Theme.surfaceContainerHighest),
                        String(Theme.primaryContainer),
                        String(Theme.primaryContainerText),
                        String(Theme.surfaceVariantText),
                        Theme.monoFont
                    ) : ""
                    textFormat: Text.RichText
                    color: Theme.surfaceText
                    linkColor: Theme.primary
                    font.family: Theme.font
                    font.pixelSize: 14
                    lineHeight: 1.2
                    wrapMode: Text.Wrap
                    onLinkActivated: link => Qt.openUrlExternally(link)
                }

                Text {
                    visible: !!(modelData.timestamp && modelData.timestamp !== "")
                    text: modelData.timestamp ? Qt.formatDateTime(new Date(modelData.timestamp), "M/d/yy, h:mm AP") : ""
                    color: Theme.alpha(Theme.surfaceVariantText, 0.65)
                    font.family: Theme.font
                    font.pixelSize: 11
                }
            }
        }
    }
}
