package me.lampu.lampcord.shared.utils

actual fun platformLog(level: String, tag: String, message: String, throwable: Throwable?) {
    println("$level/$tag: $message")
    throwable?.let { println(it.stackTraceToString()) }
}
