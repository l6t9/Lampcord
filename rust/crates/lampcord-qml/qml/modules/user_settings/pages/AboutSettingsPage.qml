import QtQuick
import QtQuick.Effects
import Lampcord

Column {
    spacing: 18

    Rectangle {
        width: parent.width
        height: 100
        radius: 24
        color: Theme.surfaceContainerHigh

        Row {
            anchors.fill: parent
            anchors.margins: 20
            spacing: 18

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: 60
                height: 60
                radius: 18
                color: Theme.primaryContainer

                Icon {
                    anchors.centerIn: parent
                    name: "discord"
                    size: 32
                    color: Theme.primaryContainerText
                }
            }
            Column {
                anchors.verticalCenter: parent.verticalCenter
                spacing: 4

                Text {
                    text: "Lampcord"
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 24
                    font.weight: Font.Bold
                }
                Rectangle {
                    height: 22
                    width: verText.implicitWidth + 16
                    radius: 11
                    color: Theme.surfaceContainerHighest
                    Text {
                        id: verText
                        anchors.centerIn: parent
                        text: "1.0.8"
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 11
                        font.weight: Font.Medium
                    }
                }
            }
        }
    }

    Rectangle {
        width: parent.width
        height: devCol.implicitHeight + 36
        radius: 24
        color: Theme.surfaceContainerHigh

        Column {
            id: devCol
            anchors.fill: parent
            anchors.margins: 18
            spacing: 14

            Row {
                spacing: 16
                Item {
                    width: 80
                    height: 80

                    Image {
                        id: lampAvatarImg
                        anchors.fill: parent
                        source: "https://github.com/l6t9.png"
                        sourceSize: Qt.size(256, 256)
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                        smooth: true
                        mipmap: true
                        visible: false
                    }
                    // 9-sided M3 Expressive cookie mask, supersampled for clean alpha edges.
                    Image {
                        id: cookie9Mask
                        anchors.fill: parent
                        visible: false
                        smooth: true
                        layer.enabled: true
                        layer.smooth: true
                        layer.textureSize: Qt.size(320, 320)
                        sourceSize: Qt.size(320, 320)
                        source: "data:image/svg+xml;utf8,"
                            + "<svg xmlns='http://www.w3.org/2000/svg' viewBox='-2 -2 104 104'>"
                            + "<path fill='%23ffffff' d='M42.73 5.53 C47.58 2.51 52.42 2.51 57.27 5.53 C62.11 8.55 67.36 10.46 73.02 11.26 C78.67 12.06 82.38 15.18 84.15 20.61 C85.92 26.04 88.71 30.87 92.53 35.12 C96.35 39.37 97.19 44.14 95.05 49.44 C92.92 54.73 91.95 60.23 92.14 65.94 C92.34 71.65 89.92 75.84 84.88 78.53 C79.84 81.21 75.56 84.80 72.04 89.30 C68.52 93.80 63.97 95.45 58.38 94.27 C52.79 93.09 47.21 93.09 41.62 94.27 C36.03 95.45 31.48 93.80 27.96 89.30 C24.44 84.80 20.16 81.21 15.12 78.53 C10.08 75.84 7.66 71.65 7.86 65.94 C8.05 60.23 7.08 54.73 4.95 49.44 C2.81 44.14 3.65 39.37 7.47 35.12 C11.29 30.87 14.08 26.04 15.85 20.61 C17.62 15.18 21.33 12.06 26.98 11.26 C32.64 10.46 37.89 8.55 42.73 5.53 Z'/>"
                            + "</svg>"
                    }
                    MultiEffect {
                        anchors.fill: parent
                        source: lampAvatarImg
                        maskEnabled: true
                        maskSource: cookie9Mask
                        maskThresholdMin: 0.5
                        maskSpreadAtMin: 0.25
                    }
                }
                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 2
                    Text {
                        text: "Lamp"
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 22
                        font.weight: Font.Bold
                    }
                    Text {
                        text: "Main Developer"
                        color: Theme.primary
                        font.family: Theme.font
                        font.pixelSize: 13
                        font.weight: Font.Medium
                    }
                }
            }

            Row {
                width: parent.width
                spacing: 4

                Rectangle {
                    width: (parent.width - 4) / 2
                    height: 40
                    topLeftRadius: 20; bottomLeftRadius: 20; topRightRadius: 6; bottomRightRadius: 6
                    color: webHover.hovered ? Theme.surfaceContainerHighest : Theme.secondaryContainer
                    Icon { anchors.centerIn: parent; name: "Language"; size: 19; color: Theme.secondaryContainerText }
                    HoverHandler { id: webHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: Qt.openUrlExternally("https://lamp.delivery") }
                    RichTooltip { anchor: parent; placement: "above"; visible: webHover.hovered; text: "Open Website" }
                }
                Rectangle {
                    width: (parent.width - 4) / 2
                    height: 40
                    topLeftRadius: 6; bottomLeftRadius: 6; topRightRadius: 20; bottomRightRadius: 20
                    color: ghHover.hovered ? Theme.surfaceContainerHighest : Theme.secondaryContainer
                    Icon { anchors.centerIn: parent; name: "github"; size: 19; color: Theme.secondaryContainerText }
                    HoverHandler { id: ghHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: Qt.openUrlExternally("https://github.com/l6t9") }
                    RichTooltip { anchor: parent; placement: "above"; visible: ghHover.hovered; text: "Open GitHub Profile" }
                }
            }

            Rectangle {
                width: parent.width
                height: 40
                radius: 20
                color: Theme.primary
                opacity: supHover.hovered ? 0.9 : 1.0
                Row {
                    anchors.centerIn: parent
                    spacing: 8
                    Icon { anchors.verticalCenter: parent.verticalCenter; name: "Favorite"; size: 17; color: Theme.primaryText }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Support the development"
                        color: Theme.primaryText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.weight: Font.DemiBold
                    }
                }
                HoverHandler { id: supHover; cursorShape: Qt.PointingHandCursor }
                TapHandler { onTapped: Qt.openUrlExternally("https://github.com/sponsors/l6t9") }
            }
        }
    }

    SettingsGroupTitle { text: "Community & Info" }

    Column {
        width: parent.width
        spacing: 2

        SettingsActionRow {
            icon: "discord"
            title: "Discord"
            first: true; last: false
            onClicked: Qt.openUrlExternally("https://discord.gg/uHXJJzxSD8")
        }
        SettingsActionRow {
            icon: "github"
            title: "GitHub Repository"
            first: false; last: false
            onClicked: Qt.openUrlExternally("https://github.com/l6t9/lampcord")
        }
        SettingsActionRow {
            icon: "Info"
            title: "License"
            first: false; last: true
            onClicked: Qt.openUrlExternally("https://github.com/l6t9/lampcord/blob/main/LICENSE")
        }
    }
}
