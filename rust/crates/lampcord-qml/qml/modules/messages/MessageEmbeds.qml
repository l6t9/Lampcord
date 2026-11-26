import QtQuick
import Lampcord

Column {
    id: root
    property var items: []
    visible: items.length > 0
    spacing: 4

    Repeater {
        model: root.items

        delegate: Rectangle {
            required property var modelData
            readonly property bool hasContent: !!(modelData.author || modelData.title || modelData.description || modelData.image || modelData.footer || (modelData.fields && modelData.fields.length > 0))
            visible: hasContent
            width: Math.min(root.width, 460)
            height: visible ? embedCol.implicitHeight + 20 : 0
            radius: 12
            color: Theme.surfaceContainerHigh
            clip: true

            Rectangle {
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: 4
                color: modelData.color ? ("#" + modelData.color.toString(16).padStart(6, "0")) : Theme.primary
            }

            Column {
                id: embedCol
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.leftMargin: 16
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                spacing: 6

                Row {
                    visible: !!(modelData.author && modelData.author.name)
                    spacing: 6
                    width: parent.width

                    RoundedImage {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: !!(modelData.author && (modelData.author.proxy_icon_url || modelData.author.icon_url))
                        width: 20
                        height: 20
                        radius: 10
                        source: modelData.author ? (modelData.author.proxy_icon_url || modelData.author.icon_url || "") : ""
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - 28
                        text: modelData.author ? (modelData.author.name || "") : ""
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 13
                        font.weight: Font.Bold
                        elide: Text.ElideRight
                    }
                }

                Text {
                    visible: !!(modelData.title && modelData.title !== "")
                    width: parent.width
                    text: modelData.title || ""
                    color: modelData.url ? Theme.primary : Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 14
                    font.weight: Font.Bold
                    wrapMode: Text.Wrap
                    HoverHandler {
                        enabled: !!modelData.url
                        cursorShape: Qt.PointingHandCursor
                    }
                    TapHandler {
                        enabled: !!modelData.url
                        onTapped: Qt.openUrlExternally(modelData.url)
                    }
                }

                Text {
                    visible: !!(modelData.description && modelData.description !== "")
                    width: parent.width
                    textFormat: Text.RichText
                    text: modelData.description ? backend.renderMarkdown(
                        modelData.description,
                        String(Theme.surfaceContainerHighest),
                        String(Theme.primaryContainer),
                        String(Theme.primaryContainerText),
                        String(Theme.surfaceVariantText),
                        Theme.monoFont
                    ) : ""
                    color: Theme.surfaceText
                    linkColor: Theme.primary
                    font.family: Theme.font
                    font.pixelSize: 13
                    lineHeight: 1.2
                    wrapMode: Text.Wrap
                    onLinkActivated: link => Qt.openUrlExternally(link)
                }

                RoundedImage {
                    visible: !!(modelData.image && (modelData.image.proxy_url || modelData.image.url))
                    width: Math.min(parent.width, 380)
                    height: visible ? 200 : 0
                    radius: 8
                    source: modelData.image ? (modelData.image.proxy_url || modelData.image.url || "") : ""
                    TapHandler {
                        onTapped: {
                            const u = modelData.image ? (modelData.image.url || modelData.image.proxy_url || "") : ""
                            if (u !== "" && window.openImage) window.openImage(u, "")
                        }
                    }
                    HoverHandler { cursorShape: Qt.PointingHandCursor }
                }

                Row {
                    visible: !!(modelData.footer && modelData.footer.text)
                    spacing: 6
                    width: parent.width

                    RoundedImage {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: !!(modelData.footer && (modelData.footer.proxy_icon_url || modelData.footer.icon_url))
                        width: 16
                        height: 16
                        radius: 8
                        source: modelData.footer ? (modelData.footer.proxy_icon_url || modelData.footer.icon_url || "") : ""
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - 24
                        text: modelData.footer ? (modelData.footer.text || "") : ""
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 11
                        elide: Text.ElideRight
                    }
                }
            }
        }
    }
}
