package me.lampu.lampcord.shared.state

import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

@OptIn(ExperimentalEncodingApi::class)
class UserSettingsProtoEditorTest {

    @Test
    fun changesOnlyUiDensityAndKeepsEveryOtherField() {
        val appearance = byteArrayOf(0x10, 0x01, 0x60, 0x01, 0x1a, 0x02, 0x7f, 0x0f)
        val root = byteArrayOf(0x18, 0x2a) + bytesField(13, appearance) + varintField(22, 7)
        val updatedAppearance = byteArrayOf(0x10, 0x01, 0x60, 0x04, 0x1a, 0x02, 0x7f, 0x0f)
        val expected = byteArrayOf(0x18, 0x2a) + bytesField(13, updatedAppearance) + varintField(22, 7)

        assertEquals(
            Base64.Default.encode(expected),
            UserSettingsProtoEditor.withUiDensity(Base64.Default.encode(root), density = 4)
        )
    }

    @Test
    fun insertsUiDensityWhenAppearanceDoesNotHaveIt() {
        val appearance = byteArrayOf(0x10, 0x01)
        val root = bytesField(13, appearance)
        val expected = bytesField(13, appearance + byteArrayOf(0x60, 0x02))

        assertEquals(
            Base64.Default.encode(expected),
            UserSettingsProtoEditor.withUiDensity(Base64.Default.encode(root), density = 2)
        )
    }

    @Test
    fun addsAppearanceFieldWhenMissingWithoutDroppingOtherFields() {
        val root = byteArrayOf(0x18, 0x2a)
        val expected = root + bytesField(13, byteArrayOf(0x60, 0x01))

        assertEquals(
            Base64.Default.encode(expected),
            UserSettingsProtoEditor.withUiDensity(Base64.Default.encode(root), density = 1)
        )
    }

    @Test
    fun invalidPayloadDoesNotProduceAPatch() {
        assertNull(UserSettingsProtoEditor.withUiDensity("not base64!", density = 4))
    }

    private fun bytesField(field: Int, payload: ByteArray): ByteArray =
        varint((field shl 3) or 2) + varint(payload.size) + payload

    private fun varintField(field: Int, value: Int): ByteArray =
        varint((field shl 3) or 0) + varint(value)

    private fun varint(value: Int): ByteArray {
        var remaining = value
        val out = mutableListOf<Byte>()
        while (remaining and 0x7f.inv() != 0) {
            out += ((remaining and 0x7f) or 0x80).toByte()
            remaining = remaining ushr 7
        }
        out += remaining.toByte()
        return out.toByteArray()
    }
}
