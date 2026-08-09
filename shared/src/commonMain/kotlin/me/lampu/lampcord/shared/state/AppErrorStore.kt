package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.snapshots.SnapshotStateList

sealed class AppError(val message: String, val severity: ErrorSeverity = Severity.ERROR) {
    object NetworkError : AppError("Network connection issue. Please check your internet.")
    class ApiError(message: String) : AppError(message)
    class GatewayError(message: String) : AppError(message)
    class GenericError(message: String) : AppError(message)

    enum class Severity {
        INFO, WARNING, ERROR
    }
}

typealias ErrorSeverity = AppError.Severity

class AppErrorStore {
    val errors = mutableStateListOf<AppError>()

    fun pushError(error: AppError) {
        errors.add(error)
    }

    fun pushError(message: String, severity: ErrorSeverity = ErrorSeverity.ERROR) {
        errors.add(AppError.GenericError(message))
    }

    fun consumeError(error: AppError) {
        errors.remove(error)
    }

    fun clear() {
        errors.clear()
    }
}
