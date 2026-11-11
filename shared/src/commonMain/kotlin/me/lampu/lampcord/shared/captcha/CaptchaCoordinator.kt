package me.lampu.lampcord.shared.captcha

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import me.lampu.lampcord.shared.api.CaptchaChallenge

class CaptchaRequest(
    val challenge: CaptchaChallenge,
    val result: CompletableDeferred<String?>,
)

class CaptchaCoordinator {
    private val mutex = Mutex()
    private val _pending = MutableStateFlow<CaptchaRequest?>(null)
    val pending: StateFlow<CaptchaRequest?> = _pending.asStateFlow()

    suspend fun solve(challenge: CaptchaChallenge): String? = mutex.withLock {
        val request = CaptchaRequest(challenge, CompletableDeferred())
        _pending.value = request
        try {
            request.result.await()
        } finally {
            _pending.value = null
        }
    }

    fun complete(token: String) {
        _pending.value?.result?.complete(token)
    }

    fun cancel() {
        _pending.value?.result?.complete(null)
    }
}
