package dev.luisamartins.owofinance.domain.model

import kotlinx.serialization.Serializable

@Serializable
data class InstallmentGroup(
    val id: String,
    val totalAmount: Long,
    val totalInstallments: Int,
    val description: String
)
