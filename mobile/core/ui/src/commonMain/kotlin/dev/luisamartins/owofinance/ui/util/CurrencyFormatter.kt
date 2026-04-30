package dev.luisamartins.owofinance.ui.util

import kotlin.math.abs

fun Long.formatCurrency(): String {
    val negative = this < 0
    val cents = abs(this)
    val reais = cents / 100
    val centavos = cents % 100
    val reaisStr = reais.toString()
    val grouped = buildString {
        val len = reaisStr.length
        val first = len % 3
        if (first > 0) append(reaisStr.substring(0, first))
        var i = first
        while (i < len) {
            if (isNotEmpty()) append(".")
            append(reaisStr.substring(i, i + 3))
            i += 3
        }
        if (isEmpty()) append("0")
    }
    return "${if (negative) "- " else ""}R\$ $grouped,${centavos.toString().padStart(2, '0')}"
}
