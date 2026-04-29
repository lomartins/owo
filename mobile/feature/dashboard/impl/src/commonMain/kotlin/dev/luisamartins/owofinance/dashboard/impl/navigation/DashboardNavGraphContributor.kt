package dev.luisamartins.owofinance.dashboard.impl.navigation

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.dashboard.api.navigation.destinations.DashboardDestination
import dev.luisamartins.owofinance.dashboard.impl.presentation.DashboardScreen

class DashboardNavGraphContributor : NavGraphContributor {
    override fun NavGraphBuilder.contribute(navController: NavController) {
        composable<DashboardDestination> {
            DashboardScreen()
        }
    }
}
