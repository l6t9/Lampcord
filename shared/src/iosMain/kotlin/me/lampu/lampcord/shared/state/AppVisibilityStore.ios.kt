package me.lampu.lampcord.shared.state

import platform.Foundation.NSNotificationCenter
import platform.Foundation.NSObjectProtocol
import platform.UIKit.UIApplicationDidBecomeActiveNotification
import platform.UIKit.UIApplicationWillResignActiveNotification

private var registered = false

// Kotlin/Native observers are released as soon as the returned protocol object becomes
// unreachable, so they must be retained for the process lifetime.
private var observers: List<NSObjectProtocol> = emptyList()

internal actual fun registerVisibilityListener(onVisible: () -> Unit, onHidden: () -> Unit) {
    if (registered) return
    registered = true

    val center = NSNotificationCenter.defaultCenter
    observers = listOf(
        center.addObserverForName(UIApplicationDidBecomeActiveNotification, null, null) {
            onVisible()
        },
        center.addObserverForName(UIApplicationWillResignActiveNotification, null, null) {
            onHidden()
        }
    )
}