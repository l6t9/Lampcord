plugins {
   alias(libs.plugins.kotlin.multiplatform)
   alias(libs.plugins.compose.multiplatform)
   alias(libs.plugins.compose.compiler)
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
           }
       }
   }
}

compose.desktop {
   application {
       mainClass = "com.example.materialcord.MainKt"
       nativeDistributions {
           targetFormats(org.jetbrains.compose.desktop.application.dsl.TargetFormat.Dmg, org.jetbrains.compose.desktop.application.dsl.TargetFormat.Msi, org.jetbrains.compose.desktop.application.dsl.TargetFormat.Deb)
           packageName = "Materialcord"
           packageVersion = "1.0.0"
       }
   }
}
