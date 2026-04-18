package dev.luisamartins.owofinance.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.runtime.Composable

@Composable
actual fun determineColorScheme(
    darkTheme: Boolean,
    dynamicColor: Boolean
): ColorScheme {
    return LightColorScheme
}
