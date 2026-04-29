package dev.luisamartins.owofinance.ui.assets

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.painter.Painter
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource

class OwoDrawable (
    val drawable: DrawableResource
) {
    companion object
}

@Composable
fun OwoDrawable.toPainter(): Painter {
    return painterResource(drawable)
}