plugins {
    alias(libs.plugins.kotlin.multiplatform)
    alias(libs.plugins.android.kotlin.multiplatform.library)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.jetbrains.compose)
}

kotlin {
    android {
        namespace = "dev.luisamartins.owofinance.core.navigation.impl"
        compileSdk = 36
        minSdk = 24
    }

    iosX64()
    iosArm64()
    iosSimulatorArm64()

    sourceSets {
        commonMain.dependencies {
            api(project(":core:navigation:api"))
            implementation(libs.navigation.compose)
            implementation(libs.koin.core)
            implementation(libs.bundles.compose.multiplatform)
        }
    }
}
