package dev.luisamartins.owofinance.core.navigation.impl

import dev.luisamartins.owofinance.core.navigation.api.NavigateBack
import dev.luisamartins.owofinance.core.navigation.api.NavigationDestination
import dev.luisamartins.owofinance.core.navigation.api.Navigator
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow

class NavigatorImpl : Navigator {
    private val channel = Channel<NavigationDestination>(
        capacity = Channel.BUFFERED,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )
    override val navigationEvents: Flow<NavigationDestination> = channel.receiveAsFlow()

    override fun navigateTo(route: NavigationDestination) {
        channel.trySend(route)
    }

    override fun navigateBack() {
        channel.trySend(NavigateBack)
    }
}
