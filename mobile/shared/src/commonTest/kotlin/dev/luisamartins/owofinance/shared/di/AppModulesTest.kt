package dev.luisamartins.owofinance.shared.di

import dev.luisamartins.owofinance.core.navigation.api.NavGraphContributor
import dev.luisamartins.owofinance.core.navigation.api.Navigator
import org.koin.core.context.stopKoin
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

class AppModulesTest {

    @AfterTest
    fun tearDown() {
        stopKoin()
    }

    @Test
    fun initKoin_resolvesNavigator() {
        val app = initKoin()
        assertNotNull(app.koin.get<Navigator>())
    }

    @Test
    fun initKoin_registersAllFeatureContributors() {
        val app = initKoin()
        val contributors = app.koin.getAll<NavGraphContributor>()
        assertEquals(5, contributors.size)
    }
}
