import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Popup {
    id: root

    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    padding: 0
    width: 440
    height: Math.min(540, parent ? parent.height - 80 : 540)

    property string scope: backend.selectedGuildId !== "" ? "guild" : "channel"
    readonly property var results: {
        try { return JSON.parse(backend.searchResultsJson) } catch (e) { return [] }
    }

    onOpened: {
        queryInput.forceActiveFocus()
    }

    background: Rectangle {
        radius: 20
        color: Theme.surfaceContainer
        border.width: 1
        border.color: Theme.outlineVariant
        clip: true
    }

    contentItem: Column {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            width: parent.width
            height: 98
            color: Theme.surfaceContainerHigh
            topLeftRadius: 20
            topRightRadius: 20

            Column {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 8

                Row {
                    width: parent.width
                    spacing: 8

                    Rectangle {
                        width: parent.width - searchBtn.width - 8
                        height: 38
                        radius: 19
                        color: Theme.surface

                        Row {
                            anchors.fill: parent
                            anchors.leftMargin: 12
                            anchors.rightMargin: 12
                            spacing: 8

                            Icon {
                                anchors.verticalCenter: parent.verticalCenter
                                name: "Search"
                                size: 18
                                color: Theme.surfaceVariantText
                            }
                            TextInput {
                                id: queryInput
                                anchors.verticalCenter: parent.verticalCenter
                                width: parent.width - 28
                                color: Theme.surfaceText
                                font.family: Theme.font
                                font.pixelSize: 14
                                clip: true
                                onAccepted: searchBtn.clicked()

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    visible: queryInput.text === "" && !queryInput.activeFocus
                                    text: "Search messages…"
                                    color: Theme.surfaceVariantText
                                    font: queryInput.font
                                }
                            }
                        }
                    }

                    FilledButton {
                        id: searchBtn
                        implicitHeight: 38
                        text: "Search"
                        enabled: queryInput.text.trim() !== ""
                        onClicked: backend.searchMessages(queryInput.text.trim(), root.scope)
                    }
                }

                Row {
                    spacing: 8

                    Repeater {
                        model: backend.selectedGuildId !== ""
                            ? [{ key: "guild", label: "Current Server" }, { key: "channel", label: "This Channel" }]
                            : [{ key: "channel", label: "This Channel" }]

                        delegate: Rectangle {
                            required property var modelData
                            readonly property bool active: root.scope === modelData.key
                            width: scopeText.implicitWidth + 20
                            height: 26
                            radius: 13
                            color: active ? Theme.primaryContainer : Theme.surface
                            Text {
                                id: scopeText
                                anchors.centerIn: parent
                                text: modelData.label
                                color: active ? Theme.primaryContainerText : Theme.surfaceVariantText
                                font.family: Theme.font
                                font.pixelSize: 12
                                font.weight: active ? Font.DemiBold : Font.Normal
                            }
                            HoverHandler { cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: root.scope = modelData.key }
                        }
                    }
                }
            }
        }

        ListView {
            id: resultList
            width: parent.width
            height: parent.height - 98
            model: root.results
            clip: true
            topMargin: 8
            bottomMargin: 8
            spacing: 6
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

            delegate: Rectangle {
                required property var modelData
                width: resultList.width - 16
                x: 8
                height: resCol.implicitHeight + 16
                radius: 12
                color: resHover.hovered ? Theme.surfaceContainerHigh : Theme.surface

                Column {
                    id: resCol
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.margins: 10
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 4

                    Row {
                        width: parent.width
                        spacing: 8

                        Avatar {
                            anchors.verticalCenter: parent.verticalCenter
                            size: 22
                            name: modelData.author
                            source: modelData.avatar
                        }
                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: modelData.author
                            color: Theme.surfaceText
                            font.family: Theme.font
                            font.pixelSize: 13
                            font.weight: Font.Bold
                        }
                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: modelData.dateLabel || ""
                            color: Theme.surfaceVariantText
                            font.family: Theme.font
                            font.pixelSize: 11
                        }
                    }

                    Text {
                        width: parent.width
                        text: modelData.content
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 13
                        wrapMode: Text.Wrap
                        maximumLineCount: 4
                        elide: Text.ElideRight
                    }
                }

                HoverHandler { id: resHover; cursorShape: Qt.PointingHandCursor }
                TapHandler {
                    onTapped: {
                        if (modelData.channelId && modelData.channelId !== "")
                            window.openChannel(modelData.channelId)
                        root.close()
                    }
                }
            }

            Text {
                anchors.centerIn: parent
                visible: root.results.length === 0
                text: queryInput.text.trim() === "" ? "Enter a query to search server or channel history" : "No messages found"
                color: Theme.surfaceVariantText
                font.family: Theme.font
                font.pixelSize: 13
            }
        }
    }
}
