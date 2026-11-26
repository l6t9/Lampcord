import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// Mobile replacement for the Compose DM side panel: a searchable list of
// open conversations. Selecting one returns to the chat surface.
Item {
    id: root

    signal openServers()
    signal openedDm()

    property string query: ""

    function handleBack() {
        if (root.query !== "") {
            root.query = ""
            return true
        }
        return false
    }

    readonly property var filtered: window.dms.filter(dm =>
        root.query === "" || dm.name.toLowerCase().indexOf(root.query.toLowerCase()) !== -1)

    Rectangle {
        anchors.fill: parent
        color: Theme.surface
    }

    Rectangle {
        id: searchBar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 64
        color: Theme.surface

        Rectangle {
            anchors.fill: parent
            anchors.margins: 8
            radius: 20
            color: Theme.surfaceContainerHigh

            Icon {
                id: searchIcon
                anchors.left: parent.left
                anchors.leftMargin: 14
                anchors.verticalCenter: parent.verticalCenter
                name: "Rounded.Search"
                size: 20
                color: Theme.surfaceVariantText
            }

            TextField {
                id: search
                anchors.left: searchIcon.right
                anchors.leftMargin: 10
                anchors.right: parent.right
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                placeholderText: "Search messages"
                placeholderTextColor: Theme.alpha(Theme.surfaceVariantText, 0.65)
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 15
                background: null
                onTextChanged: root.query = text
            }
        }
    }

    ListView {
        id: list
        anchors.top: searchBar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        model: root.filtered
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        topMargin: 4
        bottomMargin: 12
        spacing: 2
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

        delegate: DmRow {
            required property var modelData
            width: list.width
            mobile: true
            dm: modelData
            selected: backend.selectedChannelId === modelData.id
            onClicked: {
                window.openDm(modelData.id)
                root.openedDm()
            }
        }
    }

    Column {
        anchors.centerIn: parent
        spacing: 12
        visible: list.count === 0

        Icon {
            anchors.horizontalCenter: parent.horizontalCenter
            name: "Rounded.ChatBubble"
            size: 48
            color: Theme.outline
        }
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: window.dms.length === 0 ? "No conversations yet" : "No matches"
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 15
        }
        TextButton {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: window.dms.length === 0
            text: "Find a server"
            onClicked: root.openServers()
        }
    }
}
