package dev.luisamartins.owofinance.transactions.impl.presentation

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

data class AddTransactionUiState(
    val amount: String = "",
    val description: String = "",
    val isExpense: Boolean = true,
)

class AddTransactionViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(AddTransactionUiState())
    val uiState: StateFlow<AddTransactionUiState> = _uiState

    fun setAmount(amount: String) { _uiState.value = _uiState.value.copy(amount = amount) }
    fun setDescription(description: String) { _uiState.value = _uiState.value.copy(description = description) }
    fun toggleType() { _uiState.value = _uiState.value.copy(isExpense = !_uiState.value.isExpense) }
}
