import QtQuick
import Lampcord

// The Compose client's Material Symbols, by their Compose name: "Send" is
// Icons.Filled.Send, "Rounded.Send" is Icons.Rounded.Send. Paths come from
// IconPaths (a 960 grid with y in -960..0). "discord" is the brand mark.
// Rendered via Qt's SVG rasterizer (data:image/svg+xml) for crisp antialiasing
// without Qt Shape CurveRenderer artifacts.
Item {
    id: root

    property string name
    property color color: Theme.surfaceText
    property real size: 24

    implicitWidth: size
    implicitHeight: size

    readonly property bool outline: name.startsWith("Rounded.")
    readonly property string symbol: outline ? name.slice(8) : name
    readonly property bool brand: name === "discord" || name === "github"
    readonly property string discordPath: "M101.636 22.5502C93.7418 18.9611 85.4212 16.4355 76.8848 15C75.721 17.0792 74.6532 19.2082 73.7114 21.387C64.629 20.0499 55.3728 20.0499 46.2844 21.387C45.3186 19.2082 44.2567 17.0792 43.0929 15.0246C34.5265 16.4607 26.2059 18.9857 18.3113 22.5754C2.61819 45.5237 -1.61693 67.9272 0.512688 89.984C9.69702 96.6926 19.9671 101.792 30.8851 105.06C33.3387 101.792 35.5163 98.3021 37.3759 94.6626C33.8306 93.3506 30.3933 91.7417 27.1239 89.8106C27.9937 89.1921 28.8335 88.5731 29.6494 87.8796C48.8819 96.8414 71.1138 96.8414 90.3523 87.8796C91.1681 88.5485 92.0081 89.1921 92.8479 89.8106C89.5845 91.7171 86.1411 93.3506 82.6017 94.6626C84.4854 98.3021 86.639 101.768 89.0865 105.035C100.005 101.768 110.275 96.668 119.459 89.9594H119.489C121.985 64.4118 115.23 42.2309 101.636 22.5502ZM40.0634 76.4133C34.1425 76.4133 29.2655 71.0412 29.2655 64.506C29.2655 57.9708 34.0225 52.5988 40.0634 52.5988C46.1043 52.5988 50.9515 57.9708 50.8555 64.506C50.8555 71.0412 46.0743 76.4133 40.0634 76.4133ZM79.9203 76.4133C73.9994 76.4133 69.1221 71.0412 69.1221 64.506C69.1221 57.9708 73.9034 52.5988 79.9203 52.5988C85.9312 52.5988 90.8082 57.9708 90.7122 64.506C90.7122 71.0412 85.9312 76.4133 79.9203 76.4133Z"
    readonly property string githubPath: "M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0 1 12 6.844a9.59 9.59 0 0 1 2.504.337c1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0 0 22 12.017C22 6.484 17.522 2 12 2Z"
    readonly property string pathData: name === "discord" ? discordPath : name === "github" ? githubPath : IconPaths.path(symbol, !outline)
    readonly property string viewBox: name === "discord" ? "0 0 120 120" : name === "github" ? "0 0 24 24" : "0 -960 960 960"
    readonly property string fillHex: {
        const r = Math.round(root.color.r * 255).toString(16).padStart(2, "0")
        const g = Math.round(root.color.g * 255).toString(16).padStart(2, "0")
        const b = Math.round(root.color.b * 255).toString(16).padStart(2, "0")
        return "%23" + r + g + b
    }

    Image {
        anchors.fill: parent
        visible: root.pathData !== ""
        opacity: root.color.a
        smooth: true
        fillMode: Image.PreserveAspectFit
        sourceSize: Qt.size(Math.max(16, Math.ceil(root.size * 2)), Math.max(16, Math.ceil(root.size * 2)))
        source: root.pathData === "" ? "" : (
            "data:image/svg+xml;utf8,"
            + "<svg xmlns='http://www.w3.org/2000/svg' viewBox='"
            + root.viewBox
            + "'><path fill='" + root.fillHex + "' d='" + root.pathData + "'/></svg>"
        )
    }
}
