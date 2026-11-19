package me.lampu.lampcord.shared.ui.components

import android.os.SystemClock
import android.widget.Toast
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.navigationevent.NavigationEventInfo
import androidx.navigationevent.compose.NavigationBackHandler as NavigationEventBackHandler
import androidx.navigationevent.compose.rememberNavigationEventState
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.back_again_to_exit
import org.jetbrains.compose.resources.stringResource

@Composable
actual fun PlatformRootBackHandler(
    enabled: Boolean,
    onBack: () -> Boolean,
    onExit: () -> Unit,
) {
    val confirmation = remember(enabled) { BackExitConfirmation() }
    val context = LocalContext.current
    val message = stringResource(Res.string.back_again_to_exit)
    val toast = remember(context, message, confirmation) {
        Toast.makeText(context, message, Toast.LENGTH_SHORT)
    }

    LifecycleResumeEffect(confirmation, toast) {
        onPauseOrDispose {
            confirmation.reset()
            toast.cancel()
        }
    }

    val state = rememberNavigationEventState(NavigationEventInfo.None)
    NavigationEventBackHandler(
        state = state,
        isBackEnabled = enabled,
    ) {
        if (onBack()) {
            confirmation.reset()
        } else if (confirmation.onBack(SystemClock.uptimeMillis())) {
            toast.cancel()
            onExit()
        } else {
            toast.show()
        }
    }
}
