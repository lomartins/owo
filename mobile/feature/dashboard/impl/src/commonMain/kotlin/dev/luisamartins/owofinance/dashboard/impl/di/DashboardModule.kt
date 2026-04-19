package dev.luisamartins.owofinance.dashboard.impl.di

import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.dashboard.impl.navigation.DashboardNavGraphContributor
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val dashboardModule = module {
    factory(named<DashboardNavGraphContributor>()) {
        DashboardNavGraphContributor()
    } bind NavGraphContributor::class
}