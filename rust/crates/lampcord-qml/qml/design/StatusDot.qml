import QtQuick
import Lampcord

// StatusIndicator: Discord's status glyphs (StatusIcons in the Compose
// client) on an 8px grid, inset by an 18% ring in the background colour.
// Mobile is a taller phone shape.
Rectangle {
    id: root

    property string status: "offline"
    property color ring: Theme.surface
    property real size: 14

    readonly property bool mobile: status === "mobile"
    readonly property real inset: size * 0.18
    readonly property var glyph: ({
        online: ["M4,4m-4,0a4,4 0,1 1,8 0a4,4 0,1 1,-8 0", "%233BA55C"],
        idle: ["M2.00009,5C3.65695,5 5.00009,3.65685 5.00009,2C5.00009,1.23515 4.71386,0.53714 4.24268,0.00726C6.33882,0.1327 8.00001,1.87237 8.00001,4.00002C8.00001,6.20916 6.20914,8.00002 4.00001,8.00002C1.87228,8.00002 0.13256,6.33872 0.00723,4.24248C0.53713,4.71373 1.23518,5 2.00009,5Z", "%23FAA61A"],
        dnd: ["M4,8C6.20914,8 8,6.20914 8,4C8,1.79086 6.20914,0 4,0C1.79086,0 0,1.79086 0,4C0,6.20914 1.79086,8 4,8ZM2,3C1.44772,3 1,3.44772 1,4C1,4.55228 1.44772,5 2,5H6C6.55228,5 7,4.55228 7,4C7,3.44772 6.55228,3 6,3H2Z", "%23ED4245"],
        streaming: ["M4,8C6.20914,8 8,6.20914 8,4C8,1.79086 6.20914,0 4,0C1.79086,0 0,1.79086 0,4C0,6.20914 1.79086,8 4,8ZM3,5.73205L6,4L3,2.26795V5.73205Z", "%23593A93"],
        mobile: ["M0,1.5C0,0.6716 0.6716,0 1.5,0H6.5C7.3284,0 8,0.6716 8,1.5V10.5C8,11.3284 7.3284,12 6.5,12H1.5C0.6716,12 0,11.3284 0,10.5V1.5ZM5,10C5,10.5523 4.5523,11 4,11C3.4477,11 3,10.5523 3,10C3,9.4477 3.4477,9 4,9C4.5523,9 5,9.4477 5,10ZM7,2H1V8H7V2Z", "%233BA55C"]
    })[status] || null

    width: size
    height: mobile ? size * 1.5 : size
    radius: mobile ? size * 0.2 : size / 2
    color: ring

    Image {
        anchors.fill: parent
        anchors.margins: root.inset
        smooth: true
        fillMode: Image.PreserveAspectFit
        sourceSize: Qt.size(Math.max(16, Math.ceil(width * 2)), Math.max(16, Math.ceil(height * 2)))
        source: root.glyph
            ? ("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 8 "
               + (root.mobile ? "12" : "8")
               + "'><path fill='" + root.glyph[1] + "' d='" + root.glyph[0] + "'/></svg>")
            : "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 8 8'><circle cx='4' cy='4' r='3' fill='none' stroke='%23747F8D' stroke-width='2'/></svg>"
    }
}
