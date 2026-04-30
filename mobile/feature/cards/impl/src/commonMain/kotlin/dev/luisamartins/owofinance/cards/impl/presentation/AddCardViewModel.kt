package dev.luisamartins.owofinance.cards.impl.presentation

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

data class AddCardUiState(
    val name: String = "",
    val lastFourDigits: String = "",
    val closingDay: String = "",
    val dueDay: String = "",
)

class AddCardViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(AddCardUiState())
    val uiState: StateFlow<AddCardUiState> = _uiState

    fun setName(name: String) { _uiState.value = _uiState.value.copy(name = name) }
    fun setLastFour(digits: String) { _uiState.value = _uiState.value.copy(lastFourDigits = digits) }
    fun setClosingDay(day: String) { _uiState.value = _uiState.value.copy(closingDay = day) }
    fun setDueDay(day: String) { _uiState.value = _uiState.value.copy(dueDay = day) }
}
