import QtQuick
import QtQuick.Shapes

// Material 3 Expressive LoadingIndicator: morphs through the indeterminate
// shape sequence (soft burst, 9-sided cookie, pentagon, pill, sunny,
// 4-sided cookie, oval) every 650ms on a spring, turning a quarter per morph
// on top of a full turn every 4666ms. `contained` draws it inside a circle
// at the 38/48 scale of ContainedLoadingIndicator.
Item {
    id: root

    property real size: 48
    property bool contained: false
    property color color: contained ? Theme.primaryContainerText : Theme.primary
    property color containerColor: Theme.primaryContainer

    width: size
    height: size

    readonly property int samples: 96
    readonly property real morphInterval: 0.65
    readonly property real rotationPeriod: 4.666
    property real elapsed: 0

    // Each shape as a radius function of angle, peaking near 1.
    readonly property var shapes: [
        a => 1 - 0.14 * (1 - Math.cos(10 * a)) / 2,                      // soft burst
        a => 1 - 0.12 * (1 - Math.cos(9 * a)) / 2,                       // cookie 9
        a => polygon(a, 5, 0.55),                                        // pentagon
        a => superellipse(a - Math.PI / 4, 1, 0.62, 2.6),                // pill
        a => 1 - 0.16 * Math.pow((1 - Math.cos(8 * a)) / 2, 0.7),        // sunny
        a => 1 - 0.2 * (1 - Math.cos(4 * a)) / 2,                        // cookie 4
        a => superellipse(a - Math.PI / 4, 1, 0.78, 2)                   // oval
    ]

    function polygon(a, sides, rounding) {
        const sector = 2 * Math.PI / sides
        const local = ((a + Math.PI / 2) % sector + sector) % sector - sector / 2
        const flat = Math.cos(Math.PI / sides) / Math.cos(local)
        return flat * (1 - rounding) + Math.cos(Math.PI / sides) * rounding * 1.06
    }

    function superellipse(a, rx, ry, n) {
        const c = Math.abs(Math.cos(a)) / rx
        const s = Math.abs(Math.sin(a)) / ry
        return Math.pow(Math.pow(c, n) + Math.pow(s, n), -1 / n)
    }

    // Underdamped spring (damping 0.6, stiffness 200) from 0 to 1.
    function spring(t) {
        const w = Math.sqrt(200)
        const z = 0.6
        const wd = w * Math.sqrt(1 - z * z)
        return 1 - Math.exp(-z * w * t) * (Math.cos(wd * t) + (z * w / wd) * Math.sin(wd * t))
    }

    function outline() {
        const step = Math.floor(elapsed / morphInterval)
        const progress = spring(elapsed - step * morphInterval)
        const from = shapes[step % shapes.length]
        const to = shapes[(step + 1) % shapes.length]
        const turn = (elapsed / rotationPeriod) * 2 * Math.PI + (step + progress) * Math.PI / 2

        const radius = size / 2 * (contained ? 38 / 48 : 1)
        const points = []
        for (let i = 0; i <= samples; i++) {
            const a = i / samples * 2 * Math.PI
            const r = from(a) + (to(a) - from(a)) * progress
            points.push(Qt.point(size / 2 + Math.cos(a + turn) * r * radius,
                                 size / 2 + Math.sin(a + turn) * r * radius))
        }
        return points
    }

    Rectangle {
        anchors.fill: parent
        visible: root.contained
        radius: width / 2
        color: root.containerColor
    }

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer

        ShapePath {
            strokeWidth: -1
            fillColor: root.color
            PathPolyline { id: line }
        }
    }

    FrameAnimation {
        running: root.visible
        onTriggered: {
            root.elapsed += frameTime
            line.path = root.outline()
        }
    }

    Component.onCompleted: line.path = outline()
}
