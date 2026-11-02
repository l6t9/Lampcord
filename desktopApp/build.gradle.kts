import dev.nucleusframework.desktop.application.dsl.CompressionLevel
import dev.nucleusframework.desktop.application.dsl.TargetFormat
import java.awt.RenderingHints
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.file.Files
import java.nio.file.Paths
import java.util.Properties
import javax.imageio.ImageIO
import org.gradle.jvm.toolchain.JavaLanguageVersion

plugins {
    alias(libs.plugins.kotlin.multiplatform)
    alias(libs.plugins.compose.multiplatform)
    alias(libs.plugins.compose.compiler)
    id("dev.nucleusframework") version "3.0.0-dev-202610272317"
}

val appVersion = providers.gradleProperty("appVersion").get()

val platform =
    org.gradle.internal.os.OperatingSystem.current().let { os ->
        when {
            os.isWindows -> "win"
            os.isMacOsX -> "mac"
            os.isLinux -> "linux"
            else -> "unknown"
        }
    }

val localProperties = Properties()
val localPropertiesFile = rootProject.file("local.properties")
if (localPropertiesFile.exists()) {
    localProperties.load(localPropertiesFile.inputStream())
}

kotlin {
   jvm {
   }

   sourceSets {
       getByName("jvmMain") {
           dependencies {
                implementation(project(":shared"))
                implementation(compose.desktop.currentOs)
                implementation(compose.material3)
                implementation(libs.ktor.client.cio)
                implementation(libs.kotlinx.coroutines.swing)
                implementation(libs.jna)
                implementation(libs.jna.platform)
                implementation(libs.koin.compose)

                implementation("dev.nucleusframework:nucleus.nucleus-application:${libs.versions.nucleus.get()}")
                implementation("dev.nucleusframework:nucleus.decorated-window-tao:${libs.versions.nucleus.get()}")
                implementation("dev.nucleusframework:nucleus.notification-common:${libs.versions.nucleus.get()}")
                implementation("dev.nucleusframework:nucleus.notification-linux:${libs.versions.nucleus.get()}")
                implementation("dev.nucleusframework:nucleus.notification-macos:${libs.versions.nucleus.get()}")
                implementation("dev.nucleusframework:nucleus.notification-windows:${libs.versions.nucleus.get()}")
                implementation("dev.nucleusframework:composenativetray:2.1.6")
                implementation("dev.nucleusframework:nucleus.updater-runtime:${libs.versions.nucleus.get()}")
                implementation("dev.nucleusframework:nucleus.system-color:${libs.versions.nucleus.get()}")
                implementation("dev.nucleusframework:nucleus.native-http-ktor:${libs.versions.nucleus.get()}")
                implementation("dev.nucleusframework:nucleus.energy-manager:${libs.versions.nucleus.get()}")
                implementation("dev.nucleusframework:nucleus.darkmode-detector:${libs.versions.nucleus.get()}")
           }
       }
   }
}

tasks.withType<org.gradle.api.tasks.JavaExec>().configureEach {
    workingDir = rootProject.projectDir
}

val windowsIconFile = layout.buildDirectory.file("generated/icons/logo.ico")

