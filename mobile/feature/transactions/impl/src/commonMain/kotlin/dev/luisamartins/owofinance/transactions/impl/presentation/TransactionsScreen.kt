package dev.luisamartins.owofinance.transactions.impl.presentation

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
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
import dev.luisamartins.owofinance.domain.model.Transaction
import dev.luisamartins.owofinance.ui.components.BottomNavTab
import dev.luisamartins.owofinance.ui.components.OwoBottomBar
import dev.luisamartins.owofinance.ui.util.formatCurrency

@Composable
fun TransactionsScreen(
    viewModel: TransactionsViewModel,
    onNavigateToDashboard: () -> Unit,
    onNavigateToAccounts: () -> Unit,
    onAddTransaction: () -> Unit,
) {
    val state by viewModel.uiState.collectAsState()
    val colors = MaterialTheme.colorScheme
    Scaffold(
        bottomBar = {
            OwoBottomBar(
                selectedTab = BottomNavTab.TRANSACTIONS,
                onTabSelected = { tab ->
                    when (tab) {
                        BottomNavTab.DASHBOARD -> onNavigateToDashboard()
                        BottomNavTab.ACCOUNTS -> onNavigateToAccounts()
                        else -> Unit
                    }
                },
            )
        },
        floatingActionButton = {
            FloatingActionButton(
                onClick = onAddTransaction,
                containerColor = colors.primary,
                contentColor = colors.onPrimary,
            ) {
                Icon(Icons.Default.Add, contentDescription = "Add transaction")
            }
        },
        containerColor = colors.background,
    ) { innerPadding ->
        LazyColumn(modifier = Modifier.fillMaxSize().padding(innerPadding)) {
            item {
                Text(
                    text = "Transactions",
                    fontSize = 20.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.onBackground,
                    modifier = Modifier.padding(horizontal = 24.dp, vertical = 16.dp),
                )
            }
            items(state.transactions) { tx ->
                TransactionItem(
                    transaction = tx,
                    modifier = Modifier.padding(horizontal = 24.dp, vertical = 4.dp),
                )
            }
        }
    }
}

@Composable
private fun TransactionItem(transaction: Transaction, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    val isIncome = transaction.amount > 0
    Row(
        modifier = modifier
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
                Text(text = transaction.description, fontSize = 14.sp, fontWeight = FontWeight.Medium, color = colors.onSurface)
                Text(text = transaction.date.toString(), fontSize = 12.sp, color = colors.onSurfaceVariant)
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
