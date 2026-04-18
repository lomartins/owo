package dev.luisamartins.owofinance.domain.model

import kotlinx.serialization.Serializable

@Serializable
enum class AccountType {
    DEBIT, CREDIT_CARD
}
