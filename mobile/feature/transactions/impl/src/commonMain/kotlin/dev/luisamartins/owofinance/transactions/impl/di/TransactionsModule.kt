package dev.luisamartins.owofinance.transactions.impl.di

import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.transactions.impl.navigation.TransactionsNavGraphContributor
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val transactionsModule = module {
    factory(named<TransactionsNavGraphContributor>()) {
        TransactionsNavGraphContributor()
    } bind NavGraphContributor::class
}
