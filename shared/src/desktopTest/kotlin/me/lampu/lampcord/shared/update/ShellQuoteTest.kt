package me.lampu.lampcord.shared.update

import kotlin.test.Test
import kotlin.test.assertEquals

class ShellQuoteTest {

    @Test
    fun `plain paths are wrapped in single quotes`() {
        assertEquals("'/home/lampu/Lampcord.AppImage'", shellQuote("/home/lampu/Lampcord.AppImage"))
    }

    @Test
    fun `spaces do not split the argument`() {
        assertEquals(
            "'/home/lampu/My Apps/Lampcord.AppImage'",
            shellQuote("/home/lampu/My Apps/Lampcord.AppImage")
        )
    }

    @Test
    fun `single quotes are escaped rather than terminating the string`() {
        assertEquals("'/home/o'\\''brien/'\\''App'", shellQuote("/home/o'brien/'App"))
    }

    @Test
    fun `command substitution and expansion stay inert`() {
        val hostile = "/tmp/\$(rm -rf ~)/" + "`whoami`" + "/\"x\""
        val quoted = shellQuote(hostile)

        assertEquals("'/tmp/\$(rm -rf ~)/`whoami`/\"x\"'", quoted)
        assertEquals(hostile, quoted.removeSurrounding("'"))
    }

    @Test
    fun `a real staged update path round trips through the shell`() {
        val staged = "/home/lampu/.local/opt/lampcord/Lampcord.AppImage.1.0.0-a10.new"
        val unquoted = shellQuote(staged).removeSurrounding("'").replace("'\\''", "'")

        assertEquals(staged, unquoted)
    }
}
