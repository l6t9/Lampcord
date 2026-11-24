package me.lampu.lampcord.shared.state

object RustChannelVisibility {
    private var hidden: Set<String> = emptySet()
    private var readOnly: Set<String> = emptySet()

    fun replaceHidden(channelIds: List<String>) {
        hidden = channelIds.toSet()
    }

    fun replaceReadOnly(channelIds: List<String>) {
        readOnly = channelIds.toSet()
    }

    fun canView(channelId: String): Boolean = channelId !in hidden

    fun canSend(channelId: String): Boolean = channelId !in readOnly
}
