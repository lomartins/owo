package dev.luisamartins.owofinance.core.navigation.impl.di

import dev.luisamartins.owofinance.core.navigation.api.Navigator
import dev.luisamartins.owofinance.core.navigation.impl.NavigatorImpl
import org.koin.dsl.module

val navigationModule = module {
    single<Navigator> { NavigatorImpl() }
}
