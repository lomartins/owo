package dev.luisamartins.owofinance.transactions.impl.presentation

import androidx.lifecycle.ViewModel
import dev.luisamartins.owofinance.domain.model.Transaction
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.datetime.LocalDate

data class TransactionsUiState(
    val transactions: List<Transaction> = emptyList(),
)

class TransactionsViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(
        TransactionsUiState(
            transactions = listOf(
                Transaction("t1", "Supermercado", -28000L, LocalDate(2026, 4, 20), "food", "1"),
                Transaction("t2", "Salário", 350000L, LocalDate(2026, 4, 15), "salary", "1"),
                Transaction("t3", "Netflix", -5500L, LocalDate(2026, 4, 14), "entertainment", "1"),
                Transaction("t4", "Zara", -19900L, LocalDate(2026, 4, 12), "shopping", "1", "c1"),
                Transaction("t5", "Transporte", -3200L, LocalDate(2026, 4, 10), "transport", "1"),
            ),
        )
    )
    val uiState: StateFlow<TransactionsUiState> = _uiState
}
