import QtQuick
import QtQuick.Controls.Basic

// Six-box OTP field matching the Compose login screen. A transparent input
// sits over the boxes so mobile keyboards and paste still work normally.
Item {
    id: control

    property string text: ""
    property bool error: false
    property bool enabled: true
    property int length: 6

    signal accepted()

    implicitWidth: 360
    implicitHeight: 56

    function setText(value) {
        const digits = String(value).replace(/[^0-9]/g, "").slice(0, control.length)
        if (digits !== control.text)
            control.text = digits
        if (input.text !== digits)
            input.text = digits
    }

    onTextChanged: {
        if (input && input.text !== control.text)
            input.text = control.text
    }

    Row {
        id: boxes
        anchors.fill: parent
        spacing: 6

        Repeater {
            model: control.length

            Rectangle {
                required property int index
                readonly property int boxIndex: index
                readonly property bool current: enabled && boxIndex === control.text.length

                width: (boxes.width - (control.length - 1) * boxes.spacing) / control.length
                height: boxes.height
                radius: 4
                color: "transparent"
                border.width: control.error || current ? 2 : 1
                border.color: control.error ? Theme.error
                                            : current ? Theme.primary
                                                      : Theme.outline

                Text {
                    anchors.centerIn: parent
                    text: boxIndex < control.text.length ? control.text.charAt(boxIndex) : ""
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 24
                    font.bold: true
                }
            }
        }
    }

    TextInput {
        id: input
        anchors.fill: parent
        opacity: 0
        enabled: control.enabled
        maximumLength: control.length
        inputMethodHints: Qt.ImhDigitsOnly
        onTextChanged: control.setText(text)
        onAccepted: control.accepted()
        onVisibleChanged: if (visible && enabled) forceActiveFocus()
        Component.onCompleted: if (visible && enabled) forceActiveFocus()
    }
}