val generateWindowsIcon by tasks.registering {
    group = "distribution"
    description = "Builds a multi-size logo.ico from logo.png for the Windows launcher and installer."
    val source = file("src/jvmMain/resources/logo.png")

    val target = windowsIconFile.get().asFile
    inputs.file(source)
    outputs.file(target)
    doLast {
        fun resize(image: BufferedImage, size: Int): BufferedImage {
            val scaled = BufferedImage(size, size, BufferedImage.TYPE_INT_ARGB)
            val g = scaled.createGraphics()
            g.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BICUBIC)
            g.setRenderingHint(RenderingHints.KEY_RENDERING, RenderingHints.VALUE_RENDER_QUALITY)
            g.drawImage(image, 0, 0, size, size, null)
            g.dispose()
            return scaled
        }

        fun pngBytes(image: BufferedImage): ByteArray =
            ByteArrayOutputStream().also { ImageIO.write(image, "png", it) }.toByteArray()

        fun dibBytes(image: BufferedImage): ByteArray {
            val size = image.width
            val maskStride = ((size + 31) / 32) * 4
            val buffer = ByteBuffer.allocate(40 + size * size * 4 + maskStride * size).order(ByteOrder.LITTLE_ENDIAN)

            buffer.putInt(40).putInt(size).putInt(size * 2).putShort(1).putShort(32)
                .putInt(0).putInt(size * size * 4 + maskStride * size).putInt(0).putInt(0).putInt(0).putInt(0)
            for (y in size - 1 downTo 0) {
                for (x in 0 until size) {
                    val argb = image.getRGB(x, y)
                    buffer.put((argb and 0xFF).toByte())
                    buffer.put(((argb shr 8) and 0xFF).toByte())
                    buffer.put(((argb shr 16) and 0xFF).toByte())
                    buffer.put(((argb ushr 24) and 0xFF).toByte())
                }
            }

            return buffer.array()
        }

        val original = ImageIO.read(source)
        val entries = listOf(16, 20, 24, 32, 40, 48, 64, 96, 128, 256).map { size ->

            var image = original
            while (image.width / 2 >= size) image = resize(image, image.width / 2)
            image = resize(image, size)

            size to if (size == 256) pngBytes(image) else dibBytes(image)
        }
        val out = ByteArrayOutputStream()
        fun le16(v: Int) { out.write(v and 0xFF); out.write((v shr 8) and 0xFF) }
        fun le32(v: Int) { le16(v and 0xFFFF); le16((v ushr 16) and 0xFFFF) }
        le16(0); le16(1); le16(entries.size)
        var offset = 6 + 16 * entries.size
        for ((size, bytes) in entries) {
            out.write(if (size >= 256) 0 else size)
            out.write(if (size >= 256) 0 else size)
            out.write(0); out.write(0)
            le16(1); le16(32)
            le32(bytes.size); le32(offset)
            offset += bytes.size
        }
        entries.forEach { out.write(it.second) }
        target.parentFile.mkdirs()
        target.writeBytes(out.toByteArray())
    }
}

tasks.configureEach {
    if (name.startsWith("package") || name.startsWith("createDistributable") || name.startsWith("createReleaseDistributable")) {
        dependsOn(generateWindowsIcon)
    }
}

nucleus.application {
    mainClass = "me.lampu.lampcord.MainKt"
    localProperties.getProperty("compose.desktop.javaHome")?.let {
        javaHome = it
    }
    jvmArgs += listOf(
        "--enable-native-access=ALL-UNNAMED",
        "-Xmx1g",
        "-Dskiko.gpu.resourceCacheLimit=64m"
    )
    if (System.getProperty("os.name").contains("Mac")) {
        jvmArgs += listOf(
            "--add-opens=java.desktop/sun.lwawt=ALL-UNNAMED",
            "--add-opens=java.desktop/sun.lwawt.macosx=ALL-UNNAMED"
        )
    }
    providers.gradleProperty("lampcord.nmt").orNull?.let {
        if (it == "true") {
            jvmArgs += "-XX:NativeMemoryTracking=summary"
        }
    }
    nativeDistributions {
        targetFormats(

            TargetFormat.Dmg,
            TargetFormat.Nsis,
            TargetFormat.AppImage,
        )
        packageName = "Lampcord"
        packageVersion = appVersion.substringBefore('-')
        description = "Lampcord Discord Client"
        copyright = "Lampu"
        vendor = "Lampcord"
        homepage = "https://cord.lamp.delivery"
        compressionLevel = CompressionLevel.Maximum

        windows {
            iconFile.set(windowsIconFile)
            nsis {
                oneClick = false
                perMachine = false
                allowElevation = true
                allowToChangeInstallationDirectory = true
                createDesktopShortcut = true
                createStartMenuShortcut = true
                runAfterFinish = true
            }
        }

        linux {
            jvmArgs += listOf("-Djava.locale.providers=COMPAT,SPI", "-Dwebkit.disable.dmabuf.renderer=1")
            debMaintainer = "Lampu <lampu@cord.lamp.delivery>"
        }
        macOS {
            bundleID = "me.lampcord.desktop"
            dockName = "Lampcord"
            dmgPackageVersion = packageVersion
            val icns = project.file("src/jvmMain/resources/logo.icns")
            if (icns.exists()) iconFile.set(icns)
            infoPlist {
                extraKeysRawXml = """
                    <key>NSMicrophoneUsageDescription</key>
                    <string>Lampcord uses your microphone for voice calls you join.</string>
                    <key>NSLocalNetworkUsageDescription</key>
                    <string>Lampcord uses the local network to establish voice calls.</string>
                    <key>NSCameraUsageDescription</key>
                    <string>Lampcord uses your camera for video calls you join.</string>
                    <key>NSHighResolutionCapable</key>
                    <true/>
                """.trimIndent()
            }
        }
    }

    buildTypes {
        release {
            proguard {
                isEnabled.set(false)
            }
        }
    }

    graalvm {
        isEnabled.set(true)
        javaLanguageVersion.set(25)
        imageName.set("lampcord")
        buildArgs.add("-O2")
    }
}

