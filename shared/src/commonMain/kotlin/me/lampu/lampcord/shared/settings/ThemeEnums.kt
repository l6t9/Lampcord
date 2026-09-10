package me.lampu.lampcord.shared.settings

enum class ThemeMode {
    LIGHT,
    DARK,
    AUTO,
}

enum class ThemePaletteStyle {
    TONAL_SPOT,
    VIBRANT,
    EXPRESSIVE,
    RAINBOW,
    FRUIT_SALAD,
    MONOCHROME,
    NEUTRAL,
}

enum class FontOption {
    SYSTEM,
    INTER,
    GOOGLE_SANS,
    MAPLE_MONO,
    CUSTOM,
}

enum class TapTapAction {
    REPLY_OR_EDIT,
    EMOJI_PICKER,
    DISABLED
}

enum class ChatGestures {
    SWIPE_TO_MEMBERS,
    SWIPE_TO_REPLY,
}

enum class StickerAnimation {
    ALWAYS,
    ON_HOVER,
    NEVER,
}

enum class PanelAnimation {
    MINIMAL,
    EXPRESSIVE
}

enum class TransparencyMode {
    NONE,
    CHAT,
    CHAT_SETTINGS,
    FULL
}
