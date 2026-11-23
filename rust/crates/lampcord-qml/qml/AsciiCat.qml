import QtQuick

// The login-screen cats. The art mixes six scripts, and Qt's own fallback
// picks fonts lazily (differently from launch to launch), so every run of
// characters names the fonts fontconfig gives Compose for that script.
Text {
    id: root

    readonly property string art:
        "          へ   ♡     ╱|、\n" +
        "     ૮  -   ՛ )      (`   -  7\n" +
        "       /   ⁻  ៸|       |、⁻〵\n" +
        " 乀 (ˍ, ل ل        じしˍ,)ノ"

    readonly property string base: "'Noto Sans', sans-serif"
    readonly property string cjk: "'Noto Sans CJK KR', 'Noto Sans CJK JP', 'Noto Sans CJK SC', sans-serif"

    function fontFor(code) {
        if (code >= 0x0A80 && code <= 0x0AFF)
            return "'Noto Sans Gujarati', sans-serif"
        if (code >= 0x1780 && code <= 0x17FF)
            return "'Noto Sans Khmer', sans-serif"
        if ((code >= 0x0530 && code <= 0x058F) || (code >= 0x0600 && code <= 0x06FF))
            return "'Droid Sans', 'Noto Sans Armenian', 'Noto Sans Arabic', sans-serif"
        if (code === 0x2661 || (code >= 0x2500 && code <= 0x257F) || code >= 0x3000)
            return cjk
        return base
    }

    function markup(text) {
        let html = ""
        let run = ""
        let runFont = ""
        const flush = () => {
            if (run !== "")
                html += "<span style=\"font-family: " + runFont + "\">" + run + "</span>"
            run = ""
        }
        for (const ch of text) {
            if (ch === "\n") {
                flush()
                html += "<br>"
                continue
            }
            const font = fontFor(ch.codePointAt(0))
            if (font !== runFont) {
                flush()
                runFont = font
            }
            run += ch === "&" ? "&amp;" : ch === "<" ? "&lt;" : ch
        }
        flush()
        return "<div style=\"white-space: pre\">" + html + "</div>"
    }

    text: markup(art)
    textFormat: Text.RichText
    color: Theme.primary
    font.pixelSize: 16
    lineHeight: 22
    lineHeightMode: Text.FixedHeight
}
