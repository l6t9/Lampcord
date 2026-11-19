package me.lampu.lampcord

import android.app.Activity
import android.app.Application
import android.os.Bundle
import me.lampu.lampcord.shared.state.AppVisibilityStore
import kotlin.jvm.Volatile

object AppLifecycleTracker : Application.ActivityLifecycleCallbacks {
    private var startedActivities = 0

    @Volatile
    var isInForeground: Boolean = false
        private set

    fun register(application: Application) {
        // The process can start from a push or service with no activity shown.
        AppVisibilityStore.setVisible(isInForeground)
        application.registerActivityLifecycleCallbacks(this)
    }

    override fun onActivityStarted(activity: Activity) {
        startedActivities++
        if (startedActivities == 1 && !isInForeground) {
            isInForeground = true
            AppVisibilityStore.setVisible(true)
        }
    }

    override fun onActivityStopped(activity: Activity) {
        startedActivities--
        if (startedActivities <= 0 && isInForeground) {
            startedActivities = 0
            isInForeground = false
            AppVisibilityStore.setVisible(false)
        }
    }

    override fun onActivityCreated(activity: Activity, savedInstanceState: Bundle?) {}
    override fun onActivityResumed(activity: Activity) {}
    override fun onActivityPaused(activity: Activity) {}
    override fun onActivitySaveInstanceState(activity: Activity, outState: Bundle) {}
    override fun onActivityDestroyed(activity: Activity) {}
}
