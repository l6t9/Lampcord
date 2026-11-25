import QtQuick
import QtQuick.Controls.Basic

// MemberList on desktop: the search pill, "Members — N" and the rows. The
// core can't search messages yet, so the pill filters this list instead.
Item {
    id: root

    // The overlapping member sheet needs a back affordance; the desktop panel
    // keeps its original search pill at the top.
    property bool backVisible: false
    property bool mobile: false
    signal backClicked()

    readonly property var members: JSON.parse(backend.membersJson)
    readonly property var shown: {
        const needle = search.text.trim().toLowerCase()
        return needle === "" ? members : members.filter(m => m.name.toLowerCase().includes(needle))
    }

    Connections {
        target: backend
        function onSelectedChannelIdChanged() { search.clear() }
    }

    Row {
        id: mobileHeader
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: visible ? (root.mobile ? 56 : 52) : 0
        visible: root.backVisible

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 1
            color: root.mobile ? Theme.outlineVariant : "transparent"
        }

        IconButton {
            anchors.verticalCenter: parent.verticalCenter
            width: 40
            height: 40
            glyph: "Rounded.ArrowBack"
            iconColor: Theme.surfaceVariantText
            tip: "Back to chat"
            onClicked: root.backClicked()
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: "Members"
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: root.mobile ? 18 : 16
            font.weight: Font.DemiBold
        }
    }

    Rectangle {
        id: pill
        x: 16
        y: mobileHeader.height + 8
        width: parent.width - 32
        height: 40
        radius: root.mobile ? 12 : 20
        color: Theme.surfaceContainerHigh

        Row {
            anchors.fill: parent
            anchors.leftMargin: 12
            anchors.rightMargin: 12
            spacing: 10

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: "Search"
                size: 18
                color: Theme.surfaceVariantText
            }
            TextInput {
                id: search
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 28
                color: Theme.surfaceText
                selectionColor: Theme.alpha(Theme.primary, 0.4)
                selectedTextColor: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 14
                font.letterSpacing: 0.25
                clip: true

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: search.text === "" && !search.activeFocus
                    text: "Search"
                    color: Theme.surfaceVariantText
                    font: search.font
                }
            }
        }

        HoverHandler { cursorShape: Qt.IBeamCursor }
        TapHandler { onTapped: search.forceActiveFocus() }
    }

    ListView {
        id: list
        anchors.top: pill.bottom
        anchors.topMargin: 8
        anchors.bottom: parent.bottom
        width: parent.width
        model: root.shown
        spacing: 2
        bottomMargin: 8
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

        header: Text {
            width: list.width
            leftPadding: 16
            topPadding: 12
            bottomPadding: 12
            text: "Members — " + root.shown.length
            color: Theme.alpha(Theme.surfaceVariantText, 0.6)
            font.family: Theme.font
            font.pixelSize: 11
            font.weight: Font.Bold
            font.letterSpacing: 0.5
        }

        delegate: MemberRow {
            required property var modelData
            width: list.width
            mobile: root.mobile
            member: modelData
        }
    }
}
