import QtCore
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import Lampcord

// The desktop EmojiPicker: Emoji/GIF/Sticker tabs, the search field, the
// grid with group headers and the category bar. GIFs and stickers need
// core support and only show a note for now.
Popup {
    id: root

    signal picked(string emoji)

    width: 400
    height: 500
    padding: 0
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent

    property int tab: 0
    readonly property var standard: JSON.parse(Emojis.groups())

    Settings {
        id: recent
        category: "emoji"
        property string frequent: "[]"
    }

    readonly property var groups: {
        const frequent = JSON.parse(recent.frequent)
        const list = frequent.length === 0 ? [] : [{
            key: "frequent", title: "Frequently Used",
            emojis: frequent.map(s => ({ n: "", s: s, u: Emojis.image(s) }))
        }]
        return list.concat(standard)
    }

    readonly property var shown: {
        const needle = search.text.trim().toLowerCase().replace(/:/g, "")
        if (needle === "")
            return groups
        const hits = []
        for (const group of standard)
            for (const emoji of group.emojis)
                if (emoji.n.includes(needle))
                    hits.push(emoji)
        return [{ key: "search", title: "Search results", emojis: hits }]
    }

    // Flattened into header and row items so the list stays virtualised.
    readonly property int columns: Math.max(1, Math.floor((grid.width + 4) / 44))
    readonly property var rows: {
        const out = []
        shown.forEach((group, index) => {
            if (group.emojis.length === 0)
                return
            out.push({ header: true, title: group.title, group: index })
            for (let i = 0; i < group.emojis.length; i += columns)
                out.push({ header: false, emojis: group.emojis.slice(i, i + columns), group: index })
        })
        return out
    }

    function choose(emoji) {
        const list = JSON.parse(recent.frequent).filter(s => s !== emoji)
        list.unshift(emoji)
        recent.frequent = JSON.stringify(list.slice(0, 18))
        picked(emoji)
    }

    function jumpTo(groupIndex) {
        const row = rows.findIndex(r => r.header && r.group === groupIndex)
        if (row >= 0)
            grid.positionViewAtIndex(row, ListView.Beginning)
    }

    onOpened: {
        search.clear()
        search.forceActiveFocus()
        grid.positionViewAtBeginning()
    }

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Theme.quick }
        NumberAnimation { property: "scale"; from: 0.92; to: 1; duration: Theme.medium; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; to: 0; duration: Theme.quick }
    }

    background: Item {
        RectangularShadow {
            anchors.fill: card
            radius: card.radius
            blur: 24
            offset.y: 6
            color: Qt.rgba(0, 0, 0, 0.35)
        }
        Rectangle {
            id: card
            anchors.fill: parent
            radius: 12
            color: Theme.surfaceContainerHigh
        }
    }

    contentItem: Column {
        // Tabs (SecondaryTabRow).
        Row {
            id: tabs
            width: parent.width
            height: 48

            Repeater {
                model: ["Emoji", "GIFs", "Stickers"]

                delegate: Item {
                    required property string modelData
                    required property int index
                    width: tabs.width / 3
                    height: tabs.height

                    Text {
                        anchors.centerIn: parent
                        text: modelData
                        color: root.tab === index ? Theme.primary : Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.letterSpacing: 0.1
                        font.weight: root.tab === index ? Font.Bold : Font.Normal
                    }
                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: 2
                        color: Theme.primary
                        visible: root.tab === index
                    }
                    HoverHandler { cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: root.tab = index }
                }
            }
        }

        // Search.
        Item {
            width: parent.width
            height: 52

            Rectangle {
                x: 12
                y: 4
                width: parent.width - 24
                height: 44
                radius: 16
                color: Theme.surfaceContainerLow

                Icon {
                    x: 14
                    anchors.verticalCenter: parent.verticalCenter
                    name: "Search"
                    size: 20
                    color: Theme.alpha(Theme.surfaceVariantText, 0.6)
                }
                TextInput {
                    id: search
                    x: 46
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 60
                    color: Theme.surfaceText
                    selectionColor: Theme.alpha(Theme.primary, 0.4)
                    selectedTextColor: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 14
                    font.letterSpacing: 0.25
                    clip: true
                    Keys.onReturnPressed: if (root.shown.length > 0 && root.shown[0].emojis.length > 0) root.choose(root.shown[0].emojis[0].s)

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: search.text === ""
                        text: ["Find the perfect emoji", "Search GIFs (Klipy)", "Search stickers"][root.tab]
                        color: Theme.alpha(Theme.surfaceVariantText, 0.6)
                        font: search.font
                    }
                }
            }
        }

        Item {
            width: parent.width
            height: root.height - tabs.height - 52 - (root.tab === 0 ? bar.height : 0)

            ListView {
                id: grid
                anchors.fill: parent
                anchors.margins: 16
                anchors.topMargin: 0
                visible: root.tab === 0
                model: root.rows
                clip: true
                spacing: 4
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; width: 6 }

                delegate: Item {
                    required property var modelData
                    width: grid.width
                    height: modelData.header ? header.implicitHeight : 40

                    Text {
                        id: header
                        visible: modelData.header
                        topPadding: 16
                        bottomPadding: 4
                        leftPadding: 4
                        text: modelData.header ? modelData.title : ""
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 11
                        font.weight: Font.Bold
                        font.letterSpacing: 0.5
                    }

                    Row {
                        visible: !modelData.header
                        spacing: 4

                        Repeater {
                            model: modelData.header ? [] : modelData.emojis

                            delegate: Item {
                                required property var modelData
                                width: 40
                                height: 40

                                Rectangle {
                                    anchors.fill: parent
                                    radius: 4
                                    color: cell.hovered ? Theme.alpha(Theme.surfaceText, 0.08) : "transparent"
                                }
                                Image {
                                    anchors.centerIn: parent
                                    width: 32
                                    height: 32
                                    source: modelData.u
                                    sourceSize: Qt.size(64, 64)
                                    asynchronous: true
                                    cache: true
                                }
                                HoverHandler { id: cell; cursorShape: Qt.PointingHandCursor }
                                TapHandler { onTapped: root.choose(modelData.s) }
                                ToolTip.visible: cell.hovered && modelData.n !== ""
                                ToolTip.text: ":" + modelData.n + ":"
                                ToolTip.delay: 600
                            }
                        }
                    }
                }
            }

            Text {
                anchors.centerIn: parent
                visible: root.tab !== 0
                text: root.tab === 1 ? "GIF search isn't available in this build yet." : "Stickers aren't available in this build yet."
                color: Theme.surfaceVariantText
                font.family: Theme.font
                font.pixelSize: 12
            }
        }

        // Category bar (EmojiServerBar).
        Column {
            id: bar
            width: parent.width
            visible: root.tab === 0

            Rectangle { width: parent.width; height: 1; color: Theme.outlineVariant }

            Row {
                leftPadding: 16
                topPadding: 4
                bottomPadding: 4
                spacing: 4

                Repeater {
                    model: root.groups

                    delegate: Column {
                        required property var modelData
                        required property int index
                        readonly property bool selected: {
                            const top = grid.indexAt(10, grid.contentY + 1)
                            return top >= 0 && search.text === "" && root.rows[top] && root.rows[top].group === index
                        }

                        Rectangle {
                            width: 36
                            height: 3
                            radius: 1.5
                            color: parent.selected ? Theme.primary : "transparent"
                        }
                        Item { width: 1; height: 4 }
                        Rectangle {
                            width: 36
                            height: 36
                            radius: 18
                            color: parent.selected ? Theme.primaryContainer : Theme.surfaceVariant

                            Icon {
                                anchors.centerIn: parent
                                size: 20
                                name: ({ frequent: "History", people: "SentimentSatisfied", nature: "EmojiNature",
                                         food: "EmojiFoodBeverage", activity: "SportsEsports", travel: "EmojiTransportation",
                                         objects: "EmojiObjects", symbols: "EmojiSymbols", flags: "Flag" })[modelData.key]
                                color: parent.parent.selected ? Theme.primary : Theme.surfaceVariantText
                            }
                            HoverHandler { cursorShape: Qt.PointingHandCursor }
                            TapHandler { onTapped: { search.clear(); root.jumpTo(index) } }
                        }
                    }
                }
            }
        }
    }
}
