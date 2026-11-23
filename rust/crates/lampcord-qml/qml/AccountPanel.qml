import QtCore
import QtQuick
import QtQuick.Controls.Basic

// AccountPanel: avatar with status, name and status line (opens the status
// menu), the mute/deafen pair and the settings button.
Rectangle {
    id: root

    implicitHeight: 64
    radius: 20
    color: Theme.surfaceContainerHighest

    readonly property string status: backend.currentUserStatus
    readonly property string statusLabel: ({ idle: "Idle", dnd: "Do Not Disturb", invisible: "Invisible" })[status] || "Online"

    // There is no voice in the core yet; the toggles are remembered so
    // they apply once it lands, as VoiceStore does when not in a call.
    Settings {
        id: voice
        category: "voice"
        property bool muted: false
        property bool deafened: false
    }

    Row {
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        spacing: 6

        // Avatar + name: opens the status menu.
        Rectangle {
            id: identity
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - voiceToggles.width - settings.width - 12
            height: 48
            radius: 18
            color: identityHover.hovered ? Theme.alpha(Theme.surfaceText, 0.06) : "transparent"
            Behavior on color { ColorAnimation { duration: Theme.quick } }

            Row {
                anchors.fill: parent
                anchors.leftMargin: 4
                anchors.rightMargin: 8
                spacing: 10

                Item {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 40
                    height: 40

                    Avatar {
                        size: 40
                        name: backend.currentUser
                        source: backend.currentUserAvatar
                    }
                    StatusDot {
                        x: 40 - width + 2
                        y: 40 - height + 2
                        size: 16
                        status: root.status
                        ring: Theme.surfaceContainerHighest
                    }
                }

                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 50

                    Text {
                        width: parent.width
                        text: backend.currentUser || "…"
                        color: Theme.surfaceText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.letterSpacing: 0.1
                        font.weight: Font.DemiBold
                        elide: Text.ElideRight
                    }
                    Row {
                        width: parent.width
                        spacing: 0

                        Text {
                            id: statusLine
                            width: Math.min(implicitWidth, parent.width - 16)
                            text: backend.connection !== "ready" ? "Connecting…"
                                : backend.currentUserCustomStatus !== "" ? backend.currentUserCustomStatus
                                : root.statusLabel
                            color: Theme.surfaceVariantText
                            font.family: Theme.font
                            font.pixelSize: 12
                            font.letterSpacing: 0.5
                            font.weight: Font.Medium
                            elide: Text.ElideRight
                        }
                        Icon {
                            anchors.verticalCenter: statusLine.verticalCenter
                            visible: backend.currentUserCustomStatus === ""
                            name: "Rounded.ExpandMore"
                            size: 16
                            color: Theme.surfaceVariantText
                        }
                    }
                }
            }

            HoverHandler {
                id: identityHover
                cursorShape: Qt.PointingHandCursor
            }
            TapHandler {
                onTapped: statusMenu.open()
            }

            LampMenu {
                id: statusMenu
                y: -implicitHeight - 8

                Repeater {
                    model: [["online", "Online"], ["idle", "Idle"], ["dnd", "Do Not Disturb"], ["invisible", "Invisible"]]

                    delegate: LampMenuItem {
                        required property var modelData
                        text: modelData[1]
                        onTriggered: backend.setStatus(modelData[0])

                        leading: Component {
                            StatusDot {
                                size: 12
                                status: modelData[0]
                                ring: "transparent"
                            }
                        }
                    }
                }
            }
        }

        Row {
            id: voiceToggles
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            VoiceToggle {
                leading: true
                active: voice.muted
                glyph: active ? "MicOff" : "Rounded.Mic"
                tip: active ? "Unmute" : "Mute"
                onClicked: voice.muted = !voice.muted
            }
            VoiceToggle {
                leading: false
                active: voice.deafened
                glyph: active ? "HeadsetOff" : "Rounded.Headphones"
                tip: active ? "Undeafen" : "Deafen"
                onClicked: voice.deafened = !voice.deafened
            }
        }

        IconButton {
            id: settings
            anchors.verticalCenter: parent.verticalCenter
            glyph: "Rounded.Settings"
            iconSize: 20
            tip: "User Settings"
            onClicked: settingsMenu.open()

            LampMenu {
                id: settingsMenu
                y: -implicitHeight - 8

                LampMenuItem {
                    text: "Log Out"
                    danger: true
                    leading: Component { Icon { name: "Logout"; size: 20; color: Theme.error } }
                    onTriggered: backend.logout()
                }
            }
        }
    }

    // AccountVoiceToggle: half of a pill; the inner corners round up when on.
    component VoiceToggle: AbstractButton {
        id: toggle

        property bool leading
        property bool active
        property string glyph
        property string tip

        property real inner: active ? 20 : 8
        Behavior on inner { NumberAnimation { duration: Theme.medium; easing.type: Easing.OutBack } }

        width: 44
        height: 40
        hoverEnabled: true

        background: Rectangle {
            color: toggle.active ? Theme.errorContainer : Theme.secondaryContainer
            topLeftRadius: toggle.leading ? 20 : toggle.inner
            bottomLeftRadius: toggle.leading ? 20 : toggle.inner
            topRightRadius: toggle.leading ? toggle.inner : 20
            bottomRightRadius: toggle.leading ? toggle.inner : 20
            Behavior on color { ColorAnimation { duration: Theme.quick } }

            Rectangle {
                anchors.fill: parent
                topLeftRadius: parent.topLeftRadius
                bottomLeftRadius: parent.bottomLeftRadius
                topRightRadius: parent.topRightRadius
                bottomRightRadius: parent.bottomRightRadius
                color: Theme.surfaceText
                opacity: toggle.down ? 0.12 : toggle.hovered ? 0.08 : 0
            }
        }
        contentItem: Item {
            Icon {
                anchors.centerIn: parent
                name: toggle.glyph
                size: 20
                color: toggle.active ? Theme.errorContainerText : Theme.secondaryContainerText
            }
        }

        ToolTip.visible: hovered
        ToolTip.text: tip
        ToolTip.delay: 500
        HoverHandler { cursorShape: Qt.PointingHandCursor }
    }
}
