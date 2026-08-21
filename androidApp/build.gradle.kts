import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
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

    defaultConfig {
        applicationId = "me.lampu.lampcord"
        minSdk = 24
        targetSdk = 37
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    signingConfigs {
        create("release") {
            val storeFileProperty = keystoreProperties["release.storeFile"] as String?
            if (storeFileProperty != null) {
                storeFile = rootProject.file(storeFileProperty)
                storePassword = keystoreProperties["release.storePassword"] as String?
                keyAlias = keystoreProperties["release.keyAlias"] as String?
                keyPassword = keystoreProperties["release.keyPassword"] as String?
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            signingConfig = signingConfigs.getByName("release")
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
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
