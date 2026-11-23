import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import QtQuick.Shapes

// ExpressiveTooltip: an M3 RichTooltip at the end of its anchor, on
// surfaceContainer with the 16x8 caret pointing back at it.
ToolTip {
    id: root

    property Item anchor: parent
    property int maxWidth: 200

    x: anchor.width + 4 + caret.height
    y: (anchor.height - height) / 2
    delay: 300
    padding: 0
    margins: 0

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 150 }
        NumberAnimation { property: "scale"; from: 0.9; to: 1; duration: 150; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; to: 0; duration: 75 }
    }

    contentItem: Text {
        leftPadding: 16
        rightPadding: 16
        topPadding: 12
        bottomPadding: 12
        width: Math.min(implicitWidth, root.maxWidth)
        text: root.text
        color: Theme.surfaceVariantText
        font.family: Theme.font
        font.pixelSize: 14
        font.letterSpacing: 0.25
        wrapMode: Text.Wrap
    }

    background: Item {
        RectangularShadow {
            anchors.fill: box
            radius: box.radius
            blur: 8
            offset.y: 2
            color: Qt.rgba(0, 0, 0, 0.3)
        }
        Rectangle {
            id: box
            anchors.fill: parent
            radius: 12
            color: Theme.surfaceContainer
        }
        Shape {
            id: caret
            x: -height
            y: (parent.height - width) / 2
            width: 16
            height: 8
            preferredRendererType: Shape.CurveRenderer

            ShapePath {
                strokeWidth: -1
                fillColor: Theme.surfaceContainer
                startX: caret.height; startY: 0
                PathLine { x: 0; y: caret.width / 2 }
                PathLine { x: caret.height; y: caret.width }
                PathLine { x: caret.height; y: 0 }
            }
        }
    }
}
