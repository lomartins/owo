package dev.luisamartins.owofinance.accounts.impl.presentation

import androidx.lifecycle.ViewModel
import dev.luisamartins.owofinance.domain.model.Account
import dev.luisamartins.owofinance.domain.model.AccountType
import dev.luisamartins.owofinance.domain.model.Card
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

data class AccountsUiState(
    val accounts: List<Account> = emptyList(),
    val cards: List<Card> = emptyList(),
)

class AccountsViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(
        AccountsUiState(
            accounts = listOf(
                Account("1", "Nubank", 342000L, AccountType.DEBIT),
                Account("2", "Carteira", 90000L, AccountType.DEBIT),
            ),
            cards = listOf(
                Card("c1", "Nubank", "1234", 600000L, 10, 17, "1"),
            ),
        )
    )
    val uiState: StateFlow<AccountsUiState> = _uiState
}
