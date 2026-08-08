package me.lampu.lampcord.shared.utils

import android.content.ContentUris
import android.os.Build
import android.provider.MediaStore
import me.lampu.lampcord.shared.model.LocalMedia
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

actual fun getPlatformName(): String = "android"

actual fun getCurrentTimeMillis(): Long = System.currentTimeMillis()

actual fun randomUUID(): String = java.util.UUID.randomUUID().toString()

actual fun getOsVersion(): String = Build.VERSION.RELEASE

actual fun getOsSdkVersion(): String = Build.VERSION.SDK_INT.toString()

actual fun getOsArch(): String = Build.SUPPORTED_ABIS.firstOrNull() ?: "unknown"

actual fun getDeviceName(): String = "${Build.MODEL}, ${Build.PRODUCT}"

actual fun getCpuCoreCount(): Int = Runtime.getRuntime().availableProcessors()

actual fun getMemoryMemory(): Long = Runtime.getRuntime().totalMemory() / (1024 * 1024)

private object AndroidContext : KoinComponent {
    val context: Context by inject()
}

actual suspend fun getLocalMedia(): List<LocalMedia> = withContext(Dispatchers.IO) {
    val context = AndroidContext.context
    val mediaList = mutableListOf<LocalMedia>()

    val projection = arrayOf(
        MediaStore.Files.FileColumns._ID,
        MediaStore.Files.FileColumns.DISPLAY_NAME,
        MediaStore.Files.FileColumns.SIZE,
        MediaStore.Files.FileColumns.MIME_TYPE,
        MediaStore.Files.FileColumns.MEDIA_TYPE,
        MediaStore.Files.FileColumns.DURATION
    )

    val selection = "${MediaStore.Files.FileColumns.MEDIA_TYPE} \u003d ? OR ${MediaStore.Files.FileColumns.MEDIA_TYPE} \u003d ?"
    val selectionArgs = arrayOf(
        MediaStore.Files.FileColumns.MEDIA_TYPE_IMAGE.toString(),
        MediaStore.Files.FileColumns.MEDIA_TYPE_VIDEO.toString()
    )

    val sortOrder = "${MediaStore.Files.FileColumns.DATE_ADDED} DESC"

    context.contentResolver.query(
        MediaStore.Files.getContentUri("external"),
        projection,
        selection,
        selectionArgs,
        sortOrder
    )?.use { cursor ->
        val idColumn = cursor.getColumnIndexOrThrow(MediaStore.Files.FileColumns._ID)
        val nameColumn = cursor.getColumnIndexOrThrow(MediaStore.Files.FileColumns.DISPLAY_NAME)
        val sizeColumn = cursor.getColumnIndexOrThrow(MediaStore.Files.FileColumns.SIZE)
        val mimeTypeColumn = cursor.getColumnIndexOrThrow(MediaStore.Files.FileColumns.MIME_TYPE)
        val mediaTypeColumn = cursor.getColumnIndexOrThrow(MediaStore.Files.FileColumns.MEDIA_TYPE)
        val durationColumn = cursor.getColumnIndexOrThrow(MediaStore.Files.FileColumns.DURATION)

        while (cursor.moveToNext()) {
            val id = cursor.getLong(idColumn)
            val name = cursor.getString(nameColumn) ?: "unknown"
            val size = cursor.getLong(sizeColumn)
            val mimeType = cursor.getString(mimeTypeColumn) ?: "image/jpeg"
            val mediaType = cursor.getInt(mediaTypeColumn)
            val duration = if (mediaType == MediaStore.Files.FileColumns.MEDIA_TYPE_VIDEO) cursor.getLong(durationColumn) else null

            val contentUri = ContentUris.withAppendedId(
                if (mediaType == MediaStore.Files.FileColumns.MEDIA_TYPE_VIDEO)
                    MediaStore.Video.Media.EXTERNAL_CONTENT_URI
                else
                    MediaStore.Images.Media.EXTERNAL_CONTENT_URI,
                id
            )

            mediaList.add(
                LocalMedia(
                    id = id,
                    uri = contentUri.toString(),
                    name = name,
                    size = size,
                    mimeType = mimeType,
                    isVideo = mediaType == MediaStore.Files.FileColumns.MEDIA_TYPE_VIDEO,
                    duration = duration
                )
            )
        }
    }

    mediaList
}

actual suspend fun getLocalMediaBytes(uri: String): ByteArray? = withContext(Dispatchers.IO) {
    val context = AndroidContext.context
    try {
        context.contentResolver.openInputStream(android.net.Uri.parse(uri))?.use { it.readBytes() }
    } catch (e: Exception) {
        null
    }
}
