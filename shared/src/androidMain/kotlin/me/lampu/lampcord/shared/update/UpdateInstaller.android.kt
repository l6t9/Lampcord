package me.lampu.lampcord.shared.update

import android.content.Intent
import androidx.core.content.FileProvider
import me.lampu.lampcord.shared.utils.AndroidContextProvider
import java.io.File

actual fun installUpdate(
    target: UpdateTarget,
    downloadedFile: String,
    version: String,
): InstallResult {
    if (target != UpdateTarget.ANDROID) return InstallResult.Unsupported("Not an Android build")

    val context = AndroidContextProvider.applicationContext
    val apk = File(downloadedFile)
    if (!apk.isFile) return InstallResult.Failed("The downloaded build is missing")

    val uri = runCatching {
        FileProvider.getUriForFile(context, "${context.packageName}.fileprovider", apk)
    }.getOrElse { return InstallResult.Failed("The build could not be shared with the installer") }

    val intent = Intent(Intent.ACTION_VIEW).apply {
        setDataAndType(uri, "application/vnd.android.package-archive")
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    }

    return runCatching {
        context.startActivity(intent)
        InstallResult.Started
    }.getOrElse { InstallResult.Failed(it.message ?: "The installer could not be started") }
}
