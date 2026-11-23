package me.lampu.lampcord.shared.ui.components.sheet

import android.content.Context
import android.view.accessibility.AccessibilityManager
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext

@Composable
actual fun screenReaderActive(): Boolean {
    val manager = LocalContext.current
        .getSystemService(Context.ACCESSIBILITY_SERVICE) as? AccessibilityManager
        ?: return false
    return manager.isEnabled && manager.isTouchExplorationEnabled
}
