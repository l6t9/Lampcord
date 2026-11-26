import QtQuick
import Lampcord

Column {
    id: root
    property var savedAccounts: []
    signal editProfileRequested()
    signal closeRequested()

    spacing: 16

    SettingsGroupTitle { text: "Account Information" }

    Rectangle {
        width: parent.width
        height: 82
        radius: 20
        color: Theme.surfaceContainer

        Row {
            anchors.fill: parent
            anchors.leftMargin: 18
            anchors.rightMargin: 18
            spacing: 14

            UserAvatar {
                anchors.verticalCenter: parent.verticalCenter
                size: 52
                name: backend.currentUser
                source: backend.currentUserAvatar
                decoration: backend.currentUserDecoration
                status: ""
                ring: Theme.surfaceContainer
            }
            Column {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 150
                spacing: 2
                Text {
                    text: backend.currentUser || "Signed in"
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 17
                    font.weight: Font.Bold
                    elide: Text.ElideRight
                }
                Text {
                    text: backend.currentUserUsername || backend.currentUser
                    color: Theme.surfaceVariantText
                    font.family: Theme.font
                    font.pixelSize: 13
                    elide: Text.ElideRight
                }
            }
            FilledButton {
                anchors.verticalCenter: parent.verticalCenter
                text: "Edit"
                implicitHeight: 36
                implicitWidth: 74
                font.pixelSize: 14
                onClicked: root.editProfileRequested()
            }
        }
    }

    Column {
        width: parent.width
        spacing: 2

        SettingsInfoRow {
            title: "Username"
            subtitle: backend.currentUserUsername || backend.currentUser || "—"
            first: true
            last: false
        }
        SettingsInfoRow {
            title: "Email"
            subtitle: backend.currentUserEmail !== "" ? backend.currentUserEmail : "—"
            first: false
            last: true
        }
    }

    SettingsGroupTitle { text: "Password and Authentication" }

    Column {
        width: parent.width
        spacing: 2

        SettingsInfoRow {
            title: "Change Password"
            subtitle: ""
            first: true; last: false
        }
        SettingsInfoRow {
            title: "Two-factor authentication"
            subtitle: backend.currentUserMfaEnabled
                ? "Two-Factor Authentication: Enabled"
                : "Two-Factor Authentication: Disabled"
            first: false; last: true
        }
    }

    SettingsGroupTitle {
        visible: root.savedAccounts.length > 1
        text: "Account Switcher"
    }

    Column {
        visible: root.savedAccounts.length > 1
        width: parent.width
        spacing: 2

        Repeater {
            model: root.savedAccounts

            delegate: Rectangle {
                required property var modelData
                required property int index
                readonly property bool first: index === 0
                readonly property bool last: index === root.savedAccounts.length - 1
                width: parent.width
                height: 60
                topLeftRadius: first ? 20 : 4
                topRightRadius: first ? 20 : 4
                bottomLeftRadius: last ? 20 : 4
                bottomRightRadius: last ? 20 : 4
                color: modelData.active ? Theme.secondaryContainer : Theme.surfaceContainer

                Row {
                    anchors.fill: parent
                    anchors.leftMargin: 16
                    anchors.rightMargin: 12
                    spacing: 12

                    Icon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: "AccountCircle"
                        size: 24
                        color: modelData.active ? Theme.secondaryContainerText : Theme.surfaceVariantText
                    }
                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - 170
                        Text {
                            text: modelData.active ? (backend.currentUser || modelData.username || modelData.user_id) : (modelData.username || modelData.user_id)
                            color: modelData.active ? Theme.secondaryContainerText : Theme.surfaceText
                            font.family: Theme.font
                            font.pixelSize: 14
                            font.weight: Font.DemiBold
                            elide: Text.ElideRight
                        }
                        Text {
                            text: modelData.active ? "Active account" : "Saved account"
                            color: Theme.surfaceVariantText
                            font.family: Theme.font
                            font.pixelSize: 12
                        }
                    }
                    TextButton {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: !modelData.active
                        text: "Switch"
                        onClicked: {
                            backend.switchAccount(modelData.user_id)
                            root.closeRequested()
                        }
                    }
                    IconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: !modelData.active
                        glyph: "Close"
                        iconColor: Theme.error
                        tip: "Remove Account"
                        onClicked: backend.removeSavedAccount(modelData.user_id)
                    }
                }
            }
        }
    }
}
