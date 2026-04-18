package dev.luisamartins.owofinance.domain.repository

import dev.luisamartins.owofinance.domain.model.Card
import kotlinx.coroutines.flow.Flow

interface CardRepository {
    fun getCards(): Flow<List<Card>>
    suspend fun getCardById(id: String): Card?
    suspend fun insertCard(card: Card)
}
