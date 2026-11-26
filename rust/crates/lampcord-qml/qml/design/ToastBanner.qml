import QtQuick
import Lampcord

Rectangle {
    id: root

    property string title: ""
    property string body: ""
    property string channelId: ""
    property bool shown: false

    width: Math.min(340, parent ? parent.width - 32 : 340)
    height: shown ? col.implicitHeight + 20 : 0
    radius: 16
    color: Theme.surfaceContainerHighest
    border.width: shown ? 1 : 0
    border.color: Theme.outlineVariant
    opacity: shown ? 1 : 0
    visible: opacity > 0
    clip: true

    Behavior on opacity { NumberAnimation { duration: Theme.quick } }
    Behavior on height { NumberAnimation { duration: Theme.quick; easing.type: Easing.OutCubic } }

    function show(t, b, cid) {
        if (!Theme.prefs.inAppToasts)
            return
        title = t
        body = b
        channelId = cid
        shown = true
        hideTimer.restart()
    }

    Timer {
        id: hideTimer
        interval: 4500
        onTriggered: root.shown = false
    }

    Column {
        id: col
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 12
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        Text {
            width: parent.width
            text: root.title
            color: Theme.primary
            font.family: Theme.font
            font.pixelSize: 13
            font.weight: Font.Bold
            elide: Text.ElideRight
        }
        Text {
            width: parent.width
            text: root.body
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 12
            maximumLineCount: 2
            wrapMode: Text.Wrap
            elide: Text.ElideRight
        }
    }

    HoverHandler { cursorShape: Qt.PointingHandCursor }
    TapHandler {
        onTapped: {
            if (root.channelId !== "")
                window.openChannel(root.channelId)
            root.shown = false
        }
    }
}
