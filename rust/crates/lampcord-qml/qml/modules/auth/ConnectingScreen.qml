import QtQuick
import QtQuick.Layouts
import Lampcord

Rectangle {
    color: Theme.background

    ColumnLayout {
        anchors.centerIn: parent
        spacing: 16

        LoadingIndicator {
            Layout.alignment: Qt.AlignHCenter
            size: 48
            contained: true
        }
        Text {
            Layout.alignment: Qt.AlignHCenter
            text: backend.connection === "error" ? "Couldn't connect" : "Connecting…"
            color: Theme.surfaceText
            font.family: Theme.font
            font.pixelSize: 16
            font.weight: Font.DemiBold
        }
        Text {
            Layout.alignment: Qt.AlignHCenter
            Layout.maximumWidth: 420
            visible: backend.errorText !== ""
            text: backend.errorText
            color: Theme.surfaceVariantText
            font.family: Theme.font
            font.pixelSize: 14
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.Wrap
        }
        TextButton {
            Layout.alignment: Qt.AlignHCenter
            visible: backend.connection === "error"
            text: "Log out"
            onClicked: backend.logout()
        }
    }
}
