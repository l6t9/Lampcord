package me.lampu.lampcord.shared.utils

import android.content.ContentUris
import android.os.Build
import android.provider.MediaStore
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import me.lampu.lampcord.shared.model.LocalMedia
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import android.content.Context
import androidx.room.Room
import androidx.room.RoomDatabase
import me.lampu.lampcord.shared.database.AppDatabase
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import java.net.URL
import java.net.URLConnection
import android.app.DownloadManager
import android.net.Uri
import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.http.isSuccess
import me.lampu.lampcord.shared.api.RestClient
import me.lampu.lampcord.shared.api.standardHeaders

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
    val restClient: RestClient by inject()
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

actual suspend fun getLocalFiles(): List<LocalMedia> = withContext(Dispatchers.IO) {
    val fileList = mutableListOf<LocalMedia>()
    val context = AndroidContext.context
    
    // 1. Replicate Discord 126.21's broad MediaStore.Files search
    try {
        val projection = arrayOf(
            MediaStore.Files.FileColumns._ID,
            MediaStore.Files.FileColumns.DISPLAY_NAME,
            MediaStore.Files.FileColumns.SIZE,
            MediaStore.Files.FileColumns.MIME_TYPE,
            MediaStore.Files.FileColumns.DATA
        )

        // Discord 126.21 doesn\u0027t filter out media types in its main query, it just queries MediaStore.Files.getContentUri("external")
        context.contentResolver.query(
            MediaStore.Files.getContentUri("external"),
            projection,
            null,
            null,
            "${MediaStore.Files.FileColumns.DATE_ADDED} DESC"
        )?.use { cursor ->
            val idColumn = cursor.getColumnIndexOrThrow(MediaStore.Files.FileColumns._ID)
            val nameColumn = cursor.getColumnIndexOrThrow(MediaStore.Files.FileColumns.DISPLAY_NAME)
            val sizeColumn = cursor.getColumnIndexOrThrow(MediaStore.Files.FileColumns.SIZE)
            val mimeTypeColumn = cursor.getColumnIndexOrThrow(MediaStore.Files.FileColumns.MIME_TYPE)
            val dataColumn = cursor.getColumnIndex(MediaStore.Files.FileColumns.DATA)

            while (cursor.moveToNext()) {
                val id = cursor.getLong(idColumn)
                val name = cursor.getString(nameColumn) ?: "unknown"
                val size = cursor.getLong(sizeColumn)
                val mimeType = cursor.getString(mimeTypeColumn) ?: "application/octet-stream"
                val data = if (dataColumn != -1) cursor.getString(dataColumn) else null
                
                if (data != null && java.io.File(data).isDirectory) continue

                // Avoid duplicates of images/videos already in the media tab
                val isMedia = mimeType.startsWith("image/") || mimeType.startsWith("video/")
                if (isMedia) continue

                val contentUri = ContentUris.withAppendedId(
                    MediaStore.Files.getContentUri("external"),
                    id
                )

                fileList.add(LocalMedia(id, contentUri.toString(), name, size, mimeType, false))
                if (fileList.size >= 150) break
            }
        }
    } catch (e: Exception) { }

    // 2. Manual recursive crawl of high-yield folders if list is small (fallback)
    if (fileList.size < 50) {
        val searchDirs = listOfNotNull(
            android.os.Environment.getExternalStoragePublicDirectory(android.os.Environment.DIRECTORY_DOWNLOADS),
            android.os.Environment.getExternalStoragePublicDirectory(android.os.Environment.DIRECTORY_DOCUMENTS),
            @Suppress("DEPRECATION")
            android.os.Environment.getExternalStorageDirectory()
        )

        val queue = java.util.ArrayDeque<java.io.File>()
        searchDirs.forEach { if (it.exists() && it.isDirectory) queue.add(it) }

        val scannedDirs = mutableSetOf<String>()
        var crawlCount = 0

        while (queue.isNotEmpty() && fileList.size < 200 && scannedDirs.size < 100) {
            val dir = queue.removeFirst()
            val path = try { dir.absolutePath } catch(e: Exception) { continue }
            if (path in scannedDirs) continue
            scannedDirs.add(path)

            dir.listFiles()?.forEach { file ->
                if (file.isHidden) return@forEach
                if (file.isDirectory) {
                    if (queue.size < 200 && !path.contains("/Android/data")) queue.add(file)
                } else {
                    val uri = android.net.Uri.fromFile(file).toString()
                    if (fileList.none { it.uri == uri || it.name == file.name }) {
                        fileList.add(LocalMedia(file.hashCode().toLong(), uri, file.name, file.length(), "application/octet-stream", false))
                        crawlCount++
                    }
                }
            }
        }
    }

    fileList.sortWith(compareByDescending { 
        if (it.uri.startsWith("file://")) {
            java.io.File(android.net.Uri.parse(it.uri).path ?: "").lastModified()
        } else {
            Long.MAX_VALUE 
        }
    })

    fileList
}

