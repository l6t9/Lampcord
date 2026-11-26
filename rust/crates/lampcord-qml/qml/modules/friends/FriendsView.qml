import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Item {
    id: root

    property string tab: "online" // online | all | pending | blocked | add
    signal openProfile(var user)

    readonly property var allRelationships: window.relationships
    readonly property var filtered: {
        const q = searchField.text.trim().toLowerCase()
        return allRelationships.filter(rel => {
            if (tab === "online") {
                if (rel.kind !== 1 || rel.status === "offline" || rel.status === "invisible")
                    return false
            } else if (tab === "all") {
                if (rel.kind !== 1) return false
            } else if (tab === "pending") {
                if (rel.kind !== 3 && rel.kind !== 4) return false
            } else if (tab === "blocked") {
                if (rel.kind !== 2) return false
            } else {
                return false
            }
            return q === "" || rel.name.toLowerCase().includes(q) || rel.username.toLowerCase().includes(q)
        })
    }

    Item {
        id: header
        width: parent.width
        height: 56

        Row {
            anchors.left: parent.left
            anchors.leftMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            spacing: 8

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: "Group"
                size: 24
                color: Theme.surfaceVariantText
            }
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "Friends"
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 18
                font.weight: Font.Bold
            }

            Item { width: 8; height: 1 }

            Repeater {
                model: [
                    { key: "online", label: "Online" },
                    { key: "all", label: "All" },
                    { key: "pending", label: "Pending" },
                    { key: "blocked", label: "Blocked" },
                    { key: "add", label: "Add Friend" }
                ]

                delegate: Rectangle {
                    required property var modelData
                    readonly property bool active: root.tab === modelData.key
                    readonly property bool isAdd: modelData.key === "add"
                    width: tabLabel.implicitWidth + 24
                    height: 32
                    radius: 16
                    color: isAdd
                        ? (active ? Theme.primaryContainer : Theme.primary)
                        : (active ? Theme.secondaryContainer : tabHover.hovered ? Theme.alpha(Theme.surfaceText, 0.08) : "transparent")
                    Behavior on color { ColorAnimation { duration: Theme.quick } }

                    Text {
                        id: tabLabel
                        anchors.centerIn: parent
                        text: modelData.label
                        color: isAdd
                            ? (active ? Theme.primaryContainerText : Theme.primaryText)
                            : (active ? Theme.secondaryContainerText : Theme.surfaceVariantText)
                        font.family: Theme.font
                        font.pixelSize: 13
                        font.weight: Font.DemiBold
                    }

                    HoverHandler { id: tabHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: root.tab = modelData.key }
                }
            }
        }

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 1
            color: Theme.outlineVariant
        }
    }

    Column {
        visible: root.tab === "add"
        anchors.top: header.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 24
        spacing: 12

        Text {
            text: "ADD FRIEND"
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 14
            font.weight: Font.Bold
            font.letterSpacing: 0.4
        }
        Text {
            text: "You can add friends with their Discord username."
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 13
        }

        Row {
            width: Math.min(parent.width, 520)
            spacing: 8

            OutlinedField {
                id: addFriendInput
                width: parent.width - sendReqBtn.width - 8
                label: "Enter a username"
                onAccepted: sendReqBtn.clicked()
            }
            FilledButton {
                id: sendReqBtn
                anchors.verticalCenter: addFriendInput.verticalCenter
                text: "Send Request"
                enabled: addFriendInput.text.trim() !== ""
                onClicked: {
                    backend.sendFriendRequest(addFriendInput.text.trim())
                    addFriendInput.text = ""
                }
            }
        }
    }

    Item {
        visible: root.tab !== "add"
        anchors.top: header.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right

        Rectangle {
            id: searchBar
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: 16
            height: 40
            radius: 20
            color: Theme.surfaceContainerHigh

            Row {
                anchors.fill: parent
                anchors.leftMargin: 14
                anchors.rightMargin: 14
                spacing: 8

                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "Search"
                    size: 18
                    color: Theme.surfaceVariantText
                }
                TextInput {
                    id: searchField
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 28
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 14
                    clip: true

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: searchField.text === "" && !searchField.activeFocus
                        text: "Search"
                        color: Theme.surfaceVariantText
                        font: searchField.font
                    }
                }
            }
        }

        ListView {
            id: friendList
            anchors.top: searchBar.bottom
            anchors.topMargin: 8
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            model: root.filtered
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

            header: Text {
                width: friendList.width
                leftPadding: 20
                topPadding: 6
                bottomPadding: 8
                text: root.tab.toUpperCase() + " — " + root.filtered.length
                color: Theme.surfaceVariantText
                font.family: Theme.font
                font.pixelSize: 11
                font.weight: Font.Bold
                font.letterSpacing: 0.5
            }

            delegate: Rectangle {
                required property var modelData
                width: friendList.width - 24
                x: 12
                height: 60
                radius: 14
                color: rowHover.hovered ? Theme.alpha(Theme.surfaceVariant, 0.2) : "transparent"
                Behavior on color { ColorAnimation { duration: Theme.quick } }

                Row {
                    anchors.fill: parent
                    anchors.leftMargin: 12
                    anchors.rightMargin: 12
                    spacing: 12

                    UserAvatar {
                        anchors.verticalCenter: parent.verticalCenter
                        size: 40
                        name: modelData.name
                        source: modelData.avatar
                        decoration: modelData.decoration
                        status: modelData.status
                        ring: Theme.surface
                    }

                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - 40 - rowActions.width - 24
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
                            text: modelData.kind === 3 ? "Incoming Friend Request"
                                : modelData.kind === 4 ? "Outgoing Friend Request"
                                : modelData.kind === 2 ? "Blocked"
                                : modelData.activity !== "" ? modelData.activity
                                : modelData.status
                            color: Theme.surfaceVariantText
                            font.family: Theme.font
                            font.pixelSize: 12
                            elide: Text.ElideRight
                        }
                    }

                    Row {
                        id: rowActions
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 4

                        IconButton {
                            visible: modelData.kind === 1
                            glyph: "ChatBubble"
                            tip: "Message"
                            onClicked: backend.openDm(modelData.id)
                        }
                        IconButton {
                            visible: modelData.kind === 3
                            glyph: "Check"
                            iconColor: "#23A559"
                            tip: "Accept"
                            onClicked: backend.acceptFriendRequest(modelData.id)
                        }
                        IconButton {
                            glyph: "Close"
                            iconColor: Theme.error
                            tip: modelData.kind === 2 ? "Unblock" : "Remove"
                            onClicked: backend.removeRelationship(modelData.id)
                        }
                    }
                }

                HoverHandler { id: rowHover; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: root.openProfile(modelData) }
            }
        }

        Text {
            anchors.centerIn: parent
            visible: root.filtered.length === 0
            text: root.tab === "online" ? "No friends online right now"
                : root.tab === "pending" ? "No pending friend requests"
                : root.tab === "blocked" ? "No blocked users"
                : "No friends found"
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 14
        }
    }
}
