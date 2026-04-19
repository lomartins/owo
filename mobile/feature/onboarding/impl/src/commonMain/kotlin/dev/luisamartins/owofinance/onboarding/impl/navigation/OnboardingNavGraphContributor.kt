package dev.luisamartins.owofinance.onboarding.impl.navigation

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.onboarding.api.navigation.destinations.WelcomeDestination
import dev.luisamartins.owofinance.onboarding.impl.presentation.WelcomeScreen
import dev.luisamartins.owofinance.onboarding.impl.presentation.WelcomeViewModel
import org.koin.compose.viewmodel.koinViewModel

class OnboardingNavGraphContributor : NavGraphContributor {
    override fun NavGraphBuilder.contribute(navController: NavController) {
        composable<WelcomeDestination> {
            @Suppress("UNUSED_VARIABLE")
            val viewModel: WelcomeViewModel = koinViewModel()
            WelcomeScreen(
                onGetStarted = {},
            )
        }
    }
}
