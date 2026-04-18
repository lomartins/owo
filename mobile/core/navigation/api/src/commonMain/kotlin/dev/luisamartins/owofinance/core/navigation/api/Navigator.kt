package dev.luisamartins.owofinance.core.navigation.api

import kotlinx.coroutines.flow.Flow

interface Navigator {
    val navigationEvents: Flow<NavigationDestination>
    fun navigateTo(route: NavigationDestination)
    fun navigateBack()
}
