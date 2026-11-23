import QtQuick

// DMItem: 48px rows inset by 8px with a 32px avatar, small-shape highlight
// when selected.
Item {
    id: root

    property var dm
    property bool selected: false
    signal clicked()

    implicitHeight: 48

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        radius: 8
        color: root.selected ? Theme.surfaceVariant
             : hover.hovered ? Theme.alpha(Theme.surfaceText, 0.05) : "transparent"
        Behavior on color { ColorAnimation { duration: Theme.quick } }

        Row {
            anchors.fill: parent
            anchors.leftMargin: 8
            anchors.rightMargin: 8
            spacing: 12

            Item {
                anchors.verticalCenter: parent.verticalCenter
                width: 32
                height: 32

                Avatar {
                    visible: !root.dm.group
                    size: 32
                    name: root.dm.name
                    source: root.dm.avatar
                }
                Rectangle {
                    visible: root.dm.group
                    anchors.fill: parent
                    radius: 16
                    color: Theme.primaryContainer

                    Icon {
                        anchors.centerIn: parent
                        name: "group"
                        size: 20
                        color: Theme.primaryContainerText
                    }
                }
            }

            Column {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 44 - (badge.visible ? badge.width + 12 : 0)

                Text {
                    width: parent.width
                    text: root.dm.name
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 14
                    font.letterSpacing: 0.25
                    elide: Text.ElideRight
                }
                Text {
                    width: parent.width
                    visible: root.dm.group
                    text: root.dm.members + " Members"
                    color: Theme.surfaceVariantText
                    font.family: Theme.font
                    font.pixelSize: 12
                    elide: Text.ElideRight
                }
            }

            Rectangle {
                id: badge
                anchors.verticalCenter: parent.verticalCenter
                visible: root.dm.mentions > 0
                height: 16
                width: Math.max(16, count.implicitWidth + 8)
                radius: 8
                color: Theme.error

                Text {
                    id: count
                    anchors.centerIn: parent
                    text: root.dm.mentions > 99 ? "99+" : root.dm.mentions
                    color: Theme.errorText
                    font.family: Theme.font
                    font.pixelSize: 11
                    font.weight: Font.Bold
                }
            }
        }
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.PointingHandCursor
    }
    TapHandler {
        onTapped: root.clicked()
    }
}
