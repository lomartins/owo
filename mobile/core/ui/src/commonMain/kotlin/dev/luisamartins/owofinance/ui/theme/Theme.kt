package dev.luisamartins.owofinance.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable

internal val LightColorScheme = lightColorScheme(
    primary           = Primary600,
    onPrimary         = androidx.compose.ui.graphics.Color.White,
    primaryContainer  = Primary50,
    onPrimaryContainer = Primary800,

    background        = Neutral50,
    onBackground      = Neutral900,
    surface           = androidx.compose.ui.graphics.Color.White,
    onSurface         = Neutral900,
    surfaceVariant    = Neutral100,
    onSurfaceVariant  = Neutral600,

    error             = ErrorRed,
    errorContainer    = ErrorRedContainer,

    outline           = Neutral900.copy(alpha = 0.18f),
    outlineVariant    = Neutral900.copy(alpha = 0.08f),
)

@Composable
expect fun determineColorScheme(
    darkTheme: Boolean,
    dynamicColor: Boolean
): ColorScheme

@Composable
fun OwoFinanceTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    // Dynamic color is available on Android 12+
    dynamicColor: Boolean = true,
    content: @Composable () -> Unit
) {
    val colorScheme = determineColorScheme(darkTheme, dynamicColor)

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
