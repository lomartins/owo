package dev.luisamartins.owofinance.dashboard.impl.di

import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.dashboard.impl.navigation.DashboardNavGraphContributor
import dev.luisamartins.owofinance.dashboard.impl.presentation.DashboardViewModel
import org.koin.core.module.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val dashboardModule = module {
    viewModel { DashboardViewModel() }

    factory(named<DashboardNavGraphContributor>()) {
        DashboardNavGraphContributor()
    } bind NavGraphContributor::class
}
