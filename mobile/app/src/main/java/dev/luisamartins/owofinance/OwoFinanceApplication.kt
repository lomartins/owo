package dev.luisamartins.owofinance

import android.app.Application
import dev.luisamartins.owofinance.shared.di.initKoin
import org.koin.android.ext.koin.androidContext

class OwoFinanceApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        initKoin { androidContext(this@OwoFinanceApplication) }
    }
}
