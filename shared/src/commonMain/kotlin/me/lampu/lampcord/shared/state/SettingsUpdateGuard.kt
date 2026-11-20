package me.lampu.lampcord.shared.state

/**
 * Tracks the newest optimistic settings update so a slow failure from an older
 * request cannot roll back a newer (possibly successful) update.
 */
internal class SettingsUpdateGuard {
    private var latestRevision = 0

    fun begin(): Int = ++latestRevision

    fun isLatest(revision: Int): Boolean = revision == latestRevision
}
