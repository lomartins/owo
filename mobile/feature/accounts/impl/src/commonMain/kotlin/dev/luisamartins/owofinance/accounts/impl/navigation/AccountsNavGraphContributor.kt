package dev.luisamartins.owofinance.accounts.impl.navigation

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import dev.luisamartins.owofinance.accounts.api.navigation.destinations.AccountsDestination
import dev.luisamartins.owofinance.accounts.api.navigation.destinations.AddAccountDestination
import dev.luisamartins.owofinance.accounts.impl.presentation.AccountsScreen
import dev.luisamartins.owofinance.accounts.impl.presentation.AccountsViewModel
import dev.luisamartins.owofinance.accounts.impl.presentation.AddAccountScreen
import dev.luisamartins.owofinance.accounts.impl.presentation.AddAccountViewModel
import dev.luisamartins.owofinance.cards.api.navigation.destinations.AddCardDestination
import dev.luisamartins.owofinance.cards.api.navigation.destinations.CardBillDestination
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.dashboard.api.navigation.destinations.DashboardDestination
import dev.luisamartins.owofinance.transactions.api.navigation.destinations.TransactionsDestination
import org.koin.compose.viewmodel.koinViewModel

class AccountsNavGraphContributor : NavGraphContributor {
    override fun NavGraphBuilder.contribute(navController: NavController) {
        composable<AccountsDestination> {
            val viewModel: AccountsViewModel = koinViewModel()
            AccountsScreen(
                viewModel = viewModel,
                onNavigateToDashboard = {
                    navController.navigate(DashboardDestination) { launchSingleTop = true }
                },
                onNavigateToTransactions = {
                    navController.navigate(TransactionsDestination) { launchSingleTop = true }
                },
                onAddAccount = { navController.navigate(AddAccountDestination) },
                onAddCard = { navController.navigate(AddCardDestination) },
                onNavigateToCardBill = { cardId -> navController.navigate(CardBillDestination(cardId)) },
            )
        }
        composable<AddAccountDestination> {
            val viewModel: AddAccountViewModel = koinViewModel()
            AddAccountScreen(
                viewModel = viewModel,
                onNavigateBack = { navController.popBackStack() },
            )
        }
    }
}
