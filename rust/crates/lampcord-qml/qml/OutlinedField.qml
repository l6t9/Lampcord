import QtQuick
import QtQuick.Controls.Basic

// M3 outlined text field with a floating label.
TextField {
    id: control

    property string label
    property bool error: false
    property string trailingIcon
    signal trailingClicked()

    implicitHeight: 56
    leftPadding: 16
    rightPadding: trailingIcon !== "" ? 52 : 16
    topPadding: 16
    bottomPadding: 16
    font.family: Theme.font
    font.pixelSize: 16
    font.letterSpacing: 0.5
    color: Theme.surfaceText
    selectionColor: Theme.alpha(Theme.primary, 0.4)
    selectedTextColor: Theme.surfaceText
    placeholderText: ""
    verticalAlignment: TextInput.AlignVCenter

    readonly property bool raised: activeFocus || text.length > 0
    readonly property color accent: error ? Theme.error : activeFocus ? Theme.primary : Theme.outline

    background: Item {
        Rectangle {
            anchors.fill: parent
            radius: 4
            color: "transparent"
            border.width: control.activeFocus || control.error ? 2 : 1
            border.color: control.accent
        }

        // Notch behind the raised label. It covers well past the 2px border
        // so fractional display scaling can't leave an antialiased line.
        Rectangle {
            x: 12
            y: -4
            width: control.raised ? labelText.implicitWidth * 0.75 + 8 : 0
            height: 8
            color: Theme.background
        }

        Text {
            id: labelText
            text: control.label
            x: 16
            font.family: Theme.font
            font.pixelSize: 16
            font.letterSpacing: 0.5
            color: control.error ? Theme.error : control.activeFocus ? Theme.primary : Theme.surfaceVariantText
            transformOrigin: Item.Left
            scale: control.raised ? 0.75 : 1
            y: control.raised ? -height / 2 : (control.height - height) / 2

            Behavior on y { NumberAnimation { duration: Theme.quick; easing.type: Easing.OutCubic } }
            Behavior on scale { NumberAnimation { duration: Theme.quick; easing.type: Easing.OutCubic } }
        }
    }

    IconButton {
        visible: control.trailingIcon !== ""
        anchors.right: parent.right
        anchors.rightMargin: 4
        anchors.verticalCenter: parent.verticalCenter
        glyph: control.trailingIcon
        onClicked: control.trailingClicked()
    }
}
