package me.lampu.lampcord.shared.ui

import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import me.lampu.lampcord.shared.model.ProfileCollectibles
import me.lampu.lampcord.shared.model.ProfileEffectLayer
import me.lampu.lampcord.shared.model.effectLayerCycleMs
import me.lampu.lampcord.shared.model.isEffectLayerActive
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ProfileEffectTimelineTest {

    private fun layer(
        start: Long = 0,
        duration: Long = 0,
        loop: Boolean = false,
        loopDelay: Long = 0,
    ) = ProfileEffectLayer(
        source = "https://cdn.discordapp.com/effects/test.gif",
        start = start,
        duration = duration,
        loop = loop,
        loopDelay = loopDelay,
    )

    @Test
    fun singleUseLayerHonorsStartAndDuration() {
        val effect = layer(start = 100, duration = 200)

        assertFalse(isEffectLayerActive(effect, 99))
        assertTrue(isEffectLayerActive(effect, 100))
        assertTrue(isEffectLayerActive(effect, 299))
        assertFalse(isEffectLayerActive(effect, 300))
        assertNull(effectLayerCycleMs(effect))
    }

    @Test
    fun loopingLayerSkipsItsLoopDelayBeforeRepeating() {
        val effect = layer(start = 100, duration = 200, loop = true, loopDelay = 50)

        assertEquals(250L, effectLayerCycleMs(effect))
        assertTrue(isEffectLayerActive(effect, 100))
        assertTrue(isEffectLayerActive(effect, 299))
        assertFalse(isEffectLayerActive(effect, 300))
        assertFalse(isEffectLayerActive(effect, 349))
        assertTrue(isEffectLayerActive(effect, 350))
        assertFalse(isEffectLayerActive(effect, 550))
    }

    @Test
    fun durationlessLayersStayActiveAfterTheirStart() {
        val once = layer(start = 50)
        val looping = layer(loop = true, loopDelay = 100)

        assertFalse(isEffectLayerActive(once, 49))
        assertTrue(isEffectLayerActive(once, Long.MAX_VALUE - 1))
        assertNull(effectLayerCycleMs(looping))
        assertTrue(isEffectLayerActive(looping, 10_000))
    }

    @Test
    fun effectParserReadsLoopAndLoopDelay() {
        val payload = JsonObject(
            mapOf(
                "items" to JsonArray(
                    listOf(
                        JsonObject(
                            mapOf(
                                "type" to JsonPrimitive(1),
                                "effects" to JsonArray(
                                    listOf(
                                        JsonObject(
                                            mapOf(
                                                "src" to JsonPrimitive("https://cdn.discordapp.com/effects/test.gif"),
                                                "start" to JsonPrimitive(100),
                                                "duration" to JsonPrimitive(200),
                                                "loop" to JsonPrimitive(true),
                                                "loopDelay" to JsonPrimitive(250),
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )

        val product = assertNotNull(ProfileCollectibles.parseEffect("effect-sku", payload))
        val effect = product.layers.single()

        assertTrue(effect.loop)
        assertEquals(250L, effect.loopDelay)
        // 200 ms of animation followed by the 250 ms loop delay.
        assertEquals(450L, effectLayerCycleMs(effect))
        assertFalse(isEffectLayerActive(effect, 300))
        assertFalse(isEffectLayerActive(effect, 549))
        assertTrue(isEffectLayerActive(effect, 550))
    }

    @Test
    fun effectParserAcceptsSnakeCaseDelayAndNumericLoopFlag() {
        val payload = JsonObject(
            mapOf(
                "items" to JsonArray(
                    listOf(
                        JsonObject(
                            mapOf(
                                "type" to JsonPrimitive("1"),
                                "effects" to JsonArray(
                                    listOf(
                                        JsonObject(
                                            mapOf(
                                                "src" to JsonPrimitive("https://cdn.discordapp.com/effects/test.gif"),
                                                "duration" to JsonPrimitive(100),
                                                "loop" to JsonPrimitive(1),
                                                "loop_delay_ms" to JsonPrimitive(50),
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )

        val effect = assertNotNull(ProfileCollectibles.parseEffect("effect-sku", payload)).layers.single()

        assertTrue(effect.loop)
        assertEquals(50L, effect.loopDelay)
        assertEquals(150L, effectLayerCycleMs(effect))
        assertFalse(isEffectLayerActive(effect, 100))
        assertTrue(isEffectLayerActive(effect, 150))
    }
}
