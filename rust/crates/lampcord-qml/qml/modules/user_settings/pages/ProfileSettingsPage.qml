import QtQuick
import Lampcord

Column {
    id: root
    property var myProfile: null

    spacing: 14

    Rectangle {
        width: parent.width
        height: 64
        radius: 20
        color: Theme.surfaceContainer

        Row {
            anchors.fill: parent
            anchors.leftMargin: 16
            anchors.rightMargin: 16
            spacing: 14

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: "Public"
                size: 24
                color: Theme.surfaceText
            }
            Column {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 68
                spacing: 1
                Text {
                    text: "Editing Profile for:"
                    color: Theme.surfaceVariantText
                    font.family: Theme.font
                    font.pixelSize: 11
                }
                Text {
                    text: "Global Profile"
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 15
                    font.weight: Font.Bold
                }
            }
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: "KeyboardArrowDown"
                size: 20
                color: Theme.surfaceVariantText
            }
        }
    }

    Rectangle {
        width: parent.width
        height: previewCol.implicitHeight + 18
        radius: 16
        color: Theme.surfaceContainer
        border.width: 2
        border.color: Theme.alpha(Theme.surfaceText, 0.16)
        clip: true

        Column {
            id: previewCol
            width: parent.width
            spacing: 0

            Item {
                width: parent.width
                height: 100

                Rectangle {
                    anchors.fill: parent
                    color: Theme.primaryContainer
                    topLeftRadius: 14
                    topRightRadius: 14
                }
                RoundedImage {
                    anchors.fill: parent
                    visible: root.myProfile && root.myProfile.banner_url ? true : false
                    source: root.myProfile && root.myProfile.banner_url ? root.myProfile.banner_url : ""
                    radius: 14
                }
            }

            Item {
                width: parent.width
                height: 46

                Rectangle {
                    x: 10
                    y: -49
                    width: 94
                    height: 94
                    radius: 47
                    color: Theme.surfaceContainer

                    UserAvatar {
                        anchors.centerIn: parent
                        size: 82
                        name: backend.currentUser
                        source: backend.currentUserAvatar
                        decoration: backend.currentUserDecoration
                        status: backend.currentUserStatus
                        ring: Theme.surfaceContainer
                    }
                }
            }

            Column {
                x: 16
                width: parent.width - 32
                spacing: 8

                Column {
                    width: parent.width
                    spacing: 2
                    Text {
                        text: backend.currentUser || "User"
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 20
                        font.weight: Font.Bold
                    }
                    Text {
                        readonly property string uname: backend.currentUserUsername || ""
                        readonly property string prons: root.myProfile && root.myProfile.pronouns ? root.myProfile.pronouns : ""
                        visible: uname !== "" || prons !== ""
                        text: uname + (uname !== "" && prons !== "" ? "  •  " : "") + prons
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 13
                    }
                }

                Flow {
                    width: parent.width
                    spacing: 6
                    visible: root.myProfile && root.myProfile.badges && root.myProfile.badges.length > 0
                    Repeater {
                        model: root.myProfile && root.myProfile.badges ? root.myProfile.badges : []
                        delegate: Image {
                            required property var modelData
                            width: 20
                            height: 20
                            source: modelData.iconUrl || ("https://cdn.discordapp.com/badge-icons/" + modelData.icon + ".png?size=64")
                            fillMode: Image.PreserveAspectFit
                            asynchronous: true
                        }
                    }
                }

                Column {
                    width: parent.width
                    spacing: 2
                    visible: root.myProfile && root.myProfile.bio && root.myProfile.bio !== "" ? true : false
                    Text {
                        text: "About Me"
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 12
                        font.weight: Font.Bold
                    }
                    Text {
                        width: parent.width
                        text: root.myProfile && root.myProfile.bio ? root.myProfile.bio : ""
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 13
                        wrapMode: Text.Wrap
                    }
                }
            }
        }
    }

    Rectangle {
        width: parent.width
        height: 40
        radius: 20
        color: Theme.secondaryContainer
        opacity: 0.7
        Row {
            anchors.centerIn: parent
            spacing: 8
            Icon { anchors.verticalCenter: parent.verticalCenter; name: "Refresh"; size: 17; color: Theme.secondaryContainerText }
            Text { anchors.verticalCenter: parent.verticalCenter; text: "Recent Avatars"; color: Theme.secondaryContainerText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.DemiBold }
        }
    }

    Row {
        width: parent.width
        spacing: 4
        Rectangle {
            width: (parent.width - 4) / 2
            height: 40
            topLeftRadius: 20; bottomLeftRadius: 20; topRightRadius: 6; bottomRightRadius: 6
            color: Theme.secondaryContainer
            opacity: 0.7
            Row {
                anchors.centerIn: parent
                spacing: 6
                Icon { anchors.verticalCenter: parent.verticalCenter; name: "Delete"; size: 16; color: Theme.secondaryContainerText }
                Text { anchors.verticalCenter: parent.verticalCenter; text: "Remove Avatar"; color: Theme.secondaryContainerText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.Medium }
            }
        }
        Rectangle {
            width: (parent.width - 4) / 2
            height: 40
            topLeftRadius: 6; bottomLeftRadius: 6; topRightRadius: 20; bottomRightRadius: 20
            color: Theme.secondaryContainer
            opacity: 0.7
            Row {
                anchors.centerIn: parent
                spacing: 6
                Icon { anchors.verticalCenter: parent.verticalCenter; name: "Delete"; size: 16; color: Theme.secondaryContainerText }
                Text { anchors.verticalCenter: parent.verticalCenter; text: "Remove Banner"; color: Theme.secondaryContainerText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.Medium }
            }
        }
    }

    SettingsGroupTitle { text: "User Profile" }

    Rectangle {
        width: parent.width
        height: detailsCol.implicitHeight + 32
        radius: 20
        color: Theme.surfaceContainer

        Column {
            id: detailsCol
            anchors.fill: parent
            anchors.margins: 16
            spacing: 12

            Text {
                text: "Details"
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 15
                font.weight: Font.Medium
            }
            OutlinedField {
                width: parent.width
                label: "Display Name"
                text: backend.currentUser
            }
            OutlinedField {
                width: parent.width
                label: "Pronouns"
                text: root.myProfile && root.myProfile.pronouns ? root.myProfile.pronouns : ""
            }
            OutlinedField {
                width: parent.width
                label: "About Me"
                text: root.myProfile && root.myProfile.bio ? root.myProfile.bio : ""
            }
        }
    }

    SettingsGroupTitle { text: "Banner Color" }

    Rectangle {
        width: parent.width
        height: 84
        radius: 20
        color: Theme.surfaceContainer

        Column {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 8

            Text {
                text: "Customize Color"
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 15
                font.weight: Font.Medium
            }
            Row {
                spacing: 10
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Used when no banner image is set:"
                    color: Theme.surfaceVariantText
                    font.family: Theme.font
                    font.pixelSize: 13
                }
                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 24
                    height: 24
                    radius: 12
                    color: root.myProfile && root.myProfile.accent_color
                        ? ("#" + root.myProfile.accent_color.toString(16).padStart(6, "0"))
                        : Theme.outline
                    border.width: 1
                    border.color: Theme.outlineVariant
                }
            }
        }
    }

    SettingsGroupTitle { text: "Profile Themes" }

    Rectangle {
        width: parent.width
        height: 126
        radius: 20
        color: Theme.surfaceContainer

        Column {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 8

            Text {
                text: "Theme Colors"
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 15
                font.weight: Font.Medium
            }
            Text {
                text: "Nitro required."
                color: Theme.surfaceVariantText
                font.family: Theme.font
                font.pixelSize: 12
            }
            Row {
                spacing: 16
                Column {
                    spacing: 4
                    Rectangle {
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: 30; height: 30; radius: 15
                        color: root.myProfile && root.myProfile.theme_colors && root.myProfile.theme_colors.length > 0
                            ? ("#" + root.myProfile.theme_colors[0].toString(16).padStart(6, "0"))
                            : Theme.outline
                    }
                    Text { anchors.horizontalCenter: parent.horizontalCenter; text: "Primary"; color: Theme.surfaceVariantText; font.family: Theme.font; font.pixelSize: 11 }
                }
                Column {
                    spacing: 4
                    Rectangle {
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: 30; height: 30; radius: 15
                        color: root.myProfile && root.myProfile.theme_colors && root.myProfile.theme_colors.length > 1
                            ? ("#" + root.myProfile.theme_colors[1].toString(16).padStart(6, "0"))
                            : Theme.outline
                    }
                    Text { anchors.horizontalCenter: parent.horizontalCenter; text: "Secondary"; color: Theme.surfaceVariantText; font.family: Theme.font; font.pixelSize: 11 }
                }
            }
        }
    }
}
