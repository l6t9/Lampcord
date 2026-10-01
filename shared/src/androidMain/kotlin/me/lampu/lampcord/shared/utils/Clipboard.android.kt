package me.lampu.lampcord.shared.utils

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.OpenableColumns
import androidx.compose.runtime.Composable

actual fun getClipboardFiles(): List<Pair<String, ByteArray>> {
    val context = AndroidContextProvider.applicationContext
    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    val clip = clipboard.primaryClip ?: return emptyList()

    val files = mutableListOf<Pair<String, ByteArray>>()

    for (i in 0 until clip.itemCount) {
        val item = clip.getItemAt(i)

        // Copying an image out of a gallery or browser puts it in the clipboard as an Intent
        // carrying EXTRA_STREAM rather than as a plain uri, so the uri alone is not enough.
        val intentUri = item.intent?.let { intent ->
            @Suppress("DEPRECATION")
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                intent.getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java)
            } else {
                intent.getParcelableExtra(Intent.EXTRA_STREAM) as? Uri
            }
        }
        val uri = item.uri ?: intentUri ?: continue

        try {
            val fileName = getFileName(context, uri) ?: inferFileName(context, uri, i)
            context.contentResolver.openInputStream(uri)?.use { inputStream ->
                val bytes = inputStream.readBytes()
                if (bytes.isNotEmpty()) files.add(fileName to bytes)
            }
        } catch (e: Exception) {
        }
    }

    return files
}

private fun inferFileName(context: Context, uri: Uri, index: Int): String {
    val mime = context.contentResolver.getType(uri)
    val extension = when {
        mime == null -> null
        mime.contains("png") -> "png"
        mime.contains("webp") -> "webp"
        mime.contains("gif") -> "gif"
        mime.contains("heic") -> "heic"
        else -> "jpg"
    }
    return if (extension != null) "pasted_image_$index.$extension" else "pasted_image_$index.jpg"
}

private fun getFileName(context: Context, uri: Uri): String? {
    if (uri.scheme == "content") {
        context.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) {
                val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                if (index != -1) {
                    return cursor.getString(index)
                }
            }
        }
    }
    return uri.path?.let { path ->
        val cut = path.lastIndexOf('/')
        if (cut != -1) path.substring(cut + 1) else path
    }
}

actual fun setClipboardText(text: String) {
    val context = AndroidContextProvider.applicationContext
    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    val clip = ClipData.newPlainText("lampcord", text)
    clipboard.setPrimaryClip(clip)
}

@Composable
actual fun ProvideClipboard() = Unit
