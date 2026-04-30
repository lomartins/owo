package dev.luisamartins.owofinance.cards.impl.di

import dev.luisamartins.owofinance.cards.impl.navigation.CardsNavGraphContributor
import dev.luisamartins.owofinance.cards.impl.presentation.AddCardViewModel
import dev.luisamartins.owofinance.cards.impl.presentation.CardBillViewModel
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import org.koin.core.module.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val cardsModule = module {
    viewModel { (cardId: String) -> CardBillViewModel(cardId) }
    viewModel { AddCardViewModel() }

    factory(named<CardsNavGraphContributor>()) {
        CardsNavGraphContributor()
    } bind NavGraphContributor::class
}