val runtimeImageDir = layout.buildDirectory.dir("runtime-image")

val createLinuxRuntimeImage by tasks.registering(Exec::class) {
    group = "distribution"
    description = "Links the trimmed JRE the Linux packages bundle next to the jar."

    val launcher =
        javaToolchains.launcherFor { languageVersion.set(JavaLanguageVersion.of(21)) }

    val jdkHome = launcher.get().executablePath.asFile.parentFile.parentFile.absolutePath
    executable = File(jdkHome, "bin/java").absolutePath

    doFirst { runtimeImageDir.get().asFile.deleteRecursively() }
    args = listOf(
        "-m",
        "jdk.jlink/jdk.tools.jlink.internal.Main",
        "--add-modules",
        listOf(
            "java.base",
            "java.datatransfer",
            "java.desktop",
            "java.logging",
            "java.net.http",
            "java.prefs",
            "java.xml",
            "jdk.accessibility",
            "jdk.crypto.ec",
            "jdk.unsupported",
        ).joinToString(","),
        "--strip-debug",
        "--no-header-files",
        "--no-man-pages",
        "--compress=zip-9",
        "--output",
        runtimeImageDir.get().asFile.absolutePath,
    )
}

tasks.register("patchLinuxLauncher") {
    group = "distribution"
    description = "Wraps the native launcher with a shell script that sets Wayland/cursor env vars."
    notCompatibleWithConfigurationCache("File operations are not cached.")
    dependsOn("createDistributable")
    onlyIf { platform == "linux" }

    val packageAppDir = file("${layout.buildDirectory.get()}/compose/binaries/main/app/Lampcord")

    doLast {
        val binDir = packageAppDir.resolve("bin")
        val launcher = binDir.resolve("Lampcord")
        val realBinary = binDir.resolve("Lampcord.bin")
        val libAppDir = packageAppDir.resolve("lib/app")
        val cfgFile = libAppDir.resolve("Lampcord.cfg")
        val newCfgFile = binDir.resolve("Lampcord.bin.cfg")

        if (!launcher.exists()) {
            logger.warn("patchLinuxLauncher: ${launcher.absolutePath} not found, skipping.")
            return@doLast
        }
        if (realBinary.exists()) {
            logger.info("patchLinuxLauncher: already patched, skipping.")
            return@doLast
        }

        launcher.renameTo(realBinary)

        if (cfgFile.exists()) {
            cfgFile.copyTo(newCfgFile, overwrite = true)
            cfgFile.copyTo(libAppDir.resolve("Lampcord.bin.cfg"), overwrite = true)
        }

        launcher.writeText(
            "#!/bin/sh\n" +
                """
# OpenGL rendering
export SKIKO_RENDER_API="${'$'}{SKIKO_RENDER_API:-OPENGL}"

# Only force the Wayland backends when the session is actually Wayland.
if [ "${'$'}XDG_SESSION_TYPE" = "wayland" ]; then
    export SKIKO_WAYLAND="${'$'}{SKIKO_WAYLAND:-1}"
    export GDK_BACKEND="${'$'}{GDK_BACKEND:-wayland}"
fi

# Prevent AWT from reparenting (needed for some tiling WMs)
export _JAVA_AWT_WM_NONREPARENTING="${'$'}{_JAVA_AWT_WM_NONREPARENTING:-1}"

# Limit glibc malloc arenas to reduce memory fragmentation
export MALLOC_ARENA_MAX="${'$'}{MALLOC_ARENA_MAX:-4}"
export MALLOC_MMAP_THRESHOLD_="${'$'}{MALLOC_MMAP_THRESHOLD_:-131072}"

# WebView compatibility (fix black screen / GBM errors)
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1

if [ -z "${'$'}XCURSOR_THEME" ]; then
    CURSOR_THEME="$(hyprctl getoption cursor:theme 2>/dev/null | sed -n 's/.*string: *//p')"
    if [ -z "${'$'}CURSOR_THEME" ]; then
        CURSOR_THEME="$(gsettings get org.gnome.desktop.interface cursor-theme 2>/dev/null | tr -d "'")"
    fi
    if [ -n "${'$'}CURSOR_THEME" ]; then
        export XCURSOR_THEME="${'$'}CURSOR_THEME"
        export GDK_CURSOR_THEME="${'$'}CURSOR_THEME"
    fi
fi

DIR="$(cd "$(dirname "$(readlink -f "${'$'}0")")" && pwd)"
exec "${'$'}DIR/Lampcord.bin" "${'$'}@"
                """.trimIndent(),
        )
        launcher.setExecutable(true, false)
        logger.lifecycle("patchLinuxLauncher: wrapped launcher with env-var setup.")
    }
}

