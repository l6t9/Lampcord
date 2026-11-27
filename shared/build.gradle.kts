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
    packageOfResClass = "me.lampu.lampcord.shared.generated.resources"
}

val unpackTwemoji = tasks.register<Sync>("unpackTwemoji") {
    from(zipTree("src/commonMain/twemoji.zip"))
    into(layout.buildDirectory.dir("generated/twemojiResources"))
}

val generatedUpdateDir = layout.buildDirectory.dir("generated/update")

val generateAppVersion = tasks.register("generateAppVersion") {
    val appVersion = providers.gradleProperty("appVersion").get()
    val nightlyBuild = providers.gradleProperty("nightlyBuild").orNull?.toIntOrNull() ?: 0
    val version = if (nightlyBuild > 0) "$appVersion-nightly.$nightlyBuild" else appVersion
    val output = generatedUpdateDir.map { it.file("me/lampu/lampcord/shared/update/AppVersion.kt") }
    inputs.property("version", version)
    inputs.property("nightlyBuild", nightlyBuild.toString())
    outputs.file(output)
    doLast {
        val file = output.get().asFile
        file.parentFile.mkdirs()
        file.writeText(
            """
            package me.lampu.lampcord.shared.update

            const val APP_VERSION: String = "$version"

            const val NIGHTLY_BUILD: Int = $nightlyBuild
            """.trimIndent() + "\n"
        )
    }
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

        androidResources {
            enable = true
        }
    }

    listOf(
        iosArm64(),
        iosSimulatorArm64()
    ).forEach {
        it.compilations.all {
            compilerOptions.configure {
                freeCompilerArgs.add(
                    "-Xdisable-phases=DevirtualizationAnalysis,DCEPhase,RemoveRedundantCallsToStaticInitializersPhase"
                )
            }
        }
        it.binaries.framework {
            baseName = "Shared"

            isStatic = false
            binaryOption("bundleId", "me.lampcord.shared")
        }
    }

    applyDefaultHierarchyTemplate()

    listOf("iosArm64", "iosSimulatorArm64").forEach { target ->
        configurations.matching { it.name == "${target}Implementation" }.configureEach {
            exclude(group = "org.jetbrains.skiko", module = "skiko")
        }
    }

    sourceSets {
        val jvmSharedMain by creating {
            dependsOn(commonMain.get())
            dependencies {
                implementation(libs.slf4j.simple)
                implementation(libs.ktor.client.cio)
            }
        }
        getByName("androidMain").dependsOn(jvmSharedMain)

        commonMain {
            resources.srcDir(unpackTwemoji)
            kotlin.srcDir(files(generatedUpdateDir).builtBy(generateAppVersion))
        }

        commonMain.dependencies {
            implementation(libs.compose.runtime)
            implementation(libs.compose.foundation)
            implementation(libs.compose.material3)
            implementation(libs.compose.material3.adaptive)
            implementation(libs.compose.material3.adaptive.navigation.suite)
            implementation(libs.compose.ui)
            implementation(libs.compose.components.resources)

            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.kotlinx.serialization.json)
            implementation(libs.kotlinx.datetime)

            implementation(libs.ktor.client.core)
            implementation(libs.ktor.client.content.negotiation)
            implementation(libs.ktor.serialization.kotlinx.json)
            implementation(libs.ktor.client.websockets)
            implementation(libs.ktor.client.logging)

            api(libs.koin.core)
            implementation(libs.koin.compose)

            implementation(libs.multiplatform.settings)
            implementation(libs.multiplatform.settings.no.arg)

            api(libs.androidx.navigation3.runtime)
            api(libs.androidx.navigationevent.compose)

            implementation(libs.room.runtime)
            implementation(libs.sqlite.bundled)

            implementation(libs.materialKolor)
            implementation(libs.qrcode.kotlin)

            implementation(libs.coil.compose)
            implementation(libs.coil.network.ktor3)
            implementation(libs.coil.svg)
            implementation(libs.composeWebview)
        }

        getByName("iosMain") {
            dependencies {
                implementation(libs.ktor.client.darwin)
                implementation(libs.sqlite.bundled)
                implementation(libs.coil.network.ktor3)
            }
        }

        getByName("androidMain") {
            dependencies {
                implementation(libs.androidx.appcompat)
                implementation(libs.androidx.activity.compose)
                implementation(libs.androidx.media3.exoplayer)
                implementation(libs.androidx.media3.ui)
                implementation(libs.androidx.security.crypto)
                implementation(libs.coil.gif)

                implementation(libs.androidx.navigation3.ui)

                implementation("net.java.dev.jna:jna:${libs.versions.jna.get()}@aar")
            }
        }

    }
}

dependencies {
    add("kspAndroid", libs.room.compiler)
    add("kspIosArm64", libs.room.compiler)
    add("kspIosSimulatorArm64", libs.room.compiler)
}

ksp {
    arg("room.schemaLocation", "$projectDir/schemas")
}
