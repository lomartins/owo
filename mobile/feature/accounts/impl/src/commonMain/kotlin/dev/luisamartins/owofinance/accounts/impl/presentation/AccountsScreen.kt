package dev.luisamartins.owofinance.accounts.impl.presentation

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.luisamartins.owofinance.domain.model.Account
import dev.luisamartins.owofinance.domain.model.Card
import dev.luisamartins.owofinance.ui.components.BottomNavTab
import dev.luisamartins.owofinance.ui.components.OwoBottomBar
import dev.luisamartins.owofinance.ui.util.formatCurrency

@Composable
fun AccountsScreen(
    viewModel: AccountsViewModel,
    onNavigateToDashboard: () -> Unit,
    onNavigateToTransactions: () -> Unit,
    onAddAccount: () -> Unit,
    onAddCard: () -> Unit,
    onNavigateToCardBill: (String) -> Unit,
) {
    val state by viewModel.uiState.collectAsState()
    AccountsContent(
        state = state,
        onNavigateToDashboard = onNavigateToDashboard,
        onNavigateToTransactions = onNavigateToTransactions,
        onAddAccount = onAddAccount,
        onAddCard = onAddCard,
        onNavigateToCardBill = onNavigateToCardBill,
    )
}

@Composable
private fun AccountsContent(
    state: AccountsUiState,
    onNavigateToDashboard: () -> Unit,
    onNavigateToTransactions: () -> Unit,
    onAddAccount: () -> Unit,
    onAddCard: () -> Unit,
    onNavigateToCardBill: (String) -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Scaffold(
        floatingActionButton = {
            FloatingActionButton(
                onClick = onAddAccount,
                containerColor = colors.primary,
                contentColor = colors.onPrimary,
            ) {
                Icon(Icons.Default.Add, contentDescription = "Add account")
            }
        },
        containerColor = colors.background,
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .verticalScroll(rememberScrollState()),
        ) {
            Text(
                text = "Accounts",
                fontSize = 20.sp,
                fontWeight = FontWeight.SemiBold,
                color = colors.onBackground,
                modifier = Modifier.padding(horizontal = 24.dp, vertical = 16.dp),
            )

            Text(
                text = "Debit accounts",
                fontSize = 13.sp,
                fontWeight = FontWeight.SemiBold,
                color = colors.onSurfaceVariant,
                letterSpacing = 0.5.sp,
                modifier = Modifier.padding(horizontal = 24.dp),
            )
            Spacer(Modifier.height(8.dp))
            Column(
                modifier = Modifier.padding(horizontal = 24.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                state.accounts.forEach { AccountItem(account = it) }
            }

            if (state.cards.isNotEmpty()) {
                Spacer(Modifier.height(24.dp))
                Text(
                    text = "Credit cards",
                    fontSize = 13.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.onSurfaceVariant,
                    letterSpacing = 0.5.sp,
                    modifier = Modifier.padding(horizontal = 24.dp),
                )
                Spacer(Modifier.height(8.dp))
                Column(
                    modifier = Modifier.padding(horizontal = 24.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    state.cards.forEach { card ->
                        CardItem(card = card, onClick = { onNavigateToCardBill(card.id) })
                    }
                }
            }
            Spacer(Modifier.height(16.dp))
        }
    }
}

@Composable
private fun AccountItem(account: Account) {
    val colors = MaterialTheme.colorScheme
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        colors = CardDefaults.cardColors(containerColor = colors.surface),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Box(
                    modifier = Modifier.size(36.dp).clip(CircleShape).background(colors.primaryContainer),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = account.name.first().uppercaseChar().toString(),
                        fontSize = 14.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = colors.onPrimaryContainer,
                    )
                }
                Spacer(Modifier.width(12.dp))
                Column {
                    Text(text = account.name, fontSize = 15.sp, fontWeight = FontWeight.Medium, color = colors.onSurface)
                    Text(text = "Checking", fontSize = 12.sp, color = colors.onSurfaceVariant)
                }
            }
            Text(text = account.balance.formatCurrency(), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = colors.onSurface)
        }
    }
}

@Composable
private fun CardItem(card: Card, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Card(
        modifier = Modifier.fillMaxWidth().clickable { onClick() },
        shape = RoundedCornerShape(12.dp),
        colors = CardDefaults.cardColors(containerColor = colors.surface),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Box(
                    modifier = Modifier.size(36.dp).clip(CircleShape).background(colors.secondaryContainer),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = card.name.first().uppercaseChar().toString(),
                        fontSize = 14.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = colors.onSecondaryContainer,
                    )
                }
                Spacer(Modifier.width(12.dp))
                Column {
                    Text(text = card.name, fontSize = 15.sp, fontWeight = FontWeight.Medium, color = colors.onSurface)
                    Text(text = "•••• ${card.lastFourDigits}  ·  due ${card.dueDay}", fontSize = 12.sp, color = colors.onSurfaceVariant)
                }
            }
            Text(text = "Limit\n${card.limit.formatCurrency()}", fontSize = 12.sp, color = colors.onSurfaceVariant)
        }
    }
}
