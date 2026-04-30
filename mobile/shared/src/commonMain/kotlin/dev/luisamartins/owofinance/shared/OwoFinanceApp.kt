package dev.luisamartins.owofinance.shared

import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.navigation.NavController
import androidx.navigation.NavDestination
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import dev.luisamartins.owofinance.accounts.api.navigation.destinations.AccountsDestination
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.core.navigation.api.NavigateBack
import dev.luisamartins.owofinance.core.navigation.api.Navigator
import dev.luisamartins.owofinance.dashboard.api.navigation.destinations.DashboardDestination
import dev.luisamartins.owofinance.onboarding.api.navigation.destinations.WelcomeDestination
import dev.luisamartins.owofinance.transactions.api.navigation.destinations.TransactionsDestination
import dev.luisamartins.owofinance.ui.components.BottomNavTab
import dev.luisamartins.owofinance.ui.components.OwoBottomBar
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
        Scaffold(
            bottomBar = { BottomBar(navController) },
        ) { innerPadding ->
            NavHost(
                navController,
                startDestination = WelcomeDestination,
                modifier = Modifier.padding(innerPadding),
                enterTransition = {
                    val from = initialState.destination.tabIndex()
                    val to = targetState.destination.tabIndex()
                    if (from >= 0 && to >= 0) slideInHorizontally { (if (to > from) 1 else -1) * it }
                    else fadeIn()
                },
                exitTransition = {
                    val from = initialState.destination.tabIndex()
                    val to = targetState.destination.tabIndex()
                    if (from >= 0 && to >= 0) slideOutHorizontally { (if (to > from) -1 else 1) * it }
                    else fadeOut()
                },
                popEnterTransition = { fadeIn() },
                popExitTransition = { fadeOut() },
            ) {
                contributors.forEach { with(it) { contribute(navController) } }
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


@Composable
fun BottomBar(navController: NavController) {
    val currentBackStackEntry by navController.currentBackStackEntryAsState()

    val currentTab = when (currentBackStackEntry?.destination.tabIndex()) {
        0 -> BottomNavTab.DASHBOARD
        1 -> BottomNavTab.TRANSACTIONS
        2 -> BottomNavTab.ACCOUNTS
        else -> null
    }

    if (currentTab != null) {
        OwoBottomBar(
            selectedTab = currentTab,
            onTabSelected = { tab ->
                if (currentTab == tab) return@OwoBottomBar
                when (tab) {
                    BottomNavTab.DASHBOARD -> navController.navigate(DashboardDestination) {
                        launchSingleTop = true
                    }
                    BottomNavTab.TRANSACTIONS -> navController.navigate(TransactionsDestination) {
                        launchSingleTop = true
                    }
                    BottomNavTab.ACCOUNTS -> navController.navigate(AccountsDestination) {
                        launchSingleTop = true
                    }
                }
            },
        )
    }
}