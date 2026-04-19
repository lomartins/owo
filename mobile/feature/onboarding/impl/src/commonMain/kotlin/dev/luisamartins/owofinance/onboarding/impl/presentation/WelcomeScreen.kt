package dev.luisamartins.owofinance.onboarding.impl.presentation

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun WelcomeScreen(
    onGetStarted: () -> Unit,
) {
    WelcomeContent(onGetStarted = onGetStarted)
}

@Composable
private fun WelcomeContent(
    onGetStarted: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(colors.background),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Spacer(Modifier.weight(1f))

        Box(
            modifier = Modifier
                .size(80.dp)
                .clip(RoundedCornerShape(7.dp))
                .background(colors.surface)
        )

        Spacer(Modifier.height(10.dp))

        Text(
            text = "owó",
            fontSize = 42.sp,
            fontWeight = FontWeight.Medium,
            color = colors.primary,
            textAlign = TextAlign.Center,
            letterSpacing = (-0.84).sp,
        )

        Spacer(Modifier.height(4.dp))

        Text(
            text = "personal finance",
            fontSize = 13.sp,
            fontWeight = FontWeight.Normal,
            color = colors.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )

        Spacer(Modifier.weight(2.5f))

        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 30.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Button(
                onClick = onGetStarted,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(44.dp),
                shape = RoundedCornerShape(12.dp),
                colors = ButtonDefaults.buttonColors(containerColor = colors.primary),
            ) {
                Text(
                    text = "Get started",
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Medium,
                    color = colors.onPrimary,
                )
            }

            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(44.dp)
                    .border(1.dp, colors.outline, RoundedCornerShape(12.dp)),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "Sign in · coming soon",
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Normal,
                    color = colors.onSurfaceVariant,
                )
            }
        }

        Spacer(Modifier.height(36.dp))
    }
}
