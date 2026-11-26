import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// The mobile You tab: profile, presence, custom status and the settings that
// are actually backed by this client. Everything here writes through the
// backend or the shared navigation settings; there are no decorative rows.
Item {
    id: root

    readonly property var profile: {
        const raw = JSON.parse(backend.profileJson)
        return raw && raw.user_id !== undefined ? raw : null
    }
    readonly property string displayName: profile && (profile.nickname || profile.global_name || profile.username)
        ? (profile.nickname || profile.global_name || profile.username) : backend.currentUser
    readonly property string username: profile && profile.username ? "@" + profile.username : ""

    function handleBack() {
        if (logoutConfirm.opened) {
            logoutConfirm.close()
            return true
        }
        customStatus.focus = false
        return false
    }

    Component.onCompleted: backend.fetchProfile(backend.currentUserId)

    Rectangle {
        anchors.fill: parent
        color: Theme.surface
    }

    Flickable {
        anchors.fill: parent
        contentHeight: column.implicitHeight + 32
        boundsBehavior: Flickable.StopAtBounds
        clip: true
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

        Column {
            id: column
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: 12
            spacing: 12

            // ----------------------------------------------------------- profile
            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                radius: 16
                color: Theme.surfaceContainerLow

                Column {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.margins: 16
                    spacing: 10

                    Row {
                        width: parent.width
                        spacing: 14

                        UserAvatar {
                            anchors.verticalCenter: parent.verticalCenter
                            size: 64
                            name: root.displayName
                            source: backend.currentUserAvatar
                            decoration: backend.currentUserDecoration
                            status: backend.currentUserStatus
                        }

                        Column {
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width - 78
                            spacing: 2

                            Text {
                                width: parent.width
                                text: root.displayName
                                color: Theme.surfaceText
                                font.family: Theme.font
                                font.pixelSize: 20
                                font.weight: Font.Bold
                                elide: Text.ElideRight
                            }
                            Text {
                                width: parent.width
                                visible: root.username !== ""
                                text: root.username
                                color: Theme.surfaceVariantText
                                font.family: Theme.font
                                font.pixelSize: 13
                                elide: Text.ElideRight
                            }
                            Text {
                                width: parent.width
                                visible: backend.currentUserCustomStatus !== ""
                                text: backend.currentUserCustomStatus
                                color: Theme.primary
                                font.family: Theme.font
                                font.pixelSize: 13
                                elide: Text.ElideRight
                            }
                        }
                    }

                    Text {
                        width: parent.width
                        visible: profile && profile.bio ? true : false
                        text: profile && profile.bio ? profile.bio : ""
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 14
                        wrapMode: Text.Wrap
                    }

                    TextInput {
                        width: parent.width
                        text: "ID: " + backend.currentUserId
                        readOnly: true
                        selectByMouse: true
                        color: Theme.alpha(Theme.surfaceVariantText, 0.7)
                        font.family: Theme.font
                        font.pixelSize: 11
                    }
                }
            }

            // ---------------------------------------------------------- presence
            Section {
                title: "Status"

                contentComponent: Column {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    spacing: 8

                    Row {
                        width: parent.width
                        spacing: 8

                        Repeater {
                            model: [
                                { key: "online", label: "Online", color: "#23A559" },
                                { key: "idle", label: "Idle", color: "#F0B232" },
                                { key: "dnd", label: "Do Not Disturb", color: "#F23F43" },
                                { key: "invisible", label: "Invisible", color: "#80848E" }
                            ]

                            Button {
                                required property var modelData
                                width: (parent.width - 24) / 4
                                height: 48
                                checked: backend.currentUserStatus === modelData.key
                                hoverEnabled: true
                                onClicked: backend.setStatus(modelData.key)

                                contentItem: Column {
                                    anchors.centerIn: parent
                                    spacing: 4
                                    Rectangle {
                                        anchors.horizontalCenter: parent.horizontalCenter
                                        width: 14
                                        height: 14
                                        radius: 7
                                        color: modelData.color
                                    }
                                    Text {
                                        anchors.horizontalCenter: parent.horizontalCenter
                                        text: modelData.label
                                        color: Theme.surfaceText
                                        font.family: Theme.font
                                        font.pixelSize: 10
                                        elide: Text.ElideRight
                                    }
                                }

                                background: Rectangle {
                                    radius: 12
                                    color: checked ? Theme.secondaryContainer
                                        : Theme.alpha(Theme.surfaceVariant, parent.hovered ? 0.16 : 0.08)
                                    border.width: checked ? 1 : 0
                                    border.color: Theme.primary
                                }
                            }
                        }
                    }

                    TextField {
                        id: customStatus
                        anchors.left: parent.left
                        anchors.right: saveStatus.left
                        anchors.rightMargin: 8
                        placeholderText: "What are you up to?"
                        text: backend.currentUserCustomStatus
                        color: Theme.surfaceText
                        placeholderTextColor: Theme.alpha(Theme.surfaceVariantText, 0.65)
                        font.family: Theme.font
                        font.pixelSize: 14
                        background: Rectangle {
                            radius: 12
                            color: Theme.surfaceContainerHigh
                        }
                        leftPadding: 12
                        rightPadding: 12
                        topPadding: 12
                        bottomPadding: 12
                        onAccepted: backend.setCustomStatus(text)
                    }
                    TextButton {
                        id: saveStatus
                        anchors.right: parent.right
                        anchors.verticalCenter: customStatus.verticalCenter
                        text: "Save"
                        onClicked: backend.setCustomStatus(customStatus.text)
                    }
                }
            }

            // -------------------------------------------------------- navigation
            Section {
                title: "Bottom navigation"

                contentComponent: Column {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    spacing: 0

                    ToggleRow { label: "Home"; setting: memory.showHome }
                    ToggleRow { label: "Messages"; setting: memory.showDms }
                    ToggleRow { label: "Search"; setting: memory.showSearch }
                    ToggleRow { label: "Mentions"; setting: memory.showMentions }
                    ToggleRow { label: "You"; setting: memory.showYou }
                }
            }

            // ----------------------------------------------------------- account
            Section {
                title: "Account"

                contentComponent: Column {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    spacing: 0

                    TextButton {
                        width: parent.width
                        horizontalAlignment: Text.AlignLeft
                        text: "Log out"
                        onClicked: logoutConfirm.open()
                    }
                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: 12
                        width: parent.width - 24
                        text: "QML stores its session next to the desktop client's, so both apps stay signed in independently."
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 12
                        wrapMode: Text.Wrap
                    }
                }
            }
        }
    }

    component Section: Rectangle {
        id: section
        property string title
        property Component contentComponent
        anchors.left: parent.left
        anchors.right: parent.right
        radius: 16
        color: Theme.surfaceContainerLow

        Column {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: 12
            spacing: 8

            Text {
                text: section.title
                color: Theme.primary
                font.family: Theme.font
                font.pixelSize: 12
                font.weight: Font.DemiBold
                font.letterSpacing: 0.5
            }
            Loader {
                anchors.left: parent.left
                anchors.right: parent.right
                sourceComponent: section.contentComponent
            }
        }
    }

    component ToggleRow: Item {
        id: toggleRow
        property string label
        property alias setting: toggle.checked
        width: parent.width
        height: 48

        Text {
            anchors.left: parent.left
            anchors.leftMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            text: toggleRow.label
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 14
        }

        Switch {
            id: toggle
            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter
        }
    }

    Popup {
        id: logoutConfirm
        anchors.centerIn: parent
        width: Math.min(320, root.width - 32)
        height: 176
        padding: 0
        modal: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        background: Rectangle {
            radius: 20
            color: Theme.surfaceContainerHigh
        }

        contentItem: Column {
            anchors.margins: 20
            spacing: 12

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: "Log out?"
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 18
                font.weight: Font.DemiBold
            }
            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width
                text: "This removes the saved QML session. You will need your token or password to sign in again."
                color: Theme.surfaceVariantText
                font.family: Theme.font
                font.pixelSize: 13
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
            }
            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: 8

                TextButton {
                    width: 120
                    text: "Cancel"
                    onClicked: logoutConfirm.close()
                }
                FilledButton {
                    width: 120
                    text: "Log out"
                    onClicked: {
                        logoutConfirm.close()
                        backend.logout()
                    }
                }
            }
        }
    }
}
