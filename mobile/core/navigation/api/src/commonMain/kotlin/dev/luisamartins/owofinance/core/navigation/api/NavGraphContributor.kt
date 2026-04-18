package dev.luisamartins.owofinance.core.navigation.api

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder

fun interface NavGraphContributor {
    fun NavGraphBuilder.contribute(navController: NavController)
}