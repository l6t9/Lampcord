package me.lampu.lampcord.shared.update

actual object UpdateStorage {
    actual fun open(version: String, target: UpdateTarget): UpdateSink = DiscardingUpdateSink

    actual fun clear() = Unit
}

private object DiscardingUpdateSink : UpdateSink {
    override val path: String get() = ""
    override fun write(source: ByteArray, offset: Int, length: Int) = Unit
    override fun commit() = Unit
    override fun abort() = Unit
}