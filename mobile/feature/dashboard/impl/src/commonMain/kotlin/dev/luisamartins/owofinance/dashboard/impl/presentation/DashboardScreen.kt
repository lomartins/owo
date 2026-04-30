package dev.luisamartins.owofinance.dashboard.impl.presentation

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
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
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
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.luisamartins.owofinance.domain.model.Account
import dev.luisamartins.owofinance.domain.model.AccountType
import dev.luisamartins.owofinance.domain.model.Card
import dev.luisamartins.owofinance.domain.model.Transaction
import dev.luisamartins.owofinance.ui.components.BottomNavTab
import dev.luisamartins.owofinance.ui.components.OwoBottomBar
import dev.luisamartins.owofinance.ui.util.formatCurrency
import kotlinx.datetime.LocalDate

@Composable
fun DashboardScreen(
    viewModel: DashboardViewModel,
    onNavigateToTransactions: () -> Unit,
    onNavigateToAccounts: () -> Unit,
    onNavigateToAllTransactions: () -> Unit,
    onNavigateToCardBill: (String) -> Unit,
) {
    val state by viewModel.uiState.collectAsState()
    DashboardContent(
        state = state,
        onNavigateToTransactions = onNavigateToTransactions,
        onNavigateToAccounts = onNavigateToAccounts,
        onNavigateToAllTransactions = onNavigateToAllTransactions,
        onNavigateToCardBill = onNavigateToCardBill,
    )
}

@Composable
private fun DashboardContent(
    state: DashboardUiState,
    onNavigateToTransactions: () -> Unit,
    onNavigateToAccounts: () -> Unit,
    onNavigateToAllTransactions: () -> Unit,
    onNavigateToCardBill: (String) -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Scaffold(
        containerColor = colors.background,
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .verticalScroll(rememberScrollState()),
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp, vertical = 16.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "Dashboard",
                    fontSize = 20.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.onBackground,
                )
            }

            Column(modifier = Modifier.padding(horizontal = 24.dp)) {
                Text(text = "Total balance", fontSize = 13.sp, color = colors.onSurfaceVariant)
                Spacer(Modifier.height(4.dp))
                Text(
                    text = state.totalBalance.formatCurrency(),
                    fontSize = 32.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.onBackground,
                    letterSpacing = (-0.5).sp,
                )
            }

            Spacer(Modifier.height(24.dp))

            SectionHeader(title = "Debit accounts", onSeeAll = onNavigateToAccounts)
            Spacer(Modifier.height(8.dp))
            Column(
                modifier = Modifier.padding(horizontal = 24.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                state.accounts.forEach { AccountCard(account = it) }
            }

            Spacer(Modifier.height(24.dp))

            if (state.cards.isNotEmpty()) {
                SectionHeader(title = "Credit cards", onSeeAll = onNavigateToAccounts)
                Spacer(Modifier.height(8.dp))
                Column(
                    modifier = Modifier.padding(horizontal = 24.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    state.cards.forEach { card ->
                        CreditCardRow(card = card, onClick = { onNavigateToCardBill(card.id) })
                    }
                }
                Spacer(Modifier.height(24.dp))
            }

            SectionHeader(title = "Recent transactions", onSeeAll = onNavigateToAllTransactions)
            Spacer(Modifier.height(8.dp))
            Column(
                modifier = Modifier.padding(horizontal = 24.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                state.recentTransactions.forEach { TransactionRow(transaction = it) }
            }

            Spacer(Modifier.height(16.dp))
        }
    }
}

@Composable
private fun SectionHeader(title: String, onSeeAll: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Row(
        modifier = Modifier.fillMaxWidth().padding(horizontal = 24.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = title,
            fontSize = 13.sp,
            fontWeight = FontWeight.SemiBold,
            color = colors.onSurfaceVariant,
            letterSpacing = 0.5.sp,
        )
        Text(
            text = "See all",
            fontSize = 13.sp,
            color = colors.primary,
            modifier = Modifier.clickable { onSeeAll() },
        )
    }
}

@Composable
private fun AccountCard(account: Account) {
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
                    modifier = Modifier.size(36.dp).clip(CircleShape)
                        .background(colors.primaryContainer),
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
                Text(
                    text = account.name,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Medium,
                    color = colors.onSurface
                )
            }
            Text(
                text = account.balance.formatCurrency(),
                fontSize = 15.sp,
                fontWeight = FontWeight.SemiBold,
                color = colors.onSurface
            )
        }
    }
}

@Composable
private fun CreditCardRow(card: Card, onClick: () -> Unit) {
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
                    modifier = Modifier.size(36.dp).clip(CircleShape)
                        .background(colors.secondaryContainer),
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
                    Text(
                        text = card.name,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Medium,
                        color = colors.onSurface
                    )
                    Text(
                        text = "•••• ${card.lastFourDigits}",
                        fontSize = 12.sp,
                        color = colors.onSurfaceVariant
                    )
                }
            }
            Text(text = "Due ${card.dueDay}", fontSize = 12.sp, color = colors.onSurfaceVariant)
        }
    }
}

@Composable
private fun TransactionRow(transaction: Transaction) {
    val colors = MaterialTheme.colorScheme
    val isIncome = transaction.amount > 0
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(8.dp))
            .background(colors.surface)
            .padding(12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(if (isIncome) colors.primaryContainer else colors.errorContainer),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = transaction.description.first().uppercaseChar().toString(),
                    fontSize = 14.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = if (isIncome) colors.onPrimaryContainer else colors.onErrorContainer,
                )
            }
            Spacer(Modifier.width(12.dp))
            Column {
                Text(
                    text = transaction.description,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Medium,
                    color = colors.onSurface
                )
                Text(
                    text = transaction.date.toString(),
                    fontSize = 12.sp,
                    color = colors.onSurfaceVariant
                )
            }
        }
        Text(
            text = if (isIncome) "+${transaction.amount.formatCurrency()}" else transaction.amount.formatCurrency(),
            fontSize = 14.sp,
            fontWeight = FontWeight.SemiBold,
            color = if (isIncome) colors.primary else colors.error,
        )
    }
}

@Preview
@Composable
private fun DashboardPreview() {
    val mockState = DashboardUiState(
        totalBalance = 525050,
        accounts = listOf(
            Account(
                id = "1",
                name = "Checking",
                balance = 250000,
                type = AccountType.DEBIT,
            ),
            Account(
                id = "2",
                name = "Savings",
                balance = 275050,
                type = AccountType.DEBIT,
            )
        ),
        cards = listOf(
            Card(
                id = "card1",
                name = "Mastercard",
                lastFourDigits = "4242",
                limit = 500000,
                dueDay = 15,
                closingDay = 5,
                accountId = "1"
            )
        ),
        recentTransactions = listOf(
            Transaction(
                id = "tx1",
                description = "Groceries",
                amount = -4599,
                date = LocalDate(2026, 4, 27),
                categoryId = "cat1",
                accountId = "1",
                cardId = "card1",
                installmentGroupId = null
            ),
            Transaction(
                id = "tx2",
                description = "Salary",
                amount = 500000,
                date = LocalDate(2026, 4, 25),
                categoryId = "cat2",
                accountId = "1",
                cardId = null,
                installmentGroupId = null
            ),
            Transaction(
                id = "tx3",
                description = "Electricity bill",
                amount = -12000,
                date = LocalDate(2026, 4, 20),
                categoryId = "cat3",
                accountId = "1",
                cardId = null,
                installmentGroupId = null
            )
        )
    )

    DashboardContent(
        state = mockState,
        onNavigateToTransactions = {},
        onNavigateToAccounts = {},
        onNavigateToAllTransactions = {},
        onNavigateToCardBill = {}
    )
}