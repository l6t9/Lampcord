plugins {
    alias(libs.plugins.kotlin.multiplatform)
    alias(libs.plugins.android.kmp.library)
    alias(libs.plugins.compose.multiplatform)
    alias(libs.plugins.compose.compiler)
    alias(libs.plugins.kotlin.serialization)
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
            
            api(libs.koin.core)
            implementation(libs.koin.compose)
            
            implementation(libs.multiplatform.settings)
            implementation(libs.multiplatform.settings.no.arg)
            
            implementation(libs.materialKolor)
            
            implementation(libs.coil.compose)
            implementation(libs.coil.network.ktor)
        }
        
        getByName("androidMain") {
            dependencies {
                implementation(libs.androidx.appcompat)
                implementation(libs.androidx.activity.compose)
                implementation(libs.androidx.media3.exoplayer)
                implementation(libs.androidx.media3.ui)
                implementation(libs.materii.panels)
                implementation("androidx.security:security-crypto:1.1.0-alpha06")
            }
        }
        
        getByName("desktopMain") {
            dependencies {
                implementation(compose.desktop.currentOs)
                implementation(libs.vlcj)
                implementation(libs.ffmpeg.platform)
                implementation(libs.jna.core)
                implementation(libs.jna.platform)
            }
        }
    }
}
