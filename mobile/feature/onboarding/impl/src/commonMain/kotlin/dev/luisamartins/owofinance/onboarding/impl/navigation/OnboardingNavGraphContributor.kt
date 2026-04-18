package dev.luisamartins.owofinance.onboarding.impl.navigation

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.onboarding.api.navigation.destinations.WelcomeDestination
import dev.luisamartins.owofinance.onboarding.impl.ui.WelcomeScreen

class OnboardingNavGraphContributor: NavGraphContributor {
    override fun NavGraphBuilder.contribute(navController: NavController) {
        composable<WelcomeDestination> {
            WelcomeScreen(
                onGetStarted = {}
            )
        }
    }
}