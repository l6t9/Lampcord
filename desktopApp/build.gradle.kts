import org.jetbrains.compose.desktop.application.dsl.TargetFormat

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
               implementation(libs.jna.core)
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
           
           linux {
               jvmArgs += listOf("-Djava.locale.providers=COMPAT,SPI")
           }
       }
   }
}

tasks.register("patchLinuxLauncher") {
    group = "distribution"
    description = "Wraps the native launcher with a shell script that sets Wayland/cursor env vars."
    
    val packageAppDir = file("${layout.buildDirectory.get()}/compose/binaries/main/app/Lampcord")

    doLast {
        val binDir = packageAppDir.resolve("bin")
        val launcher = binDir.resolve("Lampcord")
        val realBinary = binDir.resolve("Lampcord.bin")
        val libAppDir = packageAppDir.resolve("lib/app")
        val cfgFile = libAppDir.resolve("Lampcord.cfg")
        val newCfgFile = binDir.resolve("Lampcord.bin.cfg")

        if (!launcher.exists()) {
            return@doLast
        }
        if (realBinary.exists()) {
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
export SKIKO_RENDER_API="${'$'}{SKIKO_RENDER_API:-OPENGL}"
export SKIKO_WAYLAND="${'$'}{SKIKO_WAYLAND:-1}"
export GDK_BACKEND="${'$'}{GDK_BACKEND:-wayland}"
export _JAVA_AWT_WM_NONREPARENTING="${'$'}{_JAVA_AWT_WM_NONREPARENTING:-1}"
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
    }
}

tasks.matching { it.name == "packageAppImage" }.configureEach {
    finalizedBy("patchLinuxLauncher")
}
