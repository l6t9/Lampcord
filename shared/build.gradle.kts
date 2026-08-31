plugins {
    alias(libs.plugins.kotlin.multiplatform)
    alias(libs.plugins.android.kmp.library)
    alias(libs.plugins.compose.multiplatform)
    alias(libs.plugins.compose.compiler)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.kotlin.ksp)
}

compose.resources {
    publicResClass = true
}

kotlin {
    targets.all {
        compilations.all {
            compileTaskProvider.configure {
                compilerOptions {
                    freeCompilerArgs.add("-Xexpect-actual-classes")
                }
            }
        }
    }

    android {
        namespace = "me.lampu.lampcord.shared"
        compileSdk = 37
        minSdk = 24
    }
    
    jvm("desktop")
    
    listOf(
        iosArm64(),
        iosSimulatorArm64()
    ).forEach {
        it.binaries.framework {
            baseName = "shared"
            isStatic = true
        }
    }
    
    applyDefaultHierarchyTemplate()
    
    sourceSets {
        commonMain.dependencies {
            implementation(libs.compose.runtime)
            implementation(libs.compose.foundation)
            implementation(libs.compose.material3)
            implementation(libs.compose.ui)
            implementation(libs.compose.components.resources)
            
            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.kotlinx.serialization.json)
            implementation(libs.kotlinx.datetime)
            
            implementation(libs.ktor.client.core)
            implementation(libs.ktor.client.cio)
            implementation(libs.ktor.client.content.negotiation)
            implementation(libs.ktor.serialization.kotlinx.json)
            implementation(libs.ktor.client.websockets)
            implementation(libs.slf4j.simple)
            implementation(libs.ktor.client.logging)
            
            api(libs.koin.core)
            implementation(libs.koin.compose)
            
            implementation(libs.multiplatform.settings)
            implementation(libs.multiplatform.settings.no.arg)
            
            api("androidx.navigation3:navigation3-runtime:1.2.0-alpha07")

            implementation(libs.room.runtime)
            implementation(libs.sqlite.bundled)
            
            implementation(libs.materialKolor)
            implementation(libs.qrcode.kotlin)
            
            implementation(libs.coil.compose)
            implementation(libs.coil.network.ktor)
        }
        
        getByName("androidMain") {
            dependencies {
                implementation(libs.androidx.appcompat)
                implementation(libs.androidx.activity.compose)
                implementation(libs.androidx.media3.exoplayer)
                implementation(libs.androidx.media3.ui)
                implementation(libs.androidx.security.crypto)
                
                implementation(libs.androidx.navigation3.ui)
                implementation("io.coil-kt.coil3:coil-gif:${libs.versions.coil.get()}")
            }
        }
        
        getByName("desktopMain") {
            dependencies {
                implementation(compose.desktop.currentOs)
                implementation(libs.vlcj)
                
                implementation("io.ktor:ktor-server-core:${libs.versions.ktor.get()}")
                implementation("io.ktor:ktor-server-netty:${libs.versions.ktor.get()}")
                implementation("io.ktor:ktor-server-websockets:${libs.versions.ktor.get()}")

                val javacppPlatform = System.getProperty("org.bytedeco.javacpp.platform") ?: run {
                    val osName = System.getProperty("os.name").lowercase()
                    val osArch = System.getProperty("os.arch").lowercase()
                    when {
                        osName.contains("mac") -> if (osArch == "aarch64" || osArch == "arm64") "macosx-arm64" else "macosx-x86_64"
                        osName.contains("win") -> "windows-x86_64"
                        else -> if (osArch == "aarch64" || osArch == "arm64") "linux-arm64" else "linux-x86_64"
                    }
                }
                implementation("org.bytedeco:ffmpeg:${libs.versions.ffmpegPlatform.get()}")
                implementation("org.bytedeco:ffmpeg:${libs.versions.ffmpegPlatform.get()}:$javacppPlatform")

                implementation(libs.jna.core)
                implementation(libs.jna.platform)
            }
        }
    }
}

dependencies {
    add("kspAndroid", libs.room.compiler)
    add("kspDesktop", libs.room.compiler)
    add("kspIosArm64", libs.room.compiler)
    add("kspIosSimulatorArm64", libs.room.compiler)
}
