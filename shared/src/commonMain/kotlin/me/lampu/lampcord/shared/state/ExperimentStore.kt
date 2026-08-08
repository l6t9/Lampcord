package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import kotlinx.serialization.json.*

class ExperimentStore {
    var userExperiments = mutableStateMapOf<String, Int>()
    var guildExperiments = mutableStateMapOf<String, Map<String, Int>>()

    fun handleReady(experiments: List<JsonElement>?) {
        experiments?.forEach { experiment ->
            try {
                val arr = experiment.jsonArray
                val name = arr[0].jsonPrimitive.content
                val bucket = arr[1].jsonPrimitive.int
                userExperiments[name] = bucket
            } catch (e: Exception) { }
        }
    }

    fun getBucket(name: String): Int = userExperiments[name] ?: -1
}
