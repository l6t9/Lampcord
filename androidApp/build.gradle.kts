import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.google.services)
    alias(libs.plugins.kotlin.multiplatform) apply false
    alias(libs.plugins.compose.multiplatform)
    alias(libs.plugins.compose.compiler)
}

val rustCoreEnabled = providers.gradleProperty("rustCore").orElse("true").map { it.toBoolean() }
val rustFfiJniLibsDir = rootProject.projectDir.resolve("androidApp/src/main/jniLibs")

val buildRustFfiAndroid by tasks.registering(Exec::class) {
    group = "rust"
    description = "Build lampcord-ffi for Android ABIs with cargo-ndk into src/main/jniLibs"
    onlyIf { rustCoreEnabled.get() }
    workingDir = rootProject.projectDir.resolve("rust")
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a", "-t", "armeabi-v7a", "-t", "x86_64", "-t", "x86",
        "-o", rustFfiJniLibsDir.absolutePath,
        "build", "-p", "lampcord-ffi", "--features", "uniffi", "--release",
    )
}

afterEvaluate {
    tasks.matching { it.name.startsWith("merge") && it.name.endsWith("JniLibFolders") }.configureEach {
        dependsOn(buildRustFfiAndroid)
    }
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
        minSdk = 24
        targetSdk = 37

        val appVersion = project.property("appVersion") as String
        val nightlyBuild = (project.findProperty("nightlyBuild") as String?)?.toIntOrNull() ?: 0

        applicationId = if (nightlyBuild > 0) "me.lampu.lampcord.nightly" else "me.lampu.lampcord"
        versionName = if (nightlyBuild > 0) "$appVersion-nightly.$nightlyBuild" else appVersion

        val (versionMajor, versionMinor, versionPatch) =
            appVersion.substringBefore('-').split('.').map { it.toIntOrNull() ?: 0 }
        versionCode = if (nightlyBuild > 0) {
            1_000_000 + (nightlyBuild % 1_000_000)
        } else {
            versionMajor * 1_000_000 + versionMinor * 1_000 + versionPatch
        }

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        buildConfigField("boolean", "RUST_CORE", rustCoreEnabled.get().toString())

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
        buildConfig = true
    }

    sourceSets["main"].assets.directories.add(rootProject.layout.projectDirectory.dir("shared/src/commonMain/resources").asFile.path)
}

dependencies {
    implementation(project(":shared"))
    implementation(libs.kotlinx.serialization.json)
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
