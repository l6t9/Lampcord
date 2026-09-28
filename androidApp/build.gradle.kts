import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.google.services)
    alias(libs.plugins.kotlin.multiplatform) apply false
    alias(libs.plugins.compose.multiplatform)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "me.lampu.lampcord"
    compileSdk = 37

    val keystoreProperties = Properties().apply {
        val propertiesFile = rootProject.file("local.properties")
        if (propertiesFile.exists()) {
            load(propertiesFile.inputStream())
        }
    }

    fun signingProperty(propertyName: String, environmentName: String): String? =
        (keystoreProperties[propertyName] as String?)?.takeIf { it.isNotBlank() }
            ?: System.getenv(environmentName)?.takeIf { it.isNotBlank() }

    defaultConfig {
        applicationId = "me.lampu.lampcord"
        minSdk = 24
        targetSdk = 37
        
        val appVersion = project.property("appVersion") as String
        versionName = appVersion
        // Extract version code from appVersion if possible, or just keep it as 1 for now
        // Metrolist might have a more complex way.
        versionCode = 1

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        // All four by default so the app installs on desktop emulators, whose system
        // images are x86 or x86_64 even though no real device uses those ABIs. Release
        // builds narrow this to the two ARM ABIs that every actual user needs.
        ndk { abiFilters += (providers.gradleProperty("voiceAbis").orNull ?: "armeabi-v7a,arm64-v8a,x86,x86_64").split(",") }
        externalNativeBuild {
            cmake {
                arguments += listOf("-DANDROID_STL=c++_static", "-DCMAKE_INTERPROCEDURAL_OPTIMIZATION=OFF")
                targets += "lampcord_voice"
            }
        }
    }

    splits {
        abi {
            isEnable = true
            reset()
            include("arm64-v8a", "armeabi-v7a", "x86_64", "x86")
            isUniversalApk = true
        }
    }

    signingConfigs {
        create("release") {
            val storeFileProperty = signingProperty("release.storeFile", "KEYSTORE_PATH")
            if (storeFileProperty != null) {
                storeFile = rootProject.file(storeFileProperty)
                storePassword = signingProperty("release.storePassword", "KEYSTORE_PASSWORD")
                keyAlias = signingProperty("release.keyAlias", "KEY_ALIAS")
                keyPassword = signingProperty("release.keyPassword", "KEY_PASSWORD")
            }
        }
    }

    val hasReleaseSigning = signingProperty("release.storeFile", "KEYSTORE_PATH") != null

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            // CI can build an installable optimized APK without exposing a
            // release keystore. A configured keystore is still preferred.
            signingConfig = if (hasReleaseSigning) {
                signingConfigs.getByName("release")
            } else {
                signingConfigs.getByName("debug")
            }
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    externalNativeBuild {
        cmake { path = rootProject.file("native/voice/CMakeLists.txt"); version = "3.22.1" }
    }
    ndkVersion = "27.2.12479018"

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    buildFeatures {
        compose = true
    }

    sourceSets["main"].assets.directories.add(rootProject.layout.projectDirectory.dir("shared/src/commonMain/resources").asFile.path)
}

dependencies {
    implementation(project(":shared"))
    implementation(libs.koin.android)
    implementation(libs.firebase.messaging)
    implementation(libs.material)
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.appcompat)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.navigation3.runtime)
    implementation(libs.androidx.navigation3.ui)

    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.androidx.junit)
}
