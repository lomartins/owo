package dev.luisamartins.owofinance.domain.model

import kotlinx.serialization.Serializable

@Serializable
data class Card(
    val id: String,
    val name: String,
    val lastFourDigits: String,
    val limit: Long,
    val closingDay: Int,
    val dueDay: Int,
    val accountId: String
)
