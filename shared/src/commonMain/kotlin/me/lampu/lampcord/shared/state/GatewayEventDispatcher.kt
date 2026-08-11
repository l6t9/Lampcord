package me.lampu.lampcord.shared.state

import kotlinx.serialization.json.JsonElement
import me.lampu.lampcord.shared.model.GatewayPayload

interface GatewayEventHandler {
    val supportedEvents: Set<String>
    fun handleEvent(type: String, data: JsonElement?)
}

class GatewayEventDispatcher(
    private val handlers: List<GatewayEventHandler>
) {
    private val handlerMap = mutableMapOf<String, MutableList<GatewayEventHandler>>()

    init {
        handlers.forEach { handler ->
            handler.supportedEvents.forEach { event ->
                handlerMap.getOrPut(event) { mutableListOf() }.add(handler)
            }
        }
    }

    fun dispatch(payload: GatewayPayload) {
        handlerMap[payload.t]?.forEach { handler ->
            handler.handleEvent(payload.t!!, payload.d)
        }
    }
}
