package dev.luisamartins.owofinance.cards.impl.navigation

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import androidx.navigation.toRoute
import dev.luisamartins.owofinance.cards.api.navigation.destinations.AddCardDestination
import dev.luisamartins.owofinance.cards.api.navigation.destinations.CardBillDestination
import dev.luisamartins.owofinance.cards.impl.presentation.AddCardScreen
import dev.luisamartins.owofinance.cards.impl.presentation.AddCardViewModel
import dev.luisamartins.owofinance.cards.impl.presentation.CardBillScreen
import dev.luisamartins.owofinance.cards.impl.presentation.CardBillViewModel
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import org.koin.compose.viewmodel.koinViewModel
import org.koin.core.parameter.parametersOf

class CardsNavGraphContributor : NavGraphContributor {
    override fun NavGraphBuilder.contribute(navController: NavController) {
        composable<CardBillDestination> { backStackEntry ->
            val destination: CardBillDestination = backStackEntry.toRoute()
            val viewModel: CardBillViewModel = koinViewModel(
                parameters = { parametersOf(destination.cardId) },
            )
            CardBillScreen(
                viewModel = viewModel,
                onNavigateBack = { navController.popBackStack() },
            )
        }
        composable<AddCardDestination> {
            val viewModel: AddCardViewModel = koinViewModel()
            AddCardScreen(
                viewModel = viewModel,
                onNavigateBack = { navController.popBackStack() },
            )
        }
    }
}
