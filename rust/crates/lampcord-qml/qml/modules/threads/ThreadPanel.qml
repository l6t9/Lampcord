import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Item {
    id: root

    signal closeRequested()

    property bool showArchived: false

    readonly property var threads: {
        try { return JSON.parse(backend.threadsJson) } catch (e) { return [] }
    }
    readonly property var activeThreads: threads.filter(t => !t.archived)
    readonly property var archivedThreads: threads.filter(t => !!t.archived)

    function relativeActive(threadId) {
        if (!threadId || threadId === "")
            return "Last active recently"
        // Discord snowflake timestamp (ms since 2015-01-01; 2^22 = 4194304)
        const ms = Math.floor(Number(threadId) / 4194304) + 1420070400000
        const diffSec = Math.max(0, Math.floor((Date.now() - ms) / 1000))
        if (diffSec < 60) return "Last active just now"
        if (diffSec < 3600) return "Last active " + Math.floor(diffSec / 60) + "m ago"
        if (diffSec < 86400) return "Last active " + Math.floor(diffSec / 3600) + "h ago"
        return "Last active " + Math.floor(diffSec / 86400) + "d ago"
    }

    Connections {
        target: backend
        function onSelectedChannelIdChanged() { root.showArchived = false }
    }

    Column {
        anchors.fill: parent
        spacing: 0

        Item {
            width: parent.width
            height: 56

            Row {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 12
                spacing: 8

                IconButton {
                    anchors.verticalCenter: parent.verticalCenter
                    glyph: "Close"
                    iconSize: 20
                    tip: "Close Threads"
                    onClicked: root.closeRequested()
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Threads"
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 19
                    font.weight: Font.Bold
                }
            }
        }

        // Compact pill create bar; Create button stays hidden until text is typed.
        Row {
            width: parent.width - 24
            x: 12
            spacing: 8
            bottomPadding: 8

            Rectangle {
                width: parent.width - (createBtn.visible ? createBtn.width + 8 : 0)
                height: 36
                radius: 18
                color: Theme.surfaceContainerHigh

                TextInput {
                    id: newThreadName
                    anchors.fill: parent
                    anchors.leftMargin: 14
                    anchors.rightMargin: 14
                    verticalAlignment: Text.AlignVCenter
                    color: Theme.surfaceText
                    selectionColor: Theme.alpha(Theme.primary, 0.4)
                    selectedTextColor: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 13
                    clip: true
                    onAccepted: {
                        if (text.trim() !== "") {
                            backend.createThread(text.trim(), "")
                            text = ""
                        }
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: newThreadName.text === "" && !newThreadName.activeFocus
                        text: "New thread name…"
                        color: Theme.surfaceVariantText
                        font: newThreadName.font
                    }
                }
            }

            Rectangle {
                id: createBtn
                visible: newThreadName.text.trim() !== ""
                width: createLbl.implicitWidth + 24
                height: 36
                radius: 18
                color: createHover.hovered ? Theme.primaryContainer : Theme.primary

                Text {
                    id: createLbl
                    anchors.centerIn: parent
                    text: "Create"
                    color: createHover.hovered ? Theme.primaryContainerText : Theme.primaryText
                    font.family: Theme.font
                    font.pixelSize: 13
                    font.weight: Font.DemiBold
                }
                HoverHandler { id: createHover; cursorShape: Qt.PointingHandCursor }
                TapHandler {
                    onTapped: {
                        if (newThreadName.text.trim() !== "") {
                            backend.createThread(newThreadName.text.trim(), "")
                            newThreadName.text = ""
                        }
                    }
                }
            }
        }

        Flickable {
            width: parent.width
            height: parent.height - 100
            contentHeight: contentCol.implicitHeight + 16
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

            Column {
                id: contentCol
                width: parent.width
                spacing: 2

                Text {
                    visible: root.activeThreads.length > 0
                    leftPadding: 16
                    topPadding: 6
                    bottomPadding: 4
                    text: "Active"
                    color: Theme.primary
                    font.family: Theme.font
                    font.pixelSize: 12
                    font.weight: Font.Bold
                }

                Repeater {
                    model: root.activeThreads
                    delegate: threadRowComp
                }

                Item {
                    visible: !root.showArchived
                    width: parent.width
                    height: 52

                    Rectangle {
                        x: 16
                        y: 10
                        width: archLbl.implicitWidth + 32
                        height: 36
                        radius: 18
                        color: archHover.hovered ? Theme.surfaceContainerHighest : Theme.secondaryContainer

                        Text {
                            id: archLbl
                            anchors.centerIn: parent
                            text: "Show archived"
                            color: Theme.secondaryContainerText
                            font.family: Theme.font
                            font.pixelSize: 13
                            font.weight: Font.DemiBold
                        }
                        HoverHandler { id: archHover; cursorShape: Qt.PointingHandCursor }
                        TapHandler {
                            onTapped: {
                                root.showArchived = true
                                backend.fetchThreads()
                            }
                        }
                    }
                }

                Text {
                    visible: root.showArchived && root.archivedThreads.length > 0
                    leftPadding: 16
                    topPadding: 12
                    bottomPadding: 4
                    text: "Archived"
                    color: Theme.surfaceVariantText
                    font.family: Theme.font
                    font.pixelSize: 12
                    font.weight: Font.Bold
                }

                Repeater {
                    model: root.showArchived ? root.archivedThreads : []
                    delegate: threadRowComp
                }

                Text {
                    visible: root.activeThreads.length === 0 && root.showArchived && root.archivedThreads.length === 0
                    width: parent.width
                    topPadding: 32
                    horizontalAlignment: Text.AlignHCenter
                    text: "No threads in this channel yet"
                    color: Theme.surfaceVariantText
                    font.family: Theme.font
                    font.pixelSize: 13
                }
            }
        }
    }

    Component {
        id: threadRowComp
        Rectangle {
            required property var modelData
            width: contentCol.width - 16
            x: 8
            height: 54
            radius: 12
            color: tHover.hovered ? Theme.surfaceContainerHigh : "transparent"

            Row {
                anchors.fill: parent
                anchors.leftMargin: 10
                anchors.rightMargin: 10
                spacing: 12

                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "Rounded.Topic"
                    size: 20
                    color: Theme.surfaceVariantText
                }

                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 32
                    spacing: 2

                    Text {
                        width: parent.width
                        text: modelData.name
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.weight: Font.DemiBold
                        elide: Text.ElideRight
                    }
                    Text {
                        width: parent.width
                        text: modelData.archived ? "Archived" : root.relativeActive(modelData.id)
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 12
                        elide: Text.ElideRight
                    }
                }
            }

            HoverHandler { id: tHover; cursorShape: Qt.PointingHandCursor }
            TapHandler {
                onTapped: {
                    window.openChannel(modelData.id)
                    root.closeRequested()
                }
            }
        }
    }
}
