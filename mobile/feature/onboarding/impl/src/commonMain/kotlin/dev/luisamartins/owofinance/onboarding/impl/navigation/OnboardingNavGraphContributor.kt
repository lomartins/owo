package dev.luisamartins.owofinance.onboarding.impl.navigation

import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.onboarding.api.navigation.destinations.OnboardingDestination
import dev.luisamartins.owofinance.onboarding.api.navigation.destinations.WelcomeDestination
import dev.luisamartins.owofinance.onboarding.impl.presentation.OnboardingScreen
import dev.luisamartins.owofinance.onboarding.impl.presentation.WelcomeScreen
import dev.luisamartins.owofinance.onboarding.impl.presentation.WelcomeViewModel
import org.koin.compose.viewmodel.koinViewModel

class OnboardingNavGraphContributor : NavGraphContributor {
    override fun NavGraphBuilder.contribute(navController: NavController) {
        composable<WelcomeDestination> {
            @Suppress("UNUSED_VARIABLE")
            val viewModel: WelcomeViewModel = koinViewModel()
            WelcomeScreen(
                onGetStarted = { navController.navigate(OnboardingDestination) },
            )
        }
        composable<OnboardingDestination>(
            enterTransition = { slideInHorizontally { it } + fadeIn() },
            exitTransition = { slideOutHorizontally { -it } + fadeOut() },
        ) {
            OnboardingScreen(
                onNavigateBack = { navController.popBackStack() },
                onFinish = { /* TODO Navigate to Dashboard */ },
            )
        }
    }
}
