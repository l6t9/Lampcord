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
           targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
           packageName = "Lampcord"
           packageVersion = "1.0.0"
           
           linux {
               jvmArgs += listOf("-Djava.locale.providers=COMPAT,SPI")
           }
       }
   }
}
