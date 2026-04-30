package dev.luisamartins.owofinance.accounts.impl.presentation

import androidx.lifecycle.ViewModel
import dev.luisamartins.owofinance.domain.model.AccountType
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

data class AddAccountUiState(
    val name: String = "",
    val type: AccountType = AccountType.DEBIT,
)

class AddAccountViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(AddAccountUiState())
    val uiState: StateFlow<AddAccountUiState> = _uiState

    fun setName(name: String) { _uiState.value = _uiState.value.copy(name = name) }
    fun setType(type: AccountType) { _uiState.value = _uiState.value.copy(type = type) }
}
