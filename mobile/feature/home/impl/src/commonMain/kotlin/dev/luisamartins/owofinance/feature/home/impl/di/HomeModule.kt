package dev.luisamartins.owofinance.feature.home.impl.di

import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.feature.home.impl.navigation.HomeNavGraphContributor
import org.koin.dsl.bind
import org.koin.dsl.module

val homeModule = module {
    factory { HomeNavGraphContributor() } bind NavGraphContributor::class
}