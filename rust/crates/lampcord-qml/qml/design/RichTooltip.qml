import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import QtQuick.Shapes
import Lampcord

// ExpressiveTooltip: an M3 RichTooltip on surfaceContainer with a directional
// 16x8 caret pointing back at its anchor. Supports placement: "end" (default,
// used on server rail), "above", or "below".
ToolTip {
    id: root

    property Item anchor: parent
    property string placement: "end" // "end" | "above" | "below"
    property int maxWidth: 260

    property string resolvedPlacement: placement

    function updatePlacement() {
        if (!anchor || placement === "end") {
            resolvedPlacement = placement
            return
        }
        const topY = anchor.mapToItem(null, 0, 0).y
        const bottomY = anchor.mapToItem(null, 0, anchor.height).y
        const needed = Math.max(implicitHeight, 44) + 18
        const winH = Overlay.overlay ? Overlay.overlay.height : 0
        if (placement === "above") {
            resolvedPlacement = topY < needed ? "below" : "above"
        } else if (placement === "below") {
            resolvedPlacement = (winH > 0 && bottomY + needed > winH) ? "above" : "below"
        } else {
            resolvedPlacement = placement
        }
    }

    onAboutToShow: updatePlacement()
    onVisibleChanged: if (visible) updatePlacement()

    x: resolvedPlacement === "end"
        ? (anchor ? anchor.width + 12 : 12)
        : (anchor ? Math.round((anchor.width - width) / 2) : 0)
    y: resolvedPlacement === "end"
        ? (anchor ? Math.round((anchor.height - height) / 2) : 0)
        : resolvedPlacement === "below"
            ? (anchor ? anchor.height + 10 : 10)
            : (-height - 10)
    delay: 300
    padding: 0
    margins: 8

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 140 }
        NumberAnimation { property: "scale"; from: 0.92; to: 1; duration: 140; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; to: 0; duration: 75 }
    }

    contentItem: Text {
        leftPadding: 14
        rightPadding: 14
        topPadding: 9
        bottomPadding: 9
        width: Math.min(implicitWidth, root.maxWidth)
        text: root.text
        color: Theme.surfaceVariantText
        font.family: Theme.font
        font.pixelSize: 13
        font.weight: Font.Medium
        font.letterSpacing: 0.2
        wrapMode: Text.Wrap
    }

    background: Item {
        RectangularShadow {
            anchors.fill: box
            radius: box.radius
            blur: 10
            offset.y: 2
            color: Qt.rgba(0, 0, 0, 0.32)
        }
        Rectangle {
            id: box
            anchors.fill: parent
            radius: 12
            color: Theme.surfaceContainer
        }
        // Left-pointing caret (for placement === "end")
        Shape {
            visible: root.resolvedPlacement === "end"
            x: -8
            y: (parent.height - 16) / 2
            width: 8
            height: 16
            ShapePath {
                strokeWidth: -1
                fillColor: Theme.surfaceContainer
                startX: 8; startY: 0
                PathLine { x: 0; y: 8 }
                PathLine { x: 8; y: 16 }
                PathLine { x: 8; y: 0 }
            }
        }
        // Bottom-pointing caret (for placement === "above")
        Shape {
            visible: root.resolvedPlacement === "above"
            x: (parent.width - 16) / 2
            y: parent.height
            width: 16
            height: 8
            ShapePath {
                strokeWidth: -1
                fillColor: Theme.surfaceContainer
                startX: 0; startY: 0
                PathLine { x: 8; y: 8 }
                PathLine { x: 16; y: 0 }
                PathLine { x: 0; y: 0 }
            }
        }
        // Top-pointing caret (for placement === "below")
        Shape {
            visible: root.resolvedPlacement === "below"
            x: (parent.width - 16) / 2
            y: -8
            width: 16
            height: 8
            ShapePath {
                strokeWidth: -1
                fillColor: Theme.surfaceContainer
                startX: 0; startY: 8
                PathLine { x: 8; y: 0 }
                PathLine { x: 16; y: 8 }
                PathLine { x: 0; y: 8 }
            }
        }
    }
}
