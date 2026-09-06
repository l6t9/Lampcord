import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import java.nio.file.Files
import java.nio.file.Paths
import java.util.Properties

plugins {
   alias(libs.plugins.kotlin.multiplatform)
   alias(libs.plugins.compose.multiplatform)
   alias(libs.plugins.compose.compiler)
}

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
               implementation(libs.ktor.client.cio)
               implementation(libs.kotlinx.coroutines.swing)
               implementation(libs.jna)
               implementation(libs.jna.platform)
               implementation(libs.koin.compose)
           }
       }
   }
}

tasks.withType<org.gradle.api.tasks.JavaExec>().configureEach {
    workingDir = rootProject.projectDir
    environment("MALLOC_ARENA_MAX", "4")
    if (platform == "linux") {
        environment("LC_NUMERIC", "C")
        environment("GDK_BACKEND", "wayland")
        environment("_JAVA_AWT_WM_NONREPARENTING", "1")
        environment("SKIKO_RENDER_API", "OPENGL")
        environment("SKIKO_WAYLAND", "1")
    }
}

compose.desktop {
   application {
       mainClass = "me.lampu.lampcord.MainKt"
       localProperties.getProperty("compose.desktop.javaHome")?.let {
           javaHome = it
       }
       jvmArgs += listOf(
           "-Dsun.java2d.uiScale.enabled=true",
           "--enable-native-access=ALL-UNNAMED",
           "-XX:NativeMemoryTracking=summary",
           "-Xmx512m"
       )
       nativeDistributions {
           targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb, TargetFormat.AppImage)
           packageName = "Lampcord"
           packageVersion = "1.0.0"
           description = "Lampcord Discord Client"
           copyright = "Lampu"
           vendor = "Lampcord"
           
           linux {
               jvmArgs += listOf("-Djava.locale.providers=COMPAT,SPI")
           }
       }
   }
}

tasks.register("patchLinuxLauncher") {
    group = "distribution"
    description = "Wraps the native launcher with a shell script that sets Wayland/cursor env vars."
    notCompatibleWithConfigurationCache("File operations are not cached.")
    dependsOn("createDistributable")
    
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

        // Rename the real binary
        launcher.renameTo(realBinary)

        // Copy the config file next to the renamed binary AND in lib/app (launcher looks for <binary>.cfg)
        if (cfgFile.exists()) {
            cfgFile.copyTo(newCfgFile, overwrite = true)
            cfgFile.copyTo(libAppDir.resolve("Lampcord.bin.cfg"), overwrite = true)
        }

        launcher.writeText(
            "#!/bin/sh\n" +
                """
# OpenGL rendering
export SKIKO_RENDER_API="${'$'}{SKIKO_RENDER_API:-OPENGL}"

# Wayland support in Skiko
export SKIKO_WAYLAND="${'$'}{SKIKO_WAYLAND:-1}"

# Use Wayland backend for GTK/GDK components
export GDK_BACKEND="${'$'}{GDK_BACKEND:-wayland}"

# Prevent AWT from reparenting (needed for some tiling WMs)
export _JAVA_AWT_WM_NONREPARENTING="${'$'}{_JAVA_AWT_WM_NONREPARENTING:-1}"

# Limit glibc malloc arenas to reduce memory fragmentation
export MALLOC_ARENA_MAX="${'$'}{MALLOC_ARENA_MAX:-4}"

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
    description = "Creates an AppImage using local appimagetool (replicated from Metrolist-KMP)."
    notCompatibleWithConfigurationCache("This task launches appimagetool directly.")
    dependsOn("patchLinuxLauncher")

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
            """.trimIndent(),
        )

        val appRunFile = file("${appDir.absolutePath}/AppRun")
        appRunFile.writeText(
            "#!/bin/sh\n" +
                """
export MALLOC_ARENA_MAX=4
exec "${'$'}APPDIR/bin/Lampcord" "${'$'}@"
                """.trimIndent(),
        )
        appRunFile.setExecutable(true, false)

        val appIconFile = file("${appDir.absolutePath}/Lampcord.png")
        // Try to find an icon
        val sourceIcon = file("src/jvmMain/resources/logo.png").takeIf { it.exists() }
            ?: file("../shared/src/commonMain/composeResources/lampcord.shared.generated.resources/drawable/icon.png").takeIf { it.exists() }
            ?: file("${layout.buildDirectory.get()}/compose/default-resources/1.11.1/default-icon-linux.png").takeIf { it.exists() }

        if (sourceIcon != null) {
            sourceIcon.copyTo(appIconFile, overwrite = true)
        } else {
            // Create a dummy if still not found to prevent appimagetool failure
            appIconFile.writeBytes(ByteArray(0))
        }

        // Create symlink so that $APPDIR/resources points to lib/app/resources
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