actual suspend fun getLocalMediaBytes(uri: String): ByteArray? = withContext(Dispatchers.IO) {
    val context = AndroidContext.context
    try {
        context.contentResolver.openInputStream(android.net.Uri.parse(uri))?.use { it.readBytes() }
    } catch (e: Exception) {
        null
    }
}

actual fun getDatabaseBuilder(): RoomDatabase.Builder<AppDatabase> {
    val appContext = AndroidContext.context.applicationContext
    val dbFile = appContext.getDatabasePath("lampcord.db")
    return Room.databaseBuilder<AppDatabase>(
        context = appContext,
        name = dbFile.absolutePath
    )
}

actual fun getAppStoragePath(): String = AndroidContext.context.filesDir.absolutePath

actual fun writeInternalFile(name: String, content: String) {
    val dir = AndroidContext.context.filesDir
    if (!dir.exists()) dir.mkdirs()
    java.io.File(dir, name).writeText(content)
}

actual fun writeInternalBytes(name: String, content: ByteArray) {
    val dir = AndroidContext.context.filesDir
    if (!dir.exists()) dir.mkdirs()
    java.io.File(dir, name).writeBytes(content)
}

actual fun readInternalFile(name: String): String? {
    val file = java.io.File(AndroidContext.context.filesDir, name)
    return if (file.exists()) file.readText() else null
}

actual fun checkInternalFileExists(name: String): Boolean {
    return java.io.File(AndroidContext.context.filesDir, name).exists()
}

actual fun getInternalFilePath(name: String): String {
    return java.io.File(AndroidContext.context.filesDir, name).absolutePath
}

actual fun showToast(text: String) {
    val context = AndroidContext.context
    val mainHandler = android.os.Handler(android.os.Looper.getMainLooper())
    mainHandler.post {
        android.widget.Toast.makeText(context, text, android.widget.Toast.LENGTH_SHORT).show()
    }
}

actual fun restartApp() {
    val context = AndroidContext.context
    val packageManager = context.packageManager
    val intent = packageManager.getLaunchIntentForPackage(context.packageName)
    val componentName = intent?.component
    val mainIntent = android.content.Intent.makeRestartActivityTask(componentName)
    context.startActivity(mainIntent)
    Runtime.getRuntime().exit(0)
}

@Composable
actual fun RequestMediaPermissions(onResult: (Boolean) -> Unit) {
    val launcher = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { results ->
        onResult(results.values.all { it })
    }

    LaunchedEffect(Unit) {
        val permissions = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            arrayOf(
                android.Manifest.permission.READ_MEDIA_IMAGES,
                android.Manifest.permission.READ_MEDIA_VIDEO,
                android.Manifest.permission.CAMERA
            )
        } else {
            arrayOf(
                android.Manifest.permission.READ_EXTERNAL_STORAGE,
                android.Manifest.permission.WRITE_EXTERNAL_STORAGE,
                android.Manifest.permission.CAMERA
            )
        }
        launcher.launch(permissions)
    }
}

actual suspend fun fetchUrlBytes(url: String): ByteArray? = withContext(Dispatchers.IO) {
    try {
        URL(url).readBytes()
    } catch (e: Exception) {
        null
    }
}

actual fun base64Encode(bytes: ByteArray): String {
    return android.util.Base64.encodeToString(bytes, android.util.Base64.NO_WRAP)
}

