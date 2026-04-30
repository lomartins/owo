package dev.luisamartins.owofinance.ui.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Menu
import androidx.compose.material.icons.filled.Person
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

enum class BottomNavTab { DASHBOARD, TRANSACTIONS, ACCOUNTS }

@Composable
fun OwoBottomBar(
    selectedTab: BottomNavTab,
    onTabSelected: (BottomNavTab) -> Unit,
) {
    NavigationBar(containerColor = MaterialTheme.colorScheme.surface) {
        NavigationBarItem(
            selected = selectedTab == BottomNavTab.DASHBOARD,
            onClick = { onTabSelected(BottomNavTab.DASHBOARD) },
            icon = { Icon(Icons.Default.Home, contentDescription = null) },
            label = { Text("Dashboard") },
        )
        NavigationBarItem(
            selected = selectedTab == BottomNavTab.TRANSACTIONS,
            onClick = { onTabSelected(BottomNavTab.TRANSACTIONS) },
            icon = { Icon(Icons.Default.Menu, contentDescription = null) },
            label = { Text("Transactions") },
        )
        NavigationBarItem(
            selected = selectedTab == BottomNavTab.ACCOUNTS,
            onClick = { onTabSelected(BottomNavTab.ACCOUNTS) },
            icon = { Icon(Icons.Default.Person, contentDescription = null) },
            label = { Text("Accounts") },
        )
    }
}
