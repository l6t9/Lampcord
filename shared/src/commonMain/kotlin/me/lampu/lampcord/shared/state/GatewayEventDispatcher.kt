package me.lampu.lampcord.shared.state

import kotlinx.serialization.json.JsonElement
import me.lampu.lampcord.shared.model.GatewayPayload
import me.lampu.lampcord.shared.utils.Logging

interface GatewayEventHandler {
    val supportedEvents: Set<String>
    fun handleEvent(type: String, data: JsonElement?)
    fun handlePayload(payload: GatewayPayload) {
        handleEvent(payload.t ?: return, payload.d)
    }
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
        val type = payload.t ?: return
        handlerMap[type]?.forEach { handler ->
            // A single handler must never be able to crash the whole dispatch
            // pipeline. Besides guarding the rest of the handlers for this
            // event, an uncaught throw here would bubble all the way up to the
            // SessionManager collector and permanently kill the gateway event
            // stream (a replayless SharedFlow silently drops everything after
            // the collector dies), which is exactly the "messages stop syncing
            // until I restart" symptom.
            try {
                handler.handlePayload(payload)
            } catch (e: kotlin.coroutines.cancellation.CancellationException) {
                throw e
            } catch (e: Exception) {
                Logging.e("GatewayDispatcher", "Error handling $type in ${handler::class.simpleName}", e)
            }
        }
    }
}
