package dev.luisamartins.owofinance.domain.model

import kotlinx.serialization.Serializable

@Serializable
data class Account(
    val id: String,
    val name: String,
    val balance: Long,
    val type: AccountType
)
