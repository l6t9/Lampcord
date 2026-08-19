package me.lampu.lampcord.shared.ui.components

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarResult
import androidx.compose.material3.Snackbar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import kotlinx.coroutines.flow.collect
import me.lampu.lampcord.shared.utils.SnackbarManager

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun GlobalSnackbarHost(modifier: Modifier = Modifier) {
    val hostState = remember { SnackbarHostState() }

    // Collector: show snackbars emitted by SnackbarManager
    LaunchedEffect(Unit) {
        SnackbarManager.flow.collect { req ->
            val result = hostState.showSnackbar(req.message, req.actionLabel)
            if (result == SnackbarResult.ActionPerformed) {
                try { req.action?.invoke() } catch (_: Exception) {}
            }
        }
    }

    SnackbarHost(hostState = hostState, modifier = modifier) { data ->
        // Use the Material3 Expressive snackbar styling (opt-in)
        Snackbar(snackbarData = data)
    }
}
