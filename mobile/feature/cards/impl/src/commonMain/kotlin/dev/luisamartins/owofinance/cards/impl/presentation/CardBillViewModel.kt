package dev.luisamartins.owofinance.cards.impl.presentation

import androidx.lifecycle.ViewModel
import dev.luisamartins.owofinance.domain.model.Card
import dev.luisamartins.owofinance.domain.model.Transaction
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.datetime.LocalDate

data class CardBillUiState(
    val card: Card? = null,
    val currentBill: Long = 0L,
    val dueDate: LocalDate? = null,
    val transactions: List<Transaction> = emptyList(),
)

class CardBillViewModel(private val cardId: String) : ViewModel() {
    private val _uiState = MutableStateFlow(
        CardBillUiState(
            card = Card("c1", "Nubank", "1234", 600000L, 10, 17, "1"),
            currentBill = 48000L,
            dueDate = LocalDate(2026, 5, 17),
            transactions = listOf(
                Transaction("ct1", "Zara", -19900L, LocalDate(2026, 4, 12), "shopping", "1", "c1"),
                Transaction("ct2", "Netflix", -5500L, LocalDate(2026, 4, 14), "entertainment", "1", "c1"),
                Transaction("ct3", "Supermercado", -22600L, LocalDate(2026, 4, 18), "food", "1", "c1"),
            ),
        )
    )
    val uiState: StateFlow<CardBillUiState> = _uiState
}
