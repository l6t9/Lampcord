package me.lampu.lampcord.shared.captcha

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withTimeoutOrNull
import me.lampu.lampcord.shared.api.CaptchaChallenge

const val CAPTCHA_TIMEOUT_MILLIS = 5L * 60L * 1000L

sealed interface CaptchaOutcome {
    data class Solved(val token: String) : CaptchaOutcome
    data object Cancelled : CaptchaOutcome
    data object TimedOut : CaptchaOutcome
}

class CaptchaRequest(
    val challenge: CaptchaChallenge,
    val result: CompletableDeferred<CaptchaOutcome>,
)

class CaptchaCoordinator(private val timeoutMillis: Long = CAPTCHA_TIMEOUT_MILLIS) {
    private val mutex = Mutex()
    private val _pending = MutableStateFlow<CaptchaRequest?>(null)
    val pending: StateFlow<CaptchaRequest?> = _pending.asStateFlow()

    suspend fun solve(challenge: CaptchaChallenge): CaptchaOutcome = mutex.withLock {
        val request = CaptchaRequest(challenge, CompletableDeferred())
        _pending.value = request
        try {
            withTimeoutOrNull(timeoutMillis) { request.result.await() } ?: CaptchaOutcome.TimedOut
        } finally {
            _pending.value = null
        }
    }

    fun complete(token: String) {
        _pending.value?.result?.complete(CaptchaOutcome.Solved(token))
    }

    fun cancel() {
        _pending.value?.result?.complete(CaptchaOutcome.Cancelled)
    }
}
