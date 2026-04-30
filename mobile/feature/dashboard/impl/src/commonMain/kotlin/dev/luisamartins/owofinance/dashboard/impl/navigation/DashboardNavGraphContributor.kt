package dev.luisamartins.owofinance.dashboard.impl.navigation

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import dev.luisamartins.owofinance.accounts.api.navigation.destinations.AccountsDestination
import dev.luisamartins.owofinance.cards.api.navigation.destinations.CardBillDestination
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.dashboard.api.navigation.destinations.DashboardDestination
import dev.luisamartins.owofinance.dashboard.impl.presentation.DashboardScreen
import dev.luisamartins.owofinance.dashboard.impl.presentation.DashboardViewModel
import dev.luisamartins.owofinance.transactions.api.navigation.destinations.TransactionsDestination
import org.koin.compose.viewmodel.koinViewModel

class DashboardNavGraphContributor : NavGraphContributor {
    override fun NavGraphBuilder.contribute(navController: NavController) {
        composable<DashboardDestination> {
            val viewModel: DashboardViewModel = koinViewModel()
            DashboardScreen(
                viewModel = viewModel,
                onNavigateToTransactions = {
                    navController.navigate(TransactionsDestination) { launchSingleTop = true }
                },
                onNavigateToAccounts = {
                    navController.navigate(AccountsDestination) { launchSingleTop = true }
                },
                onNavigateToAllTransactions = {
                    navController.navigate(TransactionsDestination) { launchSingleTop = true }
                },
                onNavigateToCardBill = { cardId ->
                    navController.navigate(CardBillDestination(cardId))
                },
            )
        }
    }
}
