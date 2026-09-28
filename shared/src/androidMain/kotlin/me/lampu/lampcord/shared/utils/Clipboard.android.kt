package me.lampu.lampcord.shared.utils

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns

actual fun getClipboardFiles(): List<Pair<String, ByteArray>> {
    val context = AndroidContextProvider.applicationContext
    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    val clip = clipboard.primaryClip ?: return emptyList()
    
    val files = mutableListOf<Pair<String, ByteArray>>()
    
    for (i in 0 until clip.itemCount) {
        val item = clip.getItemAt(i)
        val uri = item.uri ?: continue
        
        try {
            val fileName = getFileName(context, uri) ?: "pasted_file_$i"
            context.contentResolver.openInputStream(uri)?.use { inputStream ->
                files.add(fileName to inputStream.readBytes())
            }
        } catch (e: Exception) {
        }
    }
    
    return files
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
