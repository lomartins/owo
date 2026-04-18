pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }
}
plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "owofinance"
include(":app")
include(":core:domain")
include(":core:database")
include(":core:ui")
include(":core:navigation:api")
include(":core:navigation:impl")
include(":feature:onboarding:api")
include(":feature:onboarding:impl")
include(":feature:dashboard:api")
include(":feature:dashboard:impl")
include(":feature:accounts:api")
include(":feature:accounts:impl")
include(":feature:cards:api")
include(":feature:cards:impl")
include(":feature:transactions:api")
include(":feature:transactions:impl")
include(":shared")
