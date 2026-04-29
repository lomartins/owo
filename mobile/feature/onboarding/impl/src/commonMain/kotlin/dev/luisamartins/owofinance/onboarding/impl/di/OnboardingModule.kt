package dev.luisamartins.owofinance.onboarding.impl.di

import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.onboarding.impl.navigation.OnboardingNavGraphContributor
import dev.luisamartins.owofinance.onboarding.impl.presentation.OnboardingViewModel
import dev.luisamartins.owofinance.onboarding.impl.presentation.WelcomeViewModel
import org.koin.core.module.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val onboardingModule = module {
    viewModel { WelcomeViewModel() }
    viewModel { OnboardingViewModel() }

    factory(named<OnboardingNavGraphContributor>()) {
        OnboardingNavGraphContributor()
    } bind NavGraphContributor::class
}
