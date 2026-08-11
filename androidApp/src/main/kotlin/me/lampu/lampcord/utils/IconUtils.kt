package me.lampu.lampcord.utils

import android.content.ComponentName
import android.content.Context
import android.content.pm.PackageManager

private const val MANIFEST_PACKAGE = "me.lampu.lampcord"

enum class AppIcon(
    val aliasName: String,
    val enabledByDefault: Boolean = false,
) {
    LAMPCORD(
        aliasName = "MainActivity",
        enabledByDefault = true,
    ),
    ;

    companion object {
        fun fromPreference(value: String): AppIcon = LAMPCORD
    }
}

object IconUtils {
    fun setIcon(
        context: Context,
        icon: AppIcon,
    ) {
        val pm = context.packageManager
        val selectedComponent = icon.componentName(context)

        if (!pm.isEnabled(selectedComponent, icon.enabledByDefault)) {
            pm.setComponentEnabledSetting(
                selectedComponent,
                PackageManager.COMPONENT_ENABLED_STATE_ENABLED,
                PackageManager.DONT_KILL_APP,
            )
        }
    }

    private fun AppIcon.componentName(context: Context) = ComponentName(context, "$MANIFEST_PACKAGE.$aliasName")

    private fun PackageManager.isEnabled(
        component: ComponentName,
        enabledByDefault: Boolean,
    ): Boolean =
        when (getComponentEnabledSetting(component)) {
            PackageManager.COMPONENT_ENABLED_STATE_ENABLED -> true
            PackageManager.COMPONENT_ENABLED_STATE_DEFAULT -> enabledByDefault
            else -> false
        }
}
