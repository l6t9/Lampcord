import QtQuick

// Presence dot in Discord's colours, ringed in the surface colour so it
// reads on top of an avatar. Offline and invisible show a hollow ring.
Rectangle {
    property string status: "offline"
    property color ring: Theme.surface

    readonly property bool hollow: status === "offline" || status === "invisible"

    width: 12
    height: 12
    radius: 6
    color: ({ online: "#23A559", idle: "#F0B232", dnd: "#F23F43" })[status] || "#80848E"
    border.width: 2
    border.color: ring

    Rectangle {
        anchors.centerIn: parent
        visible: parent.hollow
        width: 4
        height: 4
        radius: 2
        color: parent.ring
    }
}
