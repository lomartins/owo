package dev.luisamartins.owofinance.domain.model

import kotlinx.datetime.LocalDate
import kotlinx.serialization.Serializable

@Serializable
data class Transaction(
    val id: String,
    val description: String,
    val amount: Long,
    val date: LocalDate,
    val categoryId: String,
    val accountId: String,
    val cardId: String? = null,
    val installmentGroupId: String? = null
)
