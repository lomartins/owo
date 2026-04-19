package dev.luisamartins.owofinance.shared.di

import dev.luisamartins.owofinance.accounts.impl.di.accountsModule
import dev.luisamartins.owofinance.cards.impl.di.cardsModule
import dev.luisamartins.owofinance.core.database.di.databaseModule
import dev.luisamartins.owofinance.core.navigation.impl.di.navigationModule
import dev.luisamartins.owofinance.dashboard.impl.di.dashboardModule
import dev.luisamartins.owofinance.domain.di.coreDomainModule
import dev.luisamartins.owofinance.onboarding.impl.di.onboardingModule
import dev.luisamartins.owofinance.transactions.impl.di.transactionsModule
import org.koin.core.KoinApplication
import org.koin.core.context.startKoin
import org.koin.dsl.KoinAppDeclaration

val appModules = listOf(
    navigationModule,
    coreDomainModule,
    databaseModule,
    onboardingModule,
    dashboardModule,
    accountsModule,
    cardsModule,
    transactionsModule,
)

fun initKoin(declaration: KoinAppDeclaration? = null): KoinApplication = startKoin {
    allowOverride(false)
    declaration?.invoke(this)
    modules(appModules)
}
