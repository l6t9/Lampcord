package me.lampu.lampcord.shared.ui.navigation

class BackInterceptor {
    private var handler: (() -> Boolean)? = null

    fun register(handler: () -> Boolean): () -> Unit {
        this.handler = handler
        return { if (this.handler === handler) this.handler = null }
    }

    fun clear() {
        handler = null
    }

    fun handleBack(): Boolean = handler?.invoke() ?: false
}
