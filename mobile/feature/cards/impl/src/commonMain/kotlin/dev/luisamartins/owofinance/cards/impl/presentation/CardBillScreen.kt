package dev.luisamartins.owofinance.cards.impl.presentation

import androidx.compose.foundation.background
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import dev.luisamartins.owofinance.ui.util.formatCurrency

@Composable
fun CardBillScreen(
    viewModel: CardBillViewModel,
    onNavigateBack: () -> Unit,
) {
    val state by viewModel.uiState.collectAsState()
    val colors = MaterialTheme.colorScheme

    Scaffold(containerColor = colors.background) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .verticalScroll(rememberScrollState()),
        ) {
            Row(
                modifier = Modifier.padding(horizontal = 8.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconButton(onClick = onNavigateBack) {
                    Icon(Icons.Default.ArrowBack, contentDescription = "Back", tint = colors.onBackground)
                }
                Text(
                    text = state.card?.name ?: "Card Bill",
                    fontSize = 20.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.onBackground,
                )
            }

            Card(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 24.dp),
                shape = RoundedCornerShape(16.dp),
                colors = CardDefaults.cardColors(containerColor = colors.primaryContainer),
            ) {
                Column(modifier = Modifier.fillMaxWidth().padding(24.dp)) {
                    Text(text = "Current bill", fontSize = 13.sp, color = colors.onPrimaryContainer.copy(alpha = 0.7f))
                    Spacer(Modifier.height(4.dp))
                    Text(
                        text = state.currentBill.formatCurrency(),
                        fontSize = 32.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = colors.onPrimaryContainer,
                        letterSpacing = (-0.5).sp,
                    )
                    Spacer(Modifier.height(12.dp))
                    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                        Column {
                            Text(text = "Due date", fontSize = 12.sp, color = colors.onPrimaryContainer.copy(alpha = 0.7f))
                            Text(
                                text = state.dueDate?.toString() ?: "-",
                                fontSize = 13.sp,
                                fontWeight = FontWeight.Medium,
                                color = colors.onPrimaryContainer,
                            )
                        }
                        state.card?.let { card ->
                            Column(horizontalAlignment = Alignment.End) {
                                Text(text = "Limit", fontSize = 12.sp, color = colors.onPrimaryContainer.copy(alpha = 0.7f))
                                Text(
                                    text = card.limit.formatCurrency(),
                                    fontSize = 13.sp,
                                    fontWeight = FontWeight.Medium,
                                    color = colors.onPrimaryContainer,
                                )
                            }
                        }
                    }
                }
            }

            Spacer(Modifier.height(24.dp))

            Text(
                text = "THIS CYCLE",
                fontSize = 12.sp,
                fontWeight = FontWeight.SemiBold,
                color = colors.onSurfaceVariant,
                letterSpacing = 1.sp,
                modifier = Modifier.padding(horizontal = 24.dp),
            )
            Spacer(Modifier.height(8.dp))
            Column(
                modifier = Modifier.padding(horizontal = 24.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                state.transactions.forEach { BillTransactionRow(transaction = it) }
            }
            Spacer(Modifier.height(16.dp))
        }
    }
}

@Composable
private fun BillTransactionRow(transaction: Transaction) {
    val colors = MaterialTheme.colorScheme
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
                    .background(colors.errorContainer),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = transaction.description.first().uppercaseChar().toString(),
                    fontSize = 14.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.onErrorContainer,
                )
            }
            Spacer(Modifier.width(12.dp))
            Column {
                Text(text = transaction.description, fontSize = 14.sp, fontWeight = FontWeight.Medium, color = colors.onSurface)
                Text(text = transaction.categoryId, fontSize = 12.sp, color = colors.onSurfaceVariant)
            }
        }
        Text(
            text = transaction.amount.formatCurrency(),
            fontSize = 14.sp,
            fontWeight = FontWeight.SemiBold,
            color = colors.error,
        )
    }
}
