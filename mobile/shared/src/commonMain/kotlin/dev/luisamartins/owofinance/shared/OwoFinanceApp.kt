package dev.luisamartins.owofinance.shared

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import dev.luisamartins.owofinance.core.navigation.api.NavigateBack
import dev.luisamartins.owofinance.core.navigation.api.NavigationDestination
import dev.luisamartins.owofinance.core.navigation.api.Navigator
import dev.luisamartins.owofinance.core.navigation.impl.NavGraphContributor
import dev.luisamartins.owofinance.core.navigation.impl.NavigatorImpl
import dev.luisamartins.owofinance.ui.theme.OwoFinanceTheme
import kotlinx.serialization.Serializable

@Composable
fun OwoFinanceApp() {
    val navController = rememberNavController()
    val navigator: Navigator = remember { NavigatorImpl() } // TODO implement DI
    val contributors: List<NavGraphContributor> = remember { listOf(
        NavGraphContributor {
            composable<FakeDestination> {
                Greeting("owó")
            }
        }
    ) }

    LaunchedEffect(Unit) {
        navigator.navigationEvents.collect { destination ->
            when (destination) {
                is NavigateBack -> navController.popBackStack()
                else -> navController.navigate(destination)   // type-safe overload, event is @Serializable
            }
        }
    }

    OwoFinanceTheme {
        Scaffold(bottomBar = {  }) { padding ->
            NavHost(navController, startDestination = FakeDestination, Modifier.padding(padding)) {
                contributors.forEach { with(it) { contribute(navController) } }
            }
        }
    }
}

@Serializable
private object FakeDestination: NavigationDestination

@Composable
fun Greeting(name: String, modifier: Modifier = Modifier) {
    Text(
        text = "Hello $name!",
        modifier = modifier
    )
}