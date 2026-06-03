package dev.luisamartins.owofinance.feature.home.impl.navigation

import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.navigation.NavController
import androidx.navigation.NavDestination
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import dev.luisamartins.owofinance.accounts.api.navigation.destinations.AccountsDestination
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.dashboard.api.navigation.destinations.DashboardDestination
import dev.luisamartins.owofinance.feature.home.api.navigation.HomeDestination
import dev.luisamartins.owofinance.feature.home.api.navigation.HomeTabNavGraphContributor
import dev.luisamartins.owofinance.feature.home.impl.presentation.HomeScreen
import dev.luisamartins.owofinance.transactions.api.navigation.destinations.TransactionsDestination
import org.koin.compose.getKoin

class HomeNavGraphContributor: NavGraphContributor {
    override fun NavGraphBuilder.contribute(navController: NavController) {
        composable<HomeDestination> {
            val koin = getKoin()
            val contributors: List<NavGraphContributor> =
                remember { koin.getAll<HomeTabNavGraphContributor>() }

            val controller = rememberNavController()

            HomeScreen(
                navController = controller,
                onNavigateToDashboard = { controller.navigate(DashboardDestination) { launchSingleTop = true } },
                onNavigateToTransactions = { controller.navigate(TransactionsDestination) { launchSingleTop = true } },
                onNavigateToAccounts = { controller.navigate(AccountsDestination) { launchSingleTop = true } },
            ) { padding ->
                NavHost(
                    modifier = Modifier.padding(padding),
                    navController = controller,
                    startDestination = TransactionsDestination,
                    enterTransition = {
                        val from = initialState.destination.tabIndex()
                        val to = targetState.destination.tabIndex()
                        if (from >= 0 && to >= 0) slideInHorizontally { (if (to > from) 1 else -1) * it }
                        else slideInHorizontally()
                    },
                    exitTransition = {
                        val from = initialState.destination.tabIndex()
                        val to = targetState.destination.tabIndex()
                        if (from >= 0 && to >= 0) slideOutHorizontally { (if (to > from) -1 else 1) * it }
                        else slideOutHorizontally()
                    },
                    popEnterTransition = { slideInHorizontally() },
                    popExitTransition = { slideOutHorizontally() },

                ) {
                    contributors.forEach { with(it) { contribute(navController) } }
                }
            }
        }
    }
}

private fun NavDestination?.tabIndex(): Int = when {
    this?.hasRoute(DashboardDestination::class) == true -> 0
    this?.hasRoute(TransactionsDestination::class) == true -> 1
    this?.hasRoute(AccountsDestination::class) == true -> 2
    else -> -1
}