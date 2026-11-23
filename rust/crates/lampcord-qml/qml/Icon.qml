import QtQuick
import QtQuick.Shapes
import Lampcord

// The Compose client's Material Symbols, by their Compose name: "Send" is
// Icons.Filled.Send, "Rounded.Send" is Icons.Rounded.Send. Paths come from
// IconPaths (a 960 grid with y in -960..0). "discord" is the brand mark.
Item {
    id: root

    property string name
    property color color: Theme.surfaceText
    property real size: 24

    implicitWidth: size
    implicitHeight: size

    readonly property bool outline: name.startsWith("Rounded.")
    readonly property string symbol: outline ? name.slice(8) : name
    readonly property bool brand: name === "discord"
    readonly property real viewBox: brand ? 120 : 960

    readonly property string discordPath: "M101.636 22.5502C93.7418 18.9611 85.4212 16.4355 76.8848 15C75.721 17.0792 74.6532 19.2082 73.7114 21.387C64.629 20.0499 55.3728 20.0499 46.2844 21.387C45.3186 19.2082 44.2567 17.0792 43.0929 15.0246C34.5265 16.4607 26.2059 18.9857 18.3113 22.5754C2.61819 45.5237 -1.61693 67.9272 0.512688 89.984C9.69702 96.6926 19.9671 101.792 30.8851 105.06C33.3387 101.792 35.5163 98.3021 37.3759 94.6626C33.8306 93.3506 30.3933 91.7417 27.1239 89.8106C27.9937 89.1921 28.8335 88.5731 29.6494 87.8796C48.8819 96.8414 71.1138 96.8414 90.3523 87.8796C91.1681 88.5485 92.0081 89.1921 92.8479 89.8106C89.5845 91.7171 86.1411 93.3506 82.6017 94.6626C84.4854 98.3021 86.639 101.768 89.0865 105.035C100.005 101.768 110.275 96.668 119.459 89.9594H119.489C121.985 64.4118 115.23 42.2309 101.636 22.5502ZM40.0634 76.4133C34.1425 76.4133 29.2655 71.0412 29.2655 64.506C29.2655 57.9708 34.0225 52.5988 40.0634 52.5988C46.1043 52.5988 50.9515 57.9708 50.8555 64.506C50.8555 71.0412 46.0743 76.4133 40.0634 76.4133ZM79.9203 76.4133C73.9994 76.4133 69.1221 71.0412 69.1221 64.506C69.1221 57.9708 73.9034 52.5988 79.9203 52.5988C85.9312 52.5988 90.8082 57.9708 90.7122 64.506C90.7122 71.0412 85.9312 76.4133 79.9203 76.4133Z"

    // Symbols are drawn in -960..0, so they sit one icon-height down.
    Shape {
        y: root.brand ? 0 : root.size
        width: root.viewBox
        height: root.viewBox
        scale: root.size / root.viewBox
        transformOrigin: Item.TopLeft
        preferredRendererType: Shape.CurveRenderer

        ShapePath {
            strokeWidth: -1
            fillColor: root.color
            PathSvg { path: root.brand ? root.discordPath : IconPaths.path(root.symbol, !root.outline) }
        }
    }
}
