package dev.luisamartins.owofinance.accounts.impl.di

import dev.luisamartins.owofinance.accounts.impl.navigation.AccountsNavGraphContributor
import dev.luisamartins.owofinance.accounts.impl.presentation.AccountsViewModel
import dev.luisamartins.owofinance.accounts.impl.presentation.AddAccountViewModel
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import org.koin.core.module.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val accountsModule = module {
    viewModel { AccountsViewModel() }
    viewModel { AddAccountViewModel() }

    factory(named<AccountsNavGraphContributor>()) {
        AccountsNavGraphContributor()
    } bind NavGraphContributor::class
}
