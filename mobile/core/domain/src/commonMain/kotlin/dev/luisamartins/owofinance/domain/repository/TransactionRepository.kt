package dev.luisamartins.owofinance.domain.repository

import dev.luisamartins.owofinance.domain.model.Transaction
import kotlinx.coroutines.flow.Flow

interface TransactionRepository {
    fun getTransactions(): Flow<List<Transaction>>
    fun getTransactionsByAccount(accountId: String): Flow<List<Transaction>>
    fun getTransactionsByCard(cardId: String): Flow<List<Transaction>>
    suspend fun insertTransaction(transaction: Transaction)
}
