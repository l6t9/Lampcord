import QtQuick
import Lampcord

// AvatarWithDecoration: the avatar, its frame drawn at 1.2x over it, and
// the status dot at 35% of the size tucked 2px past the corner.
Item {
    id: root

    property string source
    property string name
    property string decoration
    property string status
    property color ring: Theme.surface
    property real size: 40

    width: size
    height: size

    Avatar {
        size: root.size
        name: root.name
        source: root.source
    }

    Image {
        anchors.centerIn: parent
        width: root.size * 1.2
        height: width
        visible: root.decoration !== ""
        source: root.decoration
        sourceSize: Qt.size(width * 2, height * 2)
        asynchronous: true
        cache: true
    }

    StatusDot {
        visible: root.status !== ""
        size: root.size * 0.35
        x: root.size - width + 2
        y: root.size - height + 2
        status: root.status
        ring: root.ring
    }
}
