package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateListOf
import me.lampu.lampcord.shared.model.Application
import me.lampu.lampcord.shared.model.ApplicationCommand

class CommandStore {
    val availableCommands = mutableStateListOf<ApplicationCommand>()
    val availableApplications = mutableStateListOf<Application>()

    fun setCommands(commands: List<ApplicationCommand>, applications: List<Application>) {
        availableCommands.clear()
        availableCommands.addAll(commands)
        availableApplications.clear()
        availableApplications.addAll(applications)
    }

    fun clear() {
        availableCommands.clear()
        availableApplications.clear()
    }
}
