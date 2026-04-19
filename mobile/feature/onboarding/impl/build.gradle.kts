plugins {
    alias(libs.plugins.kotlin.multiplatform)
    alias(libs.plugins.android.kotlin.multiplatform.library)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.jetbrains.compose)
}

kotlin {
    android {
        namespace = "dev.luisamartins.owofinance.feature.onboarding.impl"
        compileSdk = 36
        minSdk = 24
    }

    iosX64()
    iosArm64()
    iosSimulatorArm64()

    sourceSets {
        commonMain.dependencies {
            api(project(":feature:onboarding:api"))
            implementation(project(":core:domain"))
            implementation(project(":core:ui"))
            implementation(project(":core:navigation:api"))
            implementation(libs.koin.core)
            implementation(libs.koin.compose)
            implementation(libs.koin.compose.viewmodel)
            implementation(libs.androidx.lifecycle.viewmodel)
            implementation(libs.bundles.compose.multiplatform)
            implementation(libs.navigation.compose)
        }
        androidMain.dependencies {
            implementation(libs.koin.android)
        }
    }
}
