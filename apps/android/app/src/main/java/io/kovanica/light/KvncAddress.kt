package io.kovanica.light

import java.math.BigInteger
import java.util.Locale

/**
 * Address handling for the `kvnc<64-hex>dag` encoding (`kovanica-types::address`).
 *
 * The wallet address returned by the light FFI is already in the display form;
 * contract addresses are typed by hand and may be either the `kvnc…dag` form or
 * raw 32-byte hex with or without `0x`. Everything else in the wallet works
 * with the canonical lowercase 64-hex string, which is also what the token
 * `balances` map is keyed by.
 */
object KvncAddress {
    const val PREFIX: String = "kvnc"
    const val SUFFIX: String = "dag"

    private val HEX32 = Regex("^[0-9a-fA-F]{64}$")

    /**
     * Normalise an address typed by the user (or returned by the FFI) to a
     * lowercase 64-hex string, or `null` when it is not a valid 32-byte
     * address. A truncated or checksum-broken `kvnc…dag` string is rejected
     * here rather than shipped to the node.
     */
    fun normalizeHex(raw: String?): String? {
        val value = raw?.trim().orEmpty()
        if (value.isEmpty()) return null
        val core = stripWrapper(value)
        if (!HEX32.matches(core)) return null
        return core.lowercase(Locale.US)
    }

    /** True when [raw] is a usable address in any accepted form. */
    fun isValid(raw: String?): Boolean = normalizeHex(raw) != null

    /** Render a 64-hex (or any hex) address as `kvnc<hex>dag`. */
    fun display(hex: String): String {
        val core = hex.trim().removePrefix("0x").removePrefix("0X").lowercase(Locale.US)
        return "$PREFIX$core$SUFFIX"
    }

    /** `kvnc<hex>dag` with the middle elided, for lists and labels. */
    fun short(hex: String, keep: Int = 6): String {
        val core = hex.trim().removePrefix("0x").removePrefix("0X").lowercase(Locale.US)
        if (core.length <= keep * 2 + 1) return display(core)
        return "$PREFIX${core.take(keep)}…${core.takeLast(keep)}$SUFFIX"
    }

    private fun stripWrapper(value: String): String {
        if (value.length > PREFIX.length + SUFFIX.length &&
            value.startsWith(PREFIX, ignoreCase = true) &&
            value.endsWith(SUFFIX, ignoreCase = true)
        ) {
            return value.substring(PREFIX.length, value.length - SUFFIX.length)
        }
        return value.removePrefix("0x").removePrefix("0X")
    }
}

/**
 * Fixed-point formatting for token amounts.
 *
 * Token `decimals` describe the fractional part only — they say nothing about
 * the token's value. The wallet shows the base-unit integer verbatim next to
 * the scaled figure so a user can always reconcile what the chain stores.
 */
object Amounts {
    /** Values with more than this many fractional digits are shown as raw base units. */
    private const val MAX_DECIMALS = 30

    /** `1234` at 2 decimals -> `12.34`; trailing zeros are dropped. */
    fun format(value: BigInteger, decimals: Int): String {
        if (decimals <= 0) return value.toString()
        if (decimals > MAX_DECIMALS) return value.toString()
        val negative = value.signum() < 0
        val magnitude = value.abs()
        val base = BigInteger.TEN.pow(decimals)
        val whole = magnitude / base
        val fraction = (magnitude % base).toString().padStart(decimals, '0').trimEnd('0')
        return buildString {
            if (negative) append('-')
            append(whole)
            if (fraction.isNotEmpty()) {
                append('.')
                append(fraction)
            }
        }
    }
}