actual suspend fun downloadToDownloads(url: String, filename: String): Boolean = withContext(Dispatchers.IO) {
    val ctx = AndroidContext.context

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        return@withContext downloadToMediaStoreDownloads(ctx, url, filename)
    }

    val systemService = ctx.getSystemService(Context.DOWNLOAD_SERVICE)
    val dm = if (systemService is DownloadManager) systemService else return@withContext false

    val uri = try {
        Uri.parse(url)
    } catch (e: Exception) {
        return@withContext false
    }

    // Match Discord 126.21 sanitization precisely
    val title = filename.ifBlank { uri.lastPathSegment } ?: "file"
    val sanitizedTitle = title.replace(Regex("[/\\\\]"), "_")

    val request = try {
        DownloadManager.Request(uri)
            .setTitle(sanitizedTitle)
            .setDescription(sanitizedTitle)
            .setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
            .setDestinationInExternalPublicDir(android.os.Environment.DIRECTORY_DOWNLOADS, sanitizedTitle)
    } catch (e: Exception) {
        return@withContext false
    }

    val extension = sanitizedTitle.substringAfterLast('.', "").lowercase()
    if (extension.isNotEmpty()) {
        val mimeType = android.webkit.MimeTypeMap.getSingleton().getMimeTypeFromExtension(extension)
        if (mimeType != null) {
            request.setMimeType(mimeType)
        }
    }

    @Suppress("DEPRECATION")
    request.allowScanningByMediaScanner()

    val deferred = kotlinx.coroutines.CompletableDeferred<Boolean>()
    var downloadId = -1L
    
    val filter = android.content.IntentFilter(DownloadManager.ACTION_DOWNLOAD_COMPLETE)
    val receiver = object : android.content.BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: android.content.Intent?) {
            val id = intent?.getLongExtra(DownloadManager.EXTRA_DOWNLOAD_ID, -1L) ?: -1L
            if (id != -1L && id == downloadId) {
                try {
                    val q = DownloadManager.Query().setFilterById(id)
                    val cursor = dm.query(q)
                    var success = false
                    cursor?.use {
                        if (it.moveToFirst()) {
                            val status = it.getInt(it.getColumnIndexOrThrow(DownloadManager.COLUMN_STATUS))
                            success = status == DownloadManager.STATUS_SUCCESSFUL
                        }
                    }
                    if (!deferred.isCompleted) deferred.complete(success)
                } catch (e: Exception) {
                    if (!deferred.isCompleted) deferred.complete(false)
                } finally {
                    try { ctx.unregisterReceiver(this) } catch (_: Exception) {}
                }
            }
        }
    }

    withContext(Dispatchers.Main) {
        if (Build.VERSION.SDK_INT >= 34) {
            ctx.registerReceiver(receiver, filter, Context.RECEIVER_EXPORTED)
        } else {
            ctx.registerReceiver(receiver, filter)
        }
    }

    return@withContext try {
        downloadId = dm.enqueue(request)
        if (downloadId <= 0) {
            withContext(Dispatchers.Main) { try { ctx.unregisterReceiver(receiver) } catch (_: Exception) {} }
            false
        } else {
            // Discord 126.21 patience: up to 10 minutes for a download
            withTimeoutOrNull(600000L) {
                deferred.await()
            } ?: run {
                withContext(Dispatchers.Main) { try { ctx.unregisterReceiver(receiver) } catch (_: Exception) {} }
                false
            }
        }
    } catch (e: Exception) {
        withContext(Dispatchers.Main) { try { ctx.unregisterReceiver(receiver) } catch (_: Exception) {} }
        false
    }
}

private suspend fun downloadToMediaStoreDownloads(context: Context, url: String, filename: String): Boolean {
    val resolver = context.contentResolver
    val title = filename.ifBlank { Uri.parse(url).lastPathSegment ?: "download" }
        .replace(Regex("[/\\\\]"), "_")
    val mimeType = android.webkit.MimeTypeMap.getSingleton()
        .getMimeTypeFromExtension(title.substringAfterLast('.', "").lowercase())
        ?: URLConnection.guessContentTypeFromName(title)
        ?: "application/octet-stream"

    val values = android.content.ContentValues().apply {
        put(android.provider.MediaStore.Downloads.DISPLAY_NAME, title)
        put(android.provider.MediaStore.Downloads.MIME_TYPE, mimeType)
        put(android.provider.MediaStore.Downloads.RELATIVE_PATH, android.os.Environment.DIRECTORY_DOWNLOADS)
        put(android.provider.MediaStore.Downloads.IS_PENDING, 1)
    }
    val outputUri = resolver.insert(android.provider.MediaStore.Downloads.EXTERNAL_CONTENT_URI, values) ?: return false

    return try {
        val response = AndroidContext.restClient.httpClient.get(url) {
            standardHeaders(AndroidContext.restClient)
        }
        if (!response.status.isSuccess()) throw java.io.IOException("Download request failed")
        val bytes = response.body<ByteArray>()
        if (bytes.isEmpty()) throw java.io.IOException("Download was empty")
        val isApk = title.endsWith(".apk", ignoreCase = true)
        if (isApk && (bytes.size < 2 || bytes[0] != 'P'.code.toByte() || bytes[1] != 'K'.code.toByte())) {
            throw java.io.IOException("Downloaded APK content is invalid")
        }
        resolver.openOutputStream(outputUri).use { output ->
            if (output == null) throw java.io.IOException("Unable to open Downloads output")
            output.write(bytes)
        }
        val completed = android.content.ContentValues().apply {
            put(android.provider.MediaStore.Downloads.IS_PENDING, 0)
        }
        resolver.update(outputUri, completed, null, null)
        true
    } catch (_: Exception) {
        resolver.delete(outputUri, null, null)
        false
    }
}

actual suspend fun ensureUniqueDownloadFilename(desiredName: String): String = withContext(Dispatchers.IO) {
    desiredName.replace(Regex("[/\\\\]"), "_")
}

actual fun openDownloadsFolderAndSelect(filename: String) {
    try {
        val ctx = AndroidContext.context
        val downloads = android.os.Environment.getExternalStoragePublicDirectory(android.os.Environment.DIRECTORY_DOWNLOADS)
        val uri = android.net.Uri.fromFile(downloads)
        val intent = android.content.Intent(android.content.Intent.ACTION_VIEW)
        intent.setDataAndType(uri, "resource/folder")
        intent.addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
        try {
            ctx.startActivity(intent)
        } catch (e: Exception) {
            // fallback: open downloads directory with generic view
            val i2 = android.content.Intent(android.content.Intent.ACTION_GET_CONTENT)
            i2.setDataAndType(uri, "*/*")
            i2.addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
            try { ctx.startActivity(i2) } catch (_: Exception) {}
        }
    } catch (_: Exception) {}
}
