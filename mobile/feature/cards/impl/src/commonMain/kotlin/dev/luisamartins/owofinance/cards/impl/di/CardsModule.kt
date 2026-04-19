package dev.luisamartins.owofinance.cards.impl.di

import dev.luisamartins.owofinance.cards.impl.navigation.CardsNavGraphContributor
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val cardsModule = module {
    factory(named<CardsNavGraphContributor>()) {
        CardsNavGraphContributor()
    } bind NavGraphContributor::class
}
