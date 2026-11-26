import QtQuick
import QtQuick.Controls.Basic
import Lampcord

Column {
    id: root
    spacing: 16

    function setBool(prefKey, cfgKey, value) {
        Theme.prefs[prefKey] = value
        backend.setConfig(cfgKey, value ? "true" : "false")
    }

    SettingsGroupTitle { text: "Themes" }

    SettingsActionRow {
        icon: "Palette"
        title: "Themer"
        subtitle: "Manage and edit custom themes"
        first: true; last: true
    }

    SettingsGroupTitle { text: "Wallpaper" }

    Rectangle {
        width: parent.width
        height: wallCol.implicitHeight + 32
        radius: 20
        color: Theme.surfaceContainer

        Column {
            id: wallCol
            anchors.fill: parent
            anchors.margins: 16
            spacing: 12

            Text {
                text: "Wallpaper & Colors"
                color: Theme.surfaceText
                font.family: Theme.font
                font.pixelSize: 15
                font.weight: Font.Medium
            }

            Rectangle {
                width: parent.width
                height: 112
                radius: 16
                color: Theme.surfaceContainerHigh

                Column {
                    anchors.centerIn: parent
                    spacing: 6
                    Icon {
                        anchors.horizontalCenter: parent.horizontalCenter
                        name: "Image"
                        size: 28
                        color: Theme.surfaceVariantText
                    }
                    Text {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: "No background set"
                        color: Theme.surfaceVariantText
                        font.family: Theme.font
                        font.pixelSize: 12
                    }
                }
            }

            Row {
                width: parent.width
                spacing: 4
                Rectangle {
                    width: (parent.width - 4) / 2
                    height: 40
                    topLeftRadius: 20; bottomLeftRadius: 20; topRightRadius: 8; bottomRightRadius: 8
                    color: Theme.prefs.themeMode === "system" ? Theme.primaryContainer : Theme.surfaceContainerLow
                    Row {
                        anchors.centerIn: parent
                        spacing: 6
                        Icon { anchors.verticalCenter: parent.verticalCenter; name: "Refresh"; size: 16; color: Theme.prefs.themeMode === "system" ? Theme.primaryContainerText : Theme.surfaceVariantText }
                        Text { anchors.verticalCenter: parent.verticalCenter; text: "Auto"; color: Theme.prefs.themeMode === "system" ? Theme.primaryContainerText : Theme.surfaceText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.DemiBold }
                    }
                    HoverHandler { cursorShape: Qt.PointingHandCursor }
                    TapHandler {
                        onTapped: {
                            Theme.prefs.themeMode = "system"
                            backend.setConfig("theme_mode", "AUTO")
                        }
                    }
                }
                Rectangle {
                    width: (parent.width - 4) / 2
                    height: 40
                    topLeftRadius: 8; bottomLeftRadius: 8; topRightRadius: 20; bottomRightRadius: 20
                    color: Theme.prefs.pureBlack ? Theme.primaryContainer : Theme.surfaceContainerLow
                    Row {
                        anchors.centerIn: parent
                        spacing: 6
                        Icon { anchors.verticalCenter: parent.verticalCenter; name: "DarkMode"; size: 16; color: Theme.prefs.pureBlack ? Theme.primaryContainerText : Theme.surfaceVariantText }
                        Text { anchors.verticalCenter: parent.verticalCenter; text: "Pure Black"; color: Theme.prefs.pureBlack ? Theme.primaryContainerText : Theme.surfaceText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.DemiBold }
                    }
                    HoverHandler { cursorShape: Qt.PointingHandCursor }
                    TapHandler {
                        onTapped: root.setBool("pureBlack", "pure_black", !Theme.prefs.pureBlack)
                    }
                }
            }

            Row {
                width: parent.width
                spacing: 4
                Repeater {
                    model: [
                        { key: "light", cfg: "LIGHT", label: "Light", icon: "LightMode", left: true },
                        { key: "dark", cfg: "DARK", label: "Dark", icon: "DarkMode", left: false }
                    ]
                    delegate: Rectangle {
                        required property var modelData
                        readonly property bool active: (modelData.key === "dark" && Theme.dark) || (modelData.key === "light" && !Theme.dark)
                        width: (parent.width - 4) / 2
                        height: 88
                        topLeftRadius: modelData.left ? 44 : 8
                        bottomLeftRadius: modelData.left ? 44 : 8
                        topRightRadius: modelData.left ? 8 : 44
                        bottomRightRadius: modelData.left ? 8 : 44
                        color: active ? Theme.primaryContainer : Theme.surfaceContainerLow
                        Behavior on color { ColorAnimation { duration: Theme.quick } }

                        Column {
                            anchors.centerIn: parent
                            spacing: 6
                            Icon {
                                anchors.horizontalCenter: parent.horizontalCenter
                                name: modelData.icon
                                size: 24
                                color: active ? Theme.primaryContainerText : Theme.surfaceVariantText
                            }
                            Text {
                                anchors.horizontalCenter: parent.horizontalCenter
                                text: modelData.label
                                color: active ? Theme.primaryContainerText : Theme.surfaceText
                                font.family: Theme.font
                                font.pixelSize: 13
                                font.weight: active ? Font.DemiBold : Font.Medium
                            }
                        }
                        HoverHandler { cursorShape: Qt.PointingHandCursor }
                        TapHandler {
                            onTapped: {
                                Theme.prefs.themeMode = modelData.key
                                backend.setConfig("theme_mode", modelData.cfg)
                            }
                        }
                    }
                }
            }

            Rectangle {
                width: parent.width
                height: 40
                radius: 20
                color: Theme.primary
                Row {
                    anchors.centerIn: parent
                    spacing: 8
                    Icon { anchors.verticalCenter: parent.verticalCenter; name: "Image"; size: 17; color: Theme.primaryText }
                    Text { anchors.verticalCenter: parent.verticalCenter; text: "Choose background"; color: Theme.primaryText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.DemiBold }
                }
            }
        }
    }

    SettingsGroupTitle { text: "Color palette" }

    Column {
        width: parent.width
        spacing: 2

        Rectangle {
            width: parent.width
            height: 92
            topLeftRadius: 20; topRightRadius: 20; bottomLeftRadius: 4; bottomRightRadius: 4
            color: Theme.surfaceContainer

            Column {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 10

                Text {
                    text: "Visual Style"
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 15
                    font.weight: Font.Medium
                }

                Rectangle {
                    id: styleDropBtn
                    width: styleDropRow.implicitWidth + 24
                    height: 36
                    radius: 10
                    color: Theme.surfaceContainerLow
                    border.width: 1
                    border.color: Theme.outlineVariant

                    Row {
                        id: styleDropRow
                        anchors.centerIn: parent
                        spacing: 8
                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: ({
                                "tonal_spot": "Tonal spot",
                                "expressive": "Expressive",
                                "vibrant": "Vibrant",
                                "fidelity": "Fidelity",
                                "neutral": "Neutral",
                                "monochrome": "Monochrome",
                                "fruit_salad": "Fruit salad",
                                "rainbow": "Rainbow"
                            })[Theme.prefs.paletteVariant] || "Tonal spot"
                            color: Theme.surfaceText
                            font.family: Theme.font
                            font.pixelSize: 13
                        }
                        Icon {
                            anchors.verticalCenter: parent.verticalCenter
                            name: "KeyboardArrowDown"
                            size: 16
                            color: Theme.surfaceVariantText
                        }
                    }
                    HoverHandler { cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: styleMenu.visible ? styleMenu.close() : styleMenu.open() }

                    LampMenu {
                        id: styleMenu
                        y: styleDropBtn.height + 4
                        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent
                        Repeater {
                            model: [
                                { key: "tonal_spot", cfg: "TONAL_SPOT", label: "Tonal spot" },
                                { key: "expressive", cfg: "EXPRESSIVE", label: "Expressive" },
                                { key: "vibrant", cfg: "VIBRANT", label: "Vibrant" },
                                { key: "fidelity", cfg: "FIDELITY", label: "Fidelity" },
                                { key: "neutral", cfg: "NEUTRAL", label: "Neutral" },
                                { key: "monochrome", cfg: "MONOCHROME", label: "Monochrome" },
                                { key: "fruit_salad", cfg: "FRUIT_SALAD", label: "Fruit salad" },
                                { key: "rainbow", cfg: "RAINBOW", label: "Rainbow" }
                            ]
                            delegate: LampMenuItem {
                                required property var modelData
                                text: modelData.label
                                onTriggered: {
                                    Theme.prefs.paletteVariant = modelData.key
                                    backend.setConfig("theme_palette_style", modelData.cfg)
                                    Theme.applyPalette()
                                }
                            }
                        }
                    }
                }
            }
        }

        Rectangle {
            width: parent.width
            height: srcCol.implicitHeight + 32
            radius: 4
            color: Theme.surfaceContainer

            Column {
                id: srcCol
                anchors.fill: parent
                anchors.margins: 16
                spacing: 12

                Text {
                    text: "Source"
                    color: Theme.surfaceText
                    font.family: Theme.font
                    font.pixelSize: 15
                    font.weight: Font.Medium
                }

                Row {
                    width: parent.width
                    spacing: 4
                    Rectangle {
                        width: (parent.width - 4) / 2
                        height: 40
                        topLeftRadius: 20; bottomLeftRadius: 20; topRightRadius: 8; bottomRightRadius: 8
                        color: Theme.prefs.matugenSync ? Theme.primaryContainer : Theme.surfaceContainerLow
                        Row {
                            anchors.centerIn: parent
                            spacing: 6
                            Icon { anchors.verticalCenter: parent.verticalCenter; name: "AutoAwesome"; size: 16; color: Theme.prefs.matugenSync ? Theme.primaryContainerText : Theme.surfaceVariantText }
                            Text { anchors.verticalCenter: parent.verticalCenter; text: "Dynamic"; color: Theme.prefs.matugenSync ? Theme.primaryContainerText : Theme.surfaceText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.DemiBold }
                        }
                        HoverHandler { cursorShape: Qt.PointingHandCursor }
                        TapHandler {
                            onTapped: {
                                Theme.prefs.customSeed = ""
                                root.setBool("matugenSync", "material_you", true)
                                Theme.applyPalette()
                            }
                        }
                    }
                    Rectangle {
                        width: (parent.width - 4) / 2
                        height: 40
                        topLeftRadius: 8; bottomLeftRadius: 8; topRightRadius: 20; bottomRightRadius: 20
                        color: !Theme.prefs.matugenSync ? Theme.primaryContainer : Theme.surfaceContainerLow
                        Row {
                            anchors.centerIn: parent
                            spacing: 6
                            Rectangle { anchors.verticalCenter: parent.verticalCenter; width: 14; height: 14; radius: 4; color: Theme.prefs.customSeed || Theme.primary }
                            Text { anchors.verticalCenter: parent.verticalCenter; text: "Custom"; color: !Theme.prefs.matugenSync ? Theme.primaryContainerText : Theme.surfaceText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.DemiBold }
                        }
                        HoverHandler { cursorShape: Qt.PointingHandCursor }
                        TapHandler {
                            onTapped: {
                                if (Theme.prefs.customSeed === "")
                                    Theme.prefs.customSeed = "#6750A4"
                                root.setBool("matugenSync", "material_you", false)
                                Theme.applyPalette()
                            }
                        }
                    }
                }

                Flow {
                    visible: !Theme.prefs.matugenSync
                    width: parent.width
                    spacing: 8
                    Repeater {
                        model: ["#6750A4", "#5865F2", "#006A6A", "#2E7D32", "#C2185B", "#D84315", "#1565C0", "#F9A825"]
                        delegate: Rectangle {
                            required property string modelData
                            width: 32; height: 32; radius: 16
                            color: modelData
                            border.width: Theme.prefs.customSeed.toLowerCase() === modelData.toLowerCase() ? 3 : 1
                            border.color: Theme.prefs.customSeed.toLowerCase() === modelData.toLowerCase() ? Theme.surfaceText : Theme.outlineVariant
                            HoverHandler { cursorShape: Qt.PointingHandCursor }
                            TapHandler {
                                onTapped: {
                                    Theme.prefs.customSeed = modelData
                                    backend.setConfig("accent_color", modelData)
                                    Theme.applyPalette()
                                }
                            }
                        }
                    }
                }
            }
        }

        SettingsSwitchRow {
            title: "Sync across clients"
            subtitle: "Sync your theme selection across all Discord clients."
            checked: Theme.prefs.syncAppearance
            first: false; last: true
            onToggled: value => root.setBool("syncAppearance", "sync_appearance", value)
        }
    }

    SettingsGroupTitle { text: "Animation" }

    Column {
        width: parent.width
        spacing: 2

        Rectangle {
            width: parent.width
            height: 96
            topLeftRadius: 20; topRightRadius: 20; bottomLeftRadius: 4; bottomRightRadius: 4
            color: Theme.surfaceContainer

            Column {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 10
                Text { text: "Panel Animation"; color: Theme.surfaceText; font.family: Theme.font; font.pixelSize: 15; font.weight: Font.Medium }
                Row {
                    width: parent.width
                    spacing: 4
                    Repeater {
                        model: [
                            { key: "minimal", cfg: "MINIMAL", label: "Minimal", icon: "Speed", left: true },
                            { key: "expressive", cfg: "EXPRESSIVE", label: "Expressive", icon: "AutoAwesome", left: false }
                        ]
                        delegate: Rectangle {
                            required property var modelData
                            readonly property bool active: Theme.prefs.panelAnimation === modelData.key
                            width: (parent.width - 4) / 2
                            height: 40
                            topLeftRadius: modelData.left ? 20 : 8; bottomLeftRadius: modelData.left ? 20 : 8
                            topRightRadius: modelData.left ? 8 : 20; bottomRightRadius: modelData.left ? 8 : 20
                            color: active ? Theme.primaryContainer : Theme.surfaceContainerLow
                            Row {
                                anchors.centerIn: parent
                                spacing: 6
                                Icon { anchors.verticalCenter: parent.verticalCenter; name: modelData.icon; size: 16; color: active ? Theme.primaryContainerText : Theme.surfaceVariantText }
                                Text { anchors.verticalCenter: parent.verticalCenter; text: modelData.label; color: active ? Theme.primaryContainerText : Theme.surfaceText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.DemiBold }
                            }
                            HoverHandler { cursorShape: Qt.PointingHandCursor }
                            TapHandler {
                                onTapped: {
                                    Theme.prefs.panelAnimation = modelData.key
                                    backend.setConfig("panel_animation", modelData.cfg)
                                }
                            }
                        }
                    }
                }
            }
        }

        Rectangle {
            width: parent.width
            height: 96
            topLeftRadius: 4; topRightRadius: 4; bottomLeftRadius: 20; bottomRightRadius: 20
            color: Theme.surfaceContainer

            Column {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 10
                Text { text: "Panel Layout"; color: Theme.surfaceText; font.family: Theme.font; font.pixelSize: 15; font.weight: Font.Medium }
                Row {
                    width: parent.width
                    spacing: 4
                    Repeater {
                        model: [
                            { key: "center", cfg: "CENTER", label: "Center Panel", left: true },
                            { key: "overlapping", cfg: "OVERLAPPING", label: "Overlapping", left: false }
                        ]
                        delegate: Rectangle {
                            required property var modelData
                            readonly property bool active: Theme.prefs.panelType === modelData.key
                            width: (parent.width - 4) / 2
                            height: 40
                            topLeftRadius: modelData.left ? 20 : 8; bottomLeftRadius: modelData.left ? 20 : 8
                            topRightRadius: modelData.left ? 8 : 20; bottomRightRadius: modelData.left ? 8 : 20
                            color: active ? Theme.primaryContainer : Theme.surfaceContainerLow
                            Text { anchors.centerIn: parent; text: modelData.label; color: active ? Theme.primaryContainerText : Theme.surfaceText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.DemiBold }
                            HoverHandler { cursorShape: Qt.PointingHandCursor }
                            TapHandler {
                                onTapped: {
                                    Theme.prefs.panelType = modelData.key
                                    backend.setConfig("panel_type", modelData.cfg)
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    SettingsGroupTitle { text: "Display" }

    Rectangle {
        width: parent.width
        height: 112
        radius: 20
        color: Theme.surfaceContainer

        Column {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 8
            Text { text: "UI Density"; color: Theme.surfaceText; font.family: Theme.font; font.pixelSize: 15; font.weight: Font.Medium }
            Text { text: "Adjust the space between server, channel, and member lists."; color: Theme.surfaceVariantText; font.family: Theme.font; font.pixelSize: 12 }
            Row {
                width: parent.width
                spacing: 4
                Repeater {
                    model: [
                        { key: "compact", cfg: "COMPACT", label: "Compact" },
                        { key: "default", cfg: "DEFAULT", label: "Default" },
                        { key: "spacious", cfg: "SPACIOUS", label: "Spacious" }
                    ]
                    delegate: Rectangle {
                        required property var modelData
                        required property int index
                        readonly property bool active: Theme.prefs.messageSpacingMode === modelData.key
                        width: (parent.width - 8) / 3
                        height: 36
                        topLeftRadius: index === 0 ? 18 : 6; bottomLeftRadius: index === 0 ? 18 : 6
                        topRightRadius: index === 2 ? 18 : 6; bottomRightRadius: index === 2 ? 18 : 6
                        color: active ? Theme.primaryContainer : Theme.surfaceContainerLow
                        Text { anchors.centerIn: parent; text: modelData.label; color: active ? Theme.primaryContainerText : Theme.surfaceText; font.family: Theme.font; font.pixelSize: 13; font.weight: Font.DemiBold }
                        HoverHandler { cursorShape: Qt.PointingHandCursor }
                        TapHandler {
                            onTapped: {
                                Theme.prefs.messageSpacingMode = modelData.key
                                backend.setConfig("message_spacing_mode", modelData.cfg)
                            }
                        }
                    }
                }
            }
        }
    }

    SettingsGroupTitle { text: "Typography" }

    Column {
        width: parent.width
        spacing: 2

        Rectangle {
            width: parent.width
            height: 96
            topLeftRadius: 20; topRightRadius: 20; bottomLeftRadius: 4; bottomRightRadius: 4
            color: Theme.surfaceContainer

            Column {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 10
                Text { text: "App Font"; color: Theme.surfaceText; font.family: Theme.font; font.pixelSize: 15; font.weight: Font.Medium }
                Row {
                    width: parent.width
                    spacing: 4
                    Repeater {
                        model: [
                            { key: "system", cfg: "SYSTEM", label: "System" },
                            { key: "inter", cfg: "INTER", label: "Inter" },
                            { key: "maple_mono", cfg: "MAPLE_MONO", label: "Maple Mono" },
                            { key: "custom", cfg: "CUSTOM", label: "Custom" }
                        ]
                        delegate: Rectangle {
                            required property var modelData
                            required property int index
                            readonly property bool active: Theme.prefs.fontOption === modelData.key
                            width: (parent.width - 12) / 4
                            height: 36
                            topLeftRadius: index === 0 ? 18 : 6; bottomLeftRadius: index === 0 ? 18 : 6
                            topRightRadius: index === 3 ? 18 : 6; bottomRightRadius: index === 3 ? 18 : 6
                            color: active ? Theme.primaryContainer : Theme.surfaceContainerLow
                            Text { anchors.centerIn: parent; text: modelData.label; color: active ? Theme.primaryContainerText : Theme.surfaceText; font.family: Theme.font; font.pixelSize: 12; font.weight: Font.DemiBold }
                            HoverHandler { cursorShape: Qt.PointingHandCursor }
                            TapHandler {
                                onTapped: {
                                    Theme.prefs.fontOption = modelData.key
                                    backend.setConfig("app_font", modelData.cfg)
                                }
                            }
                        }
                    }
                }
            }
        }

        SettingsSliderRow {
            title: "Font Scale: " + Math.round(Theme.prefs.fontScale * 100) + "%"
            label: ""
            from: 0.75; to: 1.5; value: Theme.prefs.fontScale
            first: false; last: true
            onMoved: v => {
                Theme.prefs.fontScale = v
                backend.setConfig("font_scale", v.toFixed(2))
            }
        }
    }
}
