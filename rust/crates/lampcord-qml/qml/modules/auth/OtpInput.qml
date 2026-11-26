import QtQuick
import QtQuick.Controls.Basic
import Lampcord

// Six-box OTP field matching the Compose login screen. The transparent input
// handles typing, paste and the mobile keyboard; the MouseArea maps taps to a
// specific digit box instead of relying on invisible text metrics.
Item {
    id: control

    property string text: ""
    property bool error: false
    property bool enabled: true
    property int length: 6

    signal accepted()

    implicitWidth: 360
    implicitHeight: 56

    function focusInput(index) {
        if (!control.enabled)
            return
        input.forceActiveFocus(Qt.MouseFocusReason)
        const target = index === undefined ? input.text.length : index
        input.cursorPosition = Math.max(0, Math.min(target, input.text.length))
    }

    function showKeyboard(index) {
        if (!control.enabled)
            return
        focusInput(index)
        // On Android the field can keep active focus after the keyboard is
        // dismissed, so forceActiveFocus() alone will not reopen it.
        Qt.inputMethod.show()
    }

    function normalize(value) {
        // Do not set TextInput.maximumLength: it truncates "123 456" before the
        // separator can be removed.
        const digits = String(value).replace(/[^0-9]/g, "").slice(0, control.length)
        if (input.text !== digits) {
            const cursor = Math.min(input.cursorPosition, digits.length)
            input.text = digits
            input.cursorPosition = cursor
        }
        if (control.text !== digits)
            control.text = digits
    }

    onTextChanged: {
        if (input.text !== control.text) {
            const cursor = Math.min(input.cursorPosition, control.text.length)
            input.text = control.text
            input.cursorPosition = cursor
        }
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
        inputMethodHints: Qt.ImhDigitsOnly
        onTextChanged: control.normalize(text)
        onAccepted: control.accepted()
        onVisibleChanged: if (visible && enabled) control.focusInput()
        Component.onCompleted: if (visible && enabled) control.focusInput()
    }

    MouseArea {
        anchors.fill: parent
        z: 2
        enabled: control.enabled
        onClicked: {
            const box = Math.min(control.length - 1,
                                 Math.max(0, Math.floor(mouseX / width * control.length)))
            control.showKeyboard(box)
        }
    }
}
