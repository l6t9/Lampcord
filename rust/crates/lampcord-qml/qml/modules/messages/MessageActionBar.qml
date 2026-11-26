import QtQuick
import Lampcord

Row {
    id: root
    property bool mine: false
    signal reactClicked()
    signal replyClicked()
    signal forwardClicked()
    signal editClicked()
    signal createThreadClicked()
    signal moreClicked(Item anchorItem)

    spacing: 2

    ToolbarBtn {
        leading: true
        glyph: "AddReaction"
        tip: "Add Reaction"
        onClicked: root.reactClicked()
    }
    ToolbarBtn {
        glyph: "Rounded.Reply"
        tip: "Reply"
        onClicked: root.replyClicked()
    }
    ToolbarBtn {
        glyph: "Forward"
        tip: "Forward"
        onClicked: root.forwardClicked()
    }
    ToolbarBtn {
        visible: root.mine
        glyph: "Edit"
        tip: "Edit"
        onClicked: root.editClicked()
    }
    ToolbarBtn {
        glyph: "Tag"
        tip: "Create Thread"
        onClicked: root.createThreadClicked()
    }
    ToolbarBtn {
        id: moreBtn
        trailing: true
        glyph: "MoreHoriz"
        tip: "More"
        onClicked: root.moreClicked(moreBtn)
    }

    component ToolbarBtn: Rectangle {
        id: tbtn
        property bool leading: false
        property bool trailing: false
        property string glyph
        property string tip
        signal clicked()

        width: 36
        height: 32
        topLeftRadius: leading ? 16 : 4
        bottomLeftRadius: leading ? 16 : 4
        topRightRadius: trailing ? 16 : 4
        bottomRightRadius: trailing ? 16 : 4
        color: tbtnHover.hovered ? Theme.primaryContainer : Theme.secondaryContainer
        Behavior on color { ColorAnimation { duration: Theme.quick } }

        Icon {
            anchors.centerIn: parent
            name: tbtn.glyph
            size: 18
            color: tbtnHover.hovered ? Theme.primaryContainerText : Theme.secondaryContainerText
        }

        HoverHandler { id: tbtnHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: tbtn.clicked() }
        RichTooltip {
            anchor: tbtn
            placement: "above"
            visible: tbtnHover.hovered && tbtn.tip !== ""
            text: tbtn.tip
        }
    }
}
