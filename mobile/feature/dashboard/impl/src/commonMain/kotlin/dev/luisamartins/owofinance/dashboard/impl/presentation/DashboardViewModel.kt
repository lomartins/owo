package dev.luisamartins.owofinance.dashboard.impl.presentation

import androidx.lifecycle.ViewModel
import dev.luisamartins.owofinance.domain.model.Account
import dev.luisamartins.owofinance.domain.model.AccountType
import dev.luisamartins.owofinance.domain.model.Card
import dev.luisamartins.owofinance.domain.model.Transaction
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.datetime.LocalDate

data class DashboardUiState(
    val totalBalance: Long = 0L,
    val accounts: List<Account> = emptyList(),
    val cards: List<Card> = emptyList(),
    val recentTransactions: List<Transaction> = emptyList(),
)

class DashboardViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(
        DashboardUiState(
            totalBalance = 432000L,
            accounts = listOf(
                Account("1", "Nubank", 342000L, AccountType.DEBIT),
                Account("2", "Carteira", 90000L, AccountType.DEBIT),
            ),
            cards = listOf(
                Card("c1", "Nubank", "1234", 600000L, 10, 17, "1"),
            ),
            recentTransactions = listOf(
                Transaction("t1", "Supermercado", -28000L, LocalDate(2026, 4, 20), "food", "1"),
                Transaction("t2", "Salário", 350000L, LocalDate(2026, 4, 15), "salary", "1"),
            ),
        )
    )
    val uiState: StateFlow<DashboardUiState> = _uiState
}
