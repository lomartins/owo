package dev.luisamartins.owofinance.domain.model

import kotlinx.serialization.Serializable

@Serializable
data class Category(
    val id: String,
    val name: String,
    val icon: String
)
