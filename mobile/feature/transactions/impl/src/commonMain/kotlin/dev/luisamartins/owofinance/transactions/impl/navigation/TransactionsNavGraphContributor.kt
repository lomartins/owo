package dev.luisamartins.owofinance.transactions.impl.navigation

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import dev.luisamartins.owofinance.accounts.api.navigation.destinations.AccountsDestination
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.dashboard.api.navigation.destinations.DashboardDestination
import dev.luisamartins.owofinance.transactions.api.navigation.destinations.AddTransactionDestination
import dev.luisamartins.owofinance.transactions.api.navigation.destinations.TransactionsDestination
import dev.luisamartins.owofinance.transactions.impl.presentation.AddTransactionScreen
import dev.luisamartins.owofinance.transactions.impl.presentation.AddTransactionViewModel
import dev.luisamartins.owofinance.transactions.impl.presentation.TransactionsScreen
import dev.luisamartins.owofinance.transactions.impl.presentation.TransactionsViewModel
import org.koin.compose.viewmodel.koinViewModel

class TransactionsNavGraphContributor : NavGraphContributor {
    override fun NavGraphBuilder.contribute(navController: NavController) {
        composable<TransactionsDestination> {
            val viewModel: TransactionsViewModel = koinViewModel()
            TransactionsScreen(
                viewModel = viewModel,
                onNavigateToDashboard = {
                    navController.navigate(DashboardDestination) { launchSingleTop = true }
                },
                onNavigateToAccounts = {
                    navController.navigate(AccountsDestination) { launchSingleTop = true }
                },
                onAddTransaction = { navController.navigate(AddTransactionDestination) },
            )
        }
        composable<AddTransactionDestination> {
            val viewModel: AddTransactionViewModel = koinViewModel()
            AddTransactionScreen(
                viewModel = viewModel,
                onNavigateBack = { navController.popBackStack() },
            )
        }
    }
}
