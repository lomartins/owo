package dev.luisamartins.owofinance.transactions.impl.di

import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.transactions.impl.navigation.TransactionsNavGraphContributor
import dev.luisamartins.owofinance.transactions.impl.presentation.AddTransactionViewModel
import dev.luisamartins.owofinance.transactions.impl.presentation.TransactionsViewModel
import org.koin.core.module.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val transactionsModule = module {
    viewModel { TransactionsViewModel() }
    viewModel { AddTransactionViewModel() }

    factory(named<TransactionsNavGraphContributor>()) {
        TransactionsNavGraphContributor()
    } bind NavGraphContributor::class
}
