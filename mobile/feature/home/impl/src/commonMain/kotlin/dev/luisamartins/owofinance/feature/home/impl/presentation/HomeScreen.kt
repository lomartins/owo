package dev.luisamartins.owofinance.feature.home.impl.presentation

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.navigation.NavController
import androidx.navigation.NavDestination
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import dev.luisamartins.owofinance.accounts.api.navigation.destinations.AccountsDestination
import dev.luisamartins.owofinance.dashboard.api.navigation.destinations.DashboardDestination
import dev.luisamartins.owofinance.transactions.api.navigation.destinations.TransactionsDestination
import dev.luisamartins.owofinance.ui.components.BottomNavTab
import dev.luisamartins.owofinance.ui.components.OwoBottomBar

@Composable
fun HomeScreen(
    navController: NavController,
    onNavigateToDashboard: () -> Unit,
    onNavigateToTransactions: () -> Unit,
    onNavigateToAccounts: () -> Unit,
    content: @Composable (PaddingValues) -> Unit,
) {
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination
    val tabIndex = currentDestination.tabIndex()

    HomeScreenContent(
        tabIndex = tabIndex,
        navigateToDashboard = onNavigateToDashboard,
        navigateToTransactions = onNavigateToTransactions,
        navigateToAccounts = onNavigateToAccounts,
        content = content
    )
}

@Composable
fun HomeScreenContent(
    tabIndex: Int,
    navigateToDashboard: () -> Unit,
    navigateToTransactions: () -> Unit,
    navigateToAccounts: () -> Unit,
    content: @Composable (PaddingValues) -> Unit,
) {
    Scaffold(
        bottomBar = {
            BottomBar(
                tabIndex,
                navigateToDashboard,
                navigateToTransactions,
                navigateToAccounts
            )
        },
        content = content
    )
}


private fun NavDestination?.tabIndex(): Int = when {
    this?.hasRoute(DashboardDestination::class) == true -> 0
    this?.hasRoute(TransactionsDestination::class) == true -> 1
    this?.hasRoute(AccountsDestination::class) == true -> 2
    else -> -1
}

@Composable
fun BottomBar(
    tabIndex: Int,
    navigateToDashboard: () -> Unit,
    navigateToTransactions: () -> Unit,
    navigateToAccounts: () -> Unit,
) {

    val currentTab = when (tabIndex) {
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
                    BottomNavTab.DASHBOARD -> navigateToDashboard()
                    BottomNavTab.TRANSACTIONS -> navigateToTransactions()
                    BottomNavTab.ACCOUNTS -> navigateToAccounts()
                }
            },
        )
    }
}