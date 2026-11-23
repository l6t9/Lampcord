import QtQuick
import QtQuick.Layouts

// Compose QrContent: a 200px white tile with the remote-auth code, the
// scanned account while it waits for approval, or a retry on failure.
Rectangle {
    id: root

    implicitWidth: 200
    implicitHeight: 200
    radius: 18
    color: "white"

    readonly property string phase: backend.qrState

    Canvas {
        id: canvas
        anchors.fill: parent
        anchors.margins: 8
        visible: root.phase === "ready"
        renderStrategy: Canvas.Cooperative

        onPaint: {
            const ctx = getContext("2d")
            ctx.reset()
            const size = backend.qrSize
            const modules = backend.qrModules
            if (size <= 0)
                return
            const quiet = 4
            const cell = Math.floor(Math.min(width, height) / (size + quiet * 2))
            const origin = (width - cell * size) / 2
            ctx.fillStyle = "black"
            for (let row = 0; row < size; row++)
                for (let col = 0; col < size; col++)
                    if (modules.charCodeAt(row * size + col) === 49)
                        ctx.fillRect(origin + col * cell, origin + row * cell, cell, cell)
        }

        Connections {
            target: backend
            function onQrModulesChanged() { canvas.requestPaint() }
        }
    }

    ColumnLayout {
        anchors.centerIn: parent
        visible: root.phase === "scanned" || root.phase === "finished"
        spacing: 0

        Avatar {
            Layout.alignment: Qt.AlignHCenter
            size: 64
            name: backend.qrUser
        }
        Item { implicitHeight: 8 }
        Text {
            Layout.alignment: Qt.AlignHCenter
            text: backend.qrUser
            color: "#1D1B20"
            font.family: Theme.font
            font.pixelSize: 14
            font.weight: Font.DemiBold
        }
        Text {
            Layout.alignment: Qt.AlignHCenter
            text: root.phase === "finished" ? "Logging in…" : "Approve on your phone"
            color: "#49454F"
            font.family: Theme.font
            font.pixelSize: 11
            font.weight: Font.Medium
        }
    }

    ColumnLayout {
        anchors.centerIn: parent
        visible: root.phase === "error"
        spacing: 0

        Text {
            Layout.alignment: Qt.AlignHCenter
            text: "Error"
            color: "#B3261E"
            font.family: Theme.font
            font.pixelSize: 14
            font.weight: Font.Medium
        }
        TextButton {
            Layout.alignment: Qt.AlignHCenter
            text: "Retry"
            onClicked: backend.startQrLogin()
        }
    }

    LoadingIndicator {
        anchors.centerIn: parent
        visible: root.phase === "idle" || root.phase === "connecting"
        size: 32
        contained: true
    }
}