tasks.register("createAppImageLocal") {
    group = "distribution"
    description = "Creates an AppImage using local appimagetool."
    notCompatibleWithConfigurationCache("This task launches appimagetool directly.")
    dependsOn("patchLinuxLauncher")

    onlyIf { platform == "linux" }

    val appImageTool = rootProject.file("tools/appimagetool.AppImage")
    val packageAppDir = file("${layout.buildDirectory.get()}/compose/binaries/main/app/Lampcord")
    val appDir = file("${layout.buildDirectory.get()}/appimage/Lampcord.AppDir")
    val outFile = rootProject.file("dist/Lampcord.AppImage")

    doFirst {
        if (!appImageTool.exists()) {
            throw GradleException("tools/appimagetool.AppImage not found.")
        }

        if (!packageAppDir.isDirectory) {
            throw GradleException("Application directory not found at ${packageAppDir.absolutePath}")
        }
        appDir.deleteRecursively()
        if (!packageAppDir.copyRecursively(appDir, overwrite = true)) {
            throw GradleException("Failed to create the AppImage staging directory")
        }
        packageAppDir
            .walkTopDown()
            .filter { it.isFile && it.canExecute() }
            .forEach { source -> File(appDir, source.relativeTo(packageAppDir).path).setExecutable(true, false) }

        val desktopFile = file("${appDir.absolutePath}/Lampcord.desktop")
        desktopFile.parentFile.mkdirs()
        desktopFile.writeText(
            """
            [Desktop Entry]
            Type=Application
            Name=Lampcord
            Exec=bin/Lampcord
            Icon=Lampcord
            Comment=Lampcord Discord Client
            Categories=Network;Chat;
            Terminal=false
            StartupWMClass=Lampcord
            """.trimIndent(),
        )

        val appRunFile = file("${appDir.absolutePath}/AppRun")
        appRunFile.writeText(
            "#!/bin/sh\n" +
                """
export MALLOC_ARENA_MAX=4
export MALLOC_MMAP_THRESHOLD_="${'$'}{MALLOC_MMAP_THRESHOLD_:-131072}"
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1
exec "${'$'}APPDIR/bin/Lampcord" "${'$'}@"
                """.trimIndent(),
        )
        appRunFile.setExecutable(true, false)

        val appIconFile = file("${appDir.absolutePath}/Lampcord.png")

        val sourceIcon = file("src/jvmMain/resources/logo.png").takeIf { it.exists() }
            ?: file("../shared/src/commonMain/composeResources/lampcord.shared.generated.resources/drawable/icon.png").takeIf { it.exists() }
            ?: file("${layout.buildDirectory.get()}/compose/default-resources/1.11.1/default-icon-linux.png").takeIf { it.exists() }

        if (sourceIcon != null) {
            sourceIcon.copyTo(appIconFile, overwrite = true)
            val themedIcon = file("${appDir.absolutePath}/usr/share/icons/hicolor/1024x1024/apps/lampcord.png")
            themedIcon.parentFile.mkdirs()
            sourceIcon.copyTo(themedIcon, overwrite = true)
        } else {

            appIconFile.writeBytes(ByteArray(0))
        }

        val resourcesLink = file("${appDir.absolutePath}/resources")
        if (!resourcesLink.exists()) {
            Files.createSymbolicLink(
                resourcesLink.toPath(),
                Paths.get("lib/app/resources"),
            )
        }

        outFile.parentFile.mkdirs()
    }

    doLast {
        val processBuilder =
            ProcessBuilder(appImageTool.absolutePath, "--appimage-extract-and-run", appDir.absolutePath, outFile.absolutePath)
        processBuilder.environment().remove("SOURCE_DATE_EPOCH")
        val process = processBuilder.inheritIO().start()

        val exitCode = process.waitFor()
        if (exitCode != 0) {
            throw GradleException("appimagetool failed with exit code ${'$'}exitCode")
        }

        outFile.setExecutable(true, false)
        logger.lifecycle("AppImage created at: ${'$'}{outFile.absolutePath}")
    }
}

tasks.matching { it.name == "packageAppImage" }.configureEach {
    dependsOn("createAppImageLocal")
}
