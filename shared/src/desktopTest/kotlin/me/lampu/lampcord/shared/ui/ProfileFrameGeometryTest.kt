package me.lampu.lampcord.shared.ui

import me.lampu.lampcord.shared.model.frameLayerAspectRatio
import me.lampu.lampcord.shared.model.frameLayerHeight
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class ProfileFrameGeometryTest {

    @Test
    fun aspectRatioComesFromMeasuredImageRatherThanAGuess() {
        assertEquals(0.5f, frameLayerAspectRatio(1200, 600))
        assertEquals(1f, frameLayerAspectRatio(600, 600))
        assertNull(frameLayerAspectRatio(0, 600))
        assertNull(frameLayerAspectRatio(1200, 0))
        assertNull(frameLayerAspectRatio(1200, -1))
    }

    @Test
    fun layerStaysHiddenUntilAValidAspectRatioIsKnown() {
        assertEquals(0f, frameLayerHeight(400f, null))
        assertEquals(0f, frameLayerHeight(400f, frameLayerAspectRatio(0, 600)))
        assertEquals(200f, frameLayerHeight(400f, frameLayerAspectRatio(1200, 600)))
        assertEquals(0f, frameLayerHeight(0f, 0.5f))
    }
}
