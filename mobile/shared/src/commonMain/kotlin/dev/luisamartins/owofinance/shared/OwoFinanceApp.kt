package dev.luisamartins.owofinance.shared

import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.rememberNavController
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.core.navigation.api.NavigateBack
import dev.luisamartins.owofinance.core.navigation.api.Navigator
import dev.luisamartins.owofinance.onboarding.api.navigation.destinations.WelcomeDestination
import dev.luisamartins.owofinance.ui.theme.OwoFinanceTheme
import org.koin.compose.getKoin
import org.koin.compose.koinInject

@Composable
fun OwoFinanceApp() {
    val navController = rememberNavController()
    val navigator: Navigator = koinInject()
    val koin = getKoin()
    val contributors: List<NavGraphContributor> =
        remember { koin.getAll<NavGraphContributor>() }

    LaunchedEffect(Unit) {
        navigator.navigationEvents.collect { destination ->
            when (destination) {
                is NavigateBack -> navController.popBackStack()
                else -> navController.navigate(destination)
            }
        }
    }

    OwoFinanceTheme {
        NavHost(
            navController,
            startDestination = WelcomeDestination,
            enterTransition = { slideInHorizontally { it } + fadeIn() },
            exitTransition = { slideOutHorizontally { -it } + fadeOut() },
        ) {
            contributors.forEach { with(it) { contribute(navController) } }
        }
    }
}
