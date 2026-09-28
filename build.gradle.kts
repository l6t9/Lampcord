// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.google.services) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.android.kmp.library) apply false
    alias(libs.plugins.kotlin.multiplatform) apply false
    alias(libs.plugins.compose.multiplatform) apply false
    alias(libs.plugins.compose.compiler) apply false
    alias(libs.plugins.kotlin.serialization) apply false
}

subprojects {
    configurations.configureEach {
        resolutionStrategy.eachDependency {
            // sketch-http-ktor3 pins ktor-client-java to 3.4.3, which would sit next to the 3.6.0 core.
            if (requested.group == "io.ktor") {
                useVersion(libs.versions.ktor.get())
                because("all ktor modules must resolve to a single version")
            }
            // ktor-server-core requests kotlin-reflect 2.3.21; an older reflect against a newer stdlib fails.
            if (requested.group == "org.jetbrains.kotlin" && requested.name == "kotlin-reflect") {
                useVersion(libs.versions.kotlin.get())
                because("kotlin-reflect must match kotlin-stdlib")
            }
        }
    }
}