package dev.luisamartins.owofinance.onboarding.impl.presentation

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigationevent.NavigationEventInfo
import androidx.navigationevent.compose.NavigationBackHandler
import androidx.navigationevent.compose.rememberNavigationEventState
import dev.luisamartins.owofinance.ui.theme.OwoFinanceTheme
import kotlinx.coroutines.launch
import org.koin.compose.viewmodel.koinViewModel

@Composable
fun OnboardingScreen(
    onFinish: () -> Unit,
    onNavigateBack: () -> Unit,
    viewModel: OnboardingViewModel = koinViewModel()
) {
    OnboardingContent(
        onNavigateBack = onNavigateBack,
        onFinish = {
            viewModel.onFinish()
            onFinish.invoke()
        }
    )
}

@Composable
private fun OnboardingContent(
    onNavigateBack: () -> Unit,
    onFinish: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val pagerState = rememberPagerState(pageCount = { 3 })
    val scope = rememberCoroutineScope()

    val navState = rememberNavigationEventState(NavigationEventInfo.None)

    // Setup Back handling

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(colors.background),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Spacer(Modifier.height(80.dp))

        HorizontalPager(
            state = pagerState,
            modifier = Modifier.weight(1f).fillMaxWidth()
        ) { page ->
            Column(
                modifier = Modifier.fillMaxSize(),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                when (page) {
                    0 -> OnboardingIllustration1()
                    1 -> OnboardingIllustration2()
                    2 -> OnboardingIllustration3()
                }

                Spacer(Modifier.height(100.dp))

                Text(
                    text = when (page) {
                        0 -> "Track everything"
                        1 -> "Know your balance"
                        else -> "Your data, your device"
                    },
                    fontSize = 22.sp,
                    fontWeight = FontWeight.Medium,
                    color = colors.onBackground,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.padding(horizontal = 24.dp)
                 )

                Spacer(Modifier.height(10.dp))

                Text(
                    text = when (page) {
                        0 -> "Add income and expenses, categorize, and see where your money goes."
                        1 -> "See your total balance across all accounts, updated in real time."
                        else -> "Everything stays local. No cloud, no account required. Your finances, private."
                    },
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Normal,
                    color = colors.onSurfaceVariant,
                    textAlign = TextAlign.Center,
                    lineHeight = 21.sp,
                    modifier = Modifier.padding(horizontal = 32.dp)
                )
            }
        }

        Row(
            modifier = Modifier
                .padding(bottom = 92.dp),
            horizontalArrangement = Arrangement.spacedBy(6.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            repeat(3) { index ->
                val isSelected = pagerState.currentPage == index
                Box(
                    modifier = Modifier
                        .height(6.dp)
                        .width(if (isSelected) 18.dp else 6.dp)
                        .clip(RoundedCornerShape(3.dp))
                        .background(
                            if (isSelected) colors.primary else colors.onSurfaceVariant.copy(
                                alpha = 0.35f
                            )
                        )
                )
            }
        }

        Button(
            onClick = {
                if (pagerState.currentPage < 2) {
                    scope.launch {
                        pagerState.animateScrollToPage(pagerState.currentPage + 1)
                    }
                } else {
                    onFinish()
                }
            },
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp)
                .padding(bottom = 42.dp) // Plus safe area
                .height(44.dp),
            shape = RoundedCornerShape(12.dp),
            colors = ButtonDefaults.buttonColors(containerColor = colors.primary),
        ) {
            Text(
                text = if (pagerState.currentPage < 2) "Next" else "Start using owo",
                fontSize = 15.sp,
                fontWeight = FontWeight.Medium,
                color = colors.onPrimary,
            )
        }
    }
}

@Composable
private fun OnboardingIllustration1() {
    val colors = MaterialTheme.colorScheme
    Box(
        modifier = Modifier
            .size(322.dp, 180.dp)
            .clip(RoundedCornerShape(16.dp))
            .background(colors.primaryContainer)
    ) {
        Box(
            modifier = Modifier.padding(top = 60.dp, start = 101.dp).size(100.dp, 60.dp)
                .clip(RoundedCornerShape(8.dp)).background(colors.primary.copy(alpha = 0.15f))
        )
        Box(
            modifier = Modifier.padding(top = 74.dp, start = 111.dp).size(42.dp, 6.dp)
                .clip(RoundedCornerShape(3.dp)).background(colors.primary.copy(alpha = 0.7f))
        )
        Box(
            modifier = Modifier.padding(top = 86.dp, start = 111.dp).size(70.dp, 6.dp)
                .clip(RoundedCornerShape(3.dp)).background(colors.primary.copy(alpha = 0.45f))
        )
        Box(
            modifier = Modifier.padding(top = 98.dp, start = 111.dp).size(56.dp, 6.dp)
                .clip(RoundedCornerShape(3.dp)).background(colors.primary.copy(alpha = 0.3f))
        )
        Box(
            modifier = Modifier.padding(top = 50.dp, start = 201.dp).size(20.dp)
                .clip(RoundedCornerShape(10.dp)).background(colors.primary),
            contentAlignment = Alignment.Center
        ) {
            Text("✓", color = colors.onPrimary, fontSize = 11.sp, fontWeight = FontWeight.Medium)
        }
    }
}

@Composable
private fun OnboardingIllustration2() {
    val colors = MaterialTheme.colorScheme
    Box(
        modifier = Modifier
            .size(322.dp, 180.dp)
            .clip(RoundedCornerShape(16.dp))
            .background(colors.primaryContainer),
        contentAlignment = Alignment.Center
    ) {
        Box(
            modifier = Modifier
                .size(70.dp)
                .border(4.dp, colors.primary, CircleShape),
            contentAlignment = Alignment.Center
        ) {
            Text(
                "R\$4.3k",
                color = colors.primary,
                fontSize = 11.sp,
                fontWeight = FontWeight.Medium
            )
        }
    }
}

@Composable
private fun OnboardingIllustration3() {
    val colors = MaterialTheme.colorScheme
    Box(
        modifier = Modifier
            .size(322.dp, 180.dp)
            .clip(RoundedCornerShape(16.dp))
            .background(colors.primaryContainer),
        contentAlignment = Alignment.Center
    ) {
        Box(
            modifier = Modifier
                .size(50.dp, 60.dp)
                .clip(RoundedCornerShape(8.dp))
                .background(colors.primary.copy(alpha = 0.15f)),
            contentAlignment = Alignment.Center
        ) {
            Box(
                modifier = Modifier
                    .padding(top = 14.dp)
                    .size(20.dp, 16.dp)
                    .clip(RoundedCornerShape(4.dp))
                    .background(colors.primary)
            )
            Box(
                modifier = Modifier
                    .padding(bottom = 10.dp)
                    .size(14.dp)
                    .border(2.dp, colors.primary, CircleShape)
            )
        }
    }
}

@Preview
@Composable
fun OnboardingScreenPreview() {
    OwoFinanceTheme {
        OnboardingContent(
            onFinish = {},
            onNavigateBack = {},
        )
    }
}
