import QtQuick
import Lampcord

Flow {
    id: root
    property var items: []
    property bool canReact: false
    signal toggleReaction(string key, bool hasMe)
    signal addReaction()

    visible: items.length > 0 && Theme.prefs.showReactions
    spacing: 6
    topPadding: 4

    Repeater {
        model: root.items

        delegate: Rectangle {
            required property var modelData
            height: 26
            width: reactRow.implicitWidth + 16
            radius: 13
            color: modelData.me ? Theme.alpha(Theme.primary, 0.22) : Theme.surfaceContainerHighest
            border.width: 1
            border.color: modelData.me ? Theme.primary : Theme.alpha(Theme.outlineVariant, 0.45)

            Row {
                id: reactRow
                anchors.centerIn: parent
                spacing: 5

                Image {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: modelData.icon !== ""
                    width: 16
                    height: 16
                    source: modelData.icon
                    fillMode: Image.PreserveAspectFit
                    asynchronous: true
                    smooth: true
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: modelData.icon === ""
                    text: modelData.name
                    color: Theme.surfaceText
                    font.pixelSize: 14
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: String(modelData.count)
                    color: modelData.me ? Theme.primary : Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 12
                    font.weight: Font.DemiBold
                }
            }

            HoverHandler { cursorShape: Qt.PointingHandCursor }
            TapHandler { onTapped: root.toggleReaction(modelData.key, !!modelData.me) }
        }
    }

    Rectangle {
        visible: root.canReact
        width: 28
        height: 26
        radius: 13
        color: addReactHover.hovered ? Theme.surfaceContainerHighest : Theme.alpha(Theme.surfaceContainerHighest, 0.7)
        border.width: 1
        border.color: Theme.alpha(Theme.outlineVariant, 0.35)

        Icon {
            anchors.centerIn: parent
            name: "AddReaction"
            size: 15
            color: addReactHover.hovered ? Theme.surfaceText : Theme.surfaceVariantText
        }
        HoverHandler { id: addReactHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: root.addReaction() }
        RichTooltip { anchor: parent; placement: "above"; visible: addReactHover.hovered; text: "Add Reaction" }
    }
}
