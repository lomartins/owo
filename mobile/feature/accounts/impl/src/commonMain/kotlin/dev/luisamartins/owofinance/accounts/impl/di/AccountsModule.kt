package dev.luisamartins.owofinance.accounts.impl.di

import dev.luisamartins.owofinance.accounts.impl.navigation.AccountsNavGraphContributor
import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val accountsModule = module {
    factory(named<AccountsNavGraphContributor>()) {
        AccountsNavGraphContributor()
    } bind NavGraphContributor::class
}
