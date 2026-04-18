package dev.luisamartins.owofinance.domain.repository

import dev.luisamartins.owofinance.domain.model.Account
import kotlinx.coroutines.flow.Flow

interface AccountRepository {
    fun getAccounts(): Flow<List<Account>>
    suspend fun getAccountById(id: String): Account?
    suspend fun insertAccount(account: Account)
}
