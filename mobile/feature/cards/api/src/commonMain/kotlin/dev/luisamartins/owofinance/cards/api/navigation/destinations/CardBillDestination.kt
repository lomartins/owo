package dev.luisamartins.owofinance.cards.api.navigation.destinations

import dev.luisamartins.owofinance.core.navigation.api.NavigationDestination
import kotlinx.serialization.Serializable

@Serializable
data class CardBillDestination(val cardId: String) : NavigationDestination
