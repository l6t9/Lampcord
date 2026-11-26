import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import Lampcord

Popup {
    id: root

    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    padding: 4
    width: Math.min(320, parent ? parent.width - 24 : 320)

    property string targetUserId: ""
    property string fallbackName: ""
    property string fallbackAvatar: ""
    property string fallbackDecoration: ""
    property string fallbackStatus: "offline"
    property string fallbackActivity: ""
    property real desiredX: -1
    property real desiredY: -1
    property bool openRight: true
    property real targetX: 0

    readonly property var rawProfile: {
        try {
            const parsed = JSON.parse(backend.profileJson)
            return parsed && String(parsed.user_id) === root.targetUserId ? parsed : null
        } catch (e) {
            return null
        }
    }

    function luminance(c) {
        const ch = v => {
            const n = v / 255.0
            return n <= 0.03928 ? n / 12.92 : Math.pow((n + 0.055) / 1.055, 2.4)
        }
        return 0.2126 * ch((c >> 16) & 255) + 0.7152 * ch((c >> 8) & 255) + 0.0722 * ch(c & 255)
    }

    function rgbToHsl(c) {
        const r = ((c >> 16) & 255) / 255.0
        const g = ((c >> 8) & 255) / 255.0
        const b = (c & 255) / 255.0
        const mx = Math.max(r, g, b), mn = Math.min(r, g, b)
        let h = 0, s = 0
        const l = (mx + mn) / 2.0
        const d = mx - mn
        if (d !== 0) {
            s = d / (1.0 - Math.abs(2.0 * l - 1.0))
            if (mx === r) h = ((g - b) / d) % 6.0
            else if (mx === g) h = (b - r) / d + 2.0
            else h = (r - g) / d + 4.0
            h *= 60.0
            if (h < 0) h += 360.0
        }
        return [h, s, l]
    }

    function hslToColor(h, s, l) {
        const cs = Math.max(0, Math.min(1, s))
        const cl = Math.max(0, Math.min(1, l))
        const c = (1 - Math.abs(2 * cl - 1)) * cs
        const hh = ((h % 360) + 360) % 360 / 60.0
        const x = c * (1 - Math.abs(hh % 2 - 1))
        let r1 = 0, g1 = 0, b1 = 0
        if (hh < 1) { r1 = c; g1 = x }
        else if (hh < 2) { r1 = x; g1 = c }
        else if (hh < 3) { g1 = c; b1 = x }
        else if (hh < 4) { g1 = x; b1 = c }
        else if (hh < 5) { r1 = x; b1 = c }
        else { r1 = c; b1 = x }
        const m = cl - c / 2.0
        return Qt.rgba(Math.max(0, Math.min(1, r1 + m)), Math.max(0, Math.min(1, g1 + m)), Math.max(0, Math.min(1, b1 + m)), 1.0)
    }

    function intToColor(c) {
        return Qt.rgba(((c >> 16) & 255) / 255.0, ((c >> 8) & 255) / 255.0, (c & 255) / 255.0, 1.0)
    }

    function mixInt(a, b, t) {
        const r = Math.round(((a >> 16) & 255) * (1 - t) + ((b >> 16) & 255) * t)
        const g = Math.round(((a >> 8) & 255) * (1 - t) + ((b >> 8) & 255) * t)
        const bl = Math.round((a & 255) * (1 - t) + (b & 255) * t)
        return (r << 16) | (g << 8) | bl
    }

    readonly property var themeColors: {
        if (!rawProfile)
            return null
        if (rawProfile.theme_colors && rawProfile.theme_colors.length >= 2)
            return [rawProfile.theme_colors[0], rawProfile.theme_colors[1]]
        if (rawProfile.theme_colors && rawProfile.theme_colors.length === 1)
            return [rawProfile.theme_colors[0], mixInt(rawProfile.theme_colors[0], 0xFFFFFF, 0.12)]
        if (rawProfile.accent_color !== null && rawProfile.accent_color !== undefined)
            return [rawProfile.accent_color, mixInt(rawProfile.accent_color, 0xFFFFFF, 0.12)]
        return null
    }

    readonly property var resolvedTheme: {
        if (themeColors && themeColors.length >= 2) {
            const p = themeColors[0]
            const a = themeColors[1]
            const hsl1 = rgbToHsl(p)
            const hsl2 = rgbToHsl(a)
            const lum1 = luminance(p)
            const lum2 = luminance(a)
            const isLight = lum1 > 0.7 && lum2 > 0.75
            const bg1 = lum1 > 0.4
                ? hslToColor(hsl1[0], hsl1[1] * 0.8, hsl1[2] * 0.88)
                : hslToColor(hsl1[0], hsl1[1], hsl1[2] * 0.65)
            const bg2 = lum2 > 0.4
                ? hslToColor(hsl2[0], hsl2[1] * 0.8, hsl2[2] * 1.05)
                : hslToColor(hsl2[0], hsl2[1], hsl2[2] * 0.65)
            const b1 = isLight
                ? hslToColor(hsl1[0], hsl1[1] * 1.15, hsl1[2] * 0.8)
                : hslToColor(hsl1[0], hsl1[1] * 1.15, Math.max(0.12, hsl1[2] * 1.3))
            const b2 = isLight
                ? hslToColor(hsl2[0], hsl2[1] * 1.15, hsl2[2] * 0.8)
                : hslToColor(hsl2[0], hsl2[1] * 1.15, Math.max(0.12, hsl2[2] * 1.3))
            const btnHue = hsl2[1] > 0.08 ? hsl2[0] : hsl1[0]
            const btnSat = hsl2[1] > 0.08 ? Math.min(0.45, Math.max(0.22, hsl2[1] * 0.65)) : Math.min(0.35, hsl1[1] * 0.6)
            const btnColor = isLight ? hslToColor(btnHue, btnSat, 0.35) : hslToColor(btnHue, btnSat, 0.82)
            const btnText = isLight ? "#FFFFFF" : "#1C1B1F"
            return {
                isCustom: true,
                bg1: bg1,
                bg2: bg2,
                b1: b1,
                b2: b2,
                bannerColor: intToColor(p),
                cutout: bg1,
                text: isLight ? "#111111" : "#FFFFFF",
                subtext: isLight ? Qt.rgba(0, 0, 0, 0.72) : Qt.rgba(1, 1, 1, 0.78),
                button: btnColor,
                buttonText: btnText
            }
        }
        return {
            isCustom: false,
            bg1: Theme.surfaceContainer,
            bg2: Theme.surface,
            b1: Theme.alpha(Theme.surfaceText, 0.2),
            b2: Theme.alpha(Theme.surfaceText, 0.1),
            bannerColor: Theme.primaryContainer,
            cutout: Theme.surfaceContainer,
            text: Theme.surfaceText,
            subtext: Theme.surfaceVariantText,
            button: Theme.primary,
            buttonText: Theme.primaryText
        }
    }

    readonly property string displayName: rawProfile
        ? (rawProfile.nickname || rawProfile.global_name || rawProfile.username || fallbackName)
        : fallbackName
    readonly property string username: rawProfile && rawProfile.username ? rawProfile.username : ""
    readonly property string pronouns: rawProfile && rawProfile.pronouns ? rawProfile.pronouns : ""
    readonly property string resolvedStatus: (rawProfile && rawProfile.status) ? rawProfile.status : fallbackStatus
    readonly property string resolvedActivity: (rawProfile && rawProfile.activity) ? rawProfile.activity : fallbackActivity
    readonly property string avatarUrl: {
        if (rawProfile && rawProfile.avatar) {
            if (rawProfile.avatar.startsWith("http"))
                return rawProfile.avatar
            const ext = rawProfile.avatar.startsWith("a_") ? "gif" : "png"
            return "https://cdn.discordapp.com/avatars/" + targetUserId + "/" + rawProfile.avatar + "." + ext + "?size=160"
        }
        return fallbackAvatar
    }
    readonly property string decorationUrl: rawProfile && rawProfile.decoration
        ? "https://cdn.discordapp.com/avatar-decoration-presets/" + rawProfile.decoration + ".png?size=160&passthrough=true"
        : fallbackDecoration
    readonly property string bannerUrl: {
        if (rawProfile && rawProfile.banner_url)
            return rawProfile.banner_url
        if (Theme.prefs.userbgEnabled && targetUserId !== "")
            return "https://usrbg.is-hardly.online/users/" + targetUserId
        return ""
    }
    readonly property bool isMe: targetUserId !== "" && targetUserId === backend.currentUserId

    function positionPopup() {
        if (!parent)
            return
        const h = Math.max(260, contentItem.implicitHeight + 8)
        if (desiredX >= 0 && desiredY >= 0) {
            openRight = desiredX < (parent.width / 2)
            let px = openRight ? (desiredX + 52) : (desiredX - width - 16)
            px = Math.max(12, Math.min(px, parent.width - width - 12))
            let py = Math.max(12, Math.min(desiredY - 64, parent.height - h - 12))
            targetX = Math.round(px)
            x = targetX
            y = Math.round(py)
        } else {
            openRight = true
            targetX = Math.round((parent.width - width) / 2)
            x = targetX
            y = Math.round((parent.height - h) / 2)
        }
    }

    function openForUser(userId, name, avatar, decoration, status, activity, ax, ay) {
        if (!userId || userId === "")
            return
        targetUserId = String(userId)
        fallbackName = name || ""
        fallbackAvatar = avatar || ""
        fallbackDecoration = decoration || ""
        fallbackStatus = status || (targetUserId === backend.currentUserId ? backend.currentUserStatus : "offline")
        fallbackActivity = activity || ""
        desiredX = (ax !== undefined && ax !== null) ? ax : -1
        desiredY = (ay !== undefined && ay !== null) ? ay : -1
        backend.fetchProfile(targetUserId)
        positionPopup()
        open()
    }

    onHeightChanged: if (opened) positionPopup()

    enter: Transition {
        ParallelAnimation {
            NumberAnimation {
                property: "x"
                from: root.desiredX >= 0 ? (root.targetX + (root.openRight ? -root.width / 4 : root.width / 4)) : root.targetX
                to: root.targetX
                duration: 220
                easing.type: Easing.OutCubic
            }
            NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 160; easing.type: Easing.OutCubic }
            NumberAnimation { property: "scale"; from: 0.96; to: 1; duration: 220; easing.type: Easing.OutCubic }
        }
    }
    exit: Transition {
        ParallelAnimation {
            NumberAnimation {
                property: "x"
                to: root.desiredX >= 0 ? (root.targetX + (root.openRight ? -root.width / 5 : root.width / 5)) : root.targetX
                duration: 160
                easing.type: Easing.OutCubic
            }
            NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 120 }
        }
    }

    background: Item {
        RectangularShadow {
            anchors.fill: outerBorder
            radius: outerBorder.radius
            blur: 24
            offset.y: 6
            color: Qt.rgba(0, 0, 0, 0.48)
        }
        Rectangle {
            id: outerBorder
            anchors.fill: parent
            radius: 16
            gradient: Gradient {
                GradientStop { position: 0.0; color: root.resolvedTheme.b1 }
                GradientStop { position: 1.0; color: root.resolvedTheme.b2 }
            }
        }
        Rectangle {
            id: innerCard
            anchors.fill: parent
            anchors.margins: 4
            radius: 12
            clip: true
            gradient: Gradient {
                GradientStop { position: 0.0; color: root.resolvedTheme.bg1 }
                GradientStop { position: 0.35; color: root.resolvedTheme.bg1 }
                GradientStop { position: 1.0; color: root.resolvedTheme.bg2 }
            }
        }
    }

    contentItem: Column {
        width: root.width - 8
        spacing: 0

        ProfileHeader {
            width: parent.width
            targetUserId: root.targetUserId
            displayName: root.displayName
            username: root.username
            avatarUrl: root.avatarUrl
            decorationUrl: root.decorationUrl
            bannerUrl: root.bannerUrl
            status: root.resolvedStatus
            activity: root.resolvedActivity
            isMe: root.isMe
            resolvedTheme: root.resolvedTheme
        }

        Column {
            width: parent.width - 28
            x: 14
            topPadding: 4
            bottomPadding: 16
            spacing: 10

            Column {
                width: parent.width
                spacing: 2

                Text {
                    width: parent.width
                    text: root.displayName || "User"
                    color: root.resolvedTheme.text
                    font.family: Theme.font
                    font.pixelSize: 20
                    font.weight: Font.Bold
                    elide: Text.ElideRight
                }

                Text {
                    width: parent.width
                    visible: root.username !== "" || root.pronouns !== ""
                    text: root.username + (root.username !== "" && root.pronouns !== "" ? "  •  " : "") + root.pronouns
                    color: root.resolvedTheme.subtext
                    font.family: Theme.font
                    font.pixelSize: 13
                    elide: Text.ElideRight
                }
            }

            Flow {
                width: parent.width
                spacing: 6
                visible: root.rawProfile && root.rawProfile.badges && root.rawProfile.badges.length > 0

                Repeater {
                    model: root.rawProfile && root.rawProfile.badges ? root.rawProfile.badges : []

                    delegate: Item {
                        required property var modelData
                        width: 22
                        height: 22

                        Image {
                            anchors.centerIn: parent
                            width: 20
                            height: 20
                            source: modelData.iconUrl || ("https://cdn.discordapp.com/badge-icons/" + modelData.icon + ".png?size=64")
                            fillMode: Image.PreserveAspectFit
                            asynchronous: true
                            smooth: true
                        }
                        RichTooltip {
                            anchor: parent
                            placement: "above"
                            visible: badgeHover.hovered && (modelData.description || "") !== ""
                            text: modelData.description || ""
                        }
                        HoverHandler { id: badgeHover }
                    }
                }
            }

            Rectangle {
                width: parent.width
                height: 38
                radius: 19
                color: root.resolvedTheme.button
                opacity: ctaHover.hovered ? 0.9 : 1.0
                Behavior on opacity { NumberAnimation { duration: Theme.quick } }

                Row {
                    anchors.centerIn: parent
                    spacing: 8

                    Icon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: root.isMe ? "Edit" : "ChatBubble"
                        size: 17
                        color: root.resolvedTheme.buttonText
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.isMe ? "Edit Profile" : "Message"
                        color: root.resolvedTheme.buttonText
                        font.family: Theme.font
                        font.pixelSize: 14
                        font.weight: Font.DemiBold
                    }
                }

                HoverHandler { id: ctaHover; cursorShape: Qt.PointingHandCursor }
                TapHandler {
                    onTapped: {
                        if (root.isMe) {
                            root.close()
                            if (window.openSettings)
                                window.openSettings()
                        } else {
                            backend.openDm(root.targetUserId)
                            root.close()
                        }
                    }
                }
            }

            Column {
                width: parent.width
                spacing: 4
                visible: root.rawProfile && root.rawProfile.bio && root.rawProfile.bio !== "" ? true : false

                Text {
                    text: "About Me"
                    color: root.resolvedTheme.text
                    font.family: Theme.font
                    font.pixelSize: 12
                    font.weight: Font.Bold
                }
                Text {
                    width: parent.width
                    textFormat: Text.RichText
                    text: root.rawProfile && root.rawProfile.bio ? backend.renderMarkdown(
                        root.rawProfile.bio,
                        String(Theme.surfaceContainerHighest),
                        String(Theme.primaryContainer),
                        String(Theme.primaryContainerText),
                        String(root.resolvedTheme.subtext),
                        Theme.monoFont
                    ) : ""
                    color: root.resolvedTheme.subtext
                    linkColor: root.resolvedTheme.button
                    font.family: Theme.font
                    font.pixelSize: 13
                    wrapMode: Text.Wrap
                    onLinkActivated: link => Qt.openUrlExternally(link)
                }
            }
        }
    }
}
