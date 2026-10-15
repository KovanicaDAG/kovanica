package io.kovanica.light

import android.content.Context
import android.content.SharedPreferences
import org.json.JSONObject

/**
 * Comms settings and the local contact book, persisted in the app's existing
 * `"kvnc"` SharedPreferences file (same store as `keystore`, `rpc`, ...).
 *
 * Keys: `relay_url`, `display_name`, `contacts`. The contacts map is stored as
 * a compact JSON object of `pubkeyHex -> label`.
 */
class CommsStore(context: Context) {
    private val prefs: SharedPreferences =
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    /** Relay origin, e.g. `http://10.0.2.2:8787`. Defaults to the emulator host. */
    var relayUrl: String
        get() = prefs.getString(KEY_RELAY_URL, DEFAULT_RELAY_URL) ?: DEFAULT_RELAY_URL
        set(value) = prefs.edit().putString(KEY_RELAY_URL, value).apply()

    /** Friendly name shown for this identity. Empty until the user sets one. */
    var displayName: String
        get() = prefs.getString(KEY_DISPLAY_NAME, "") ?: ""
        set(value) = prefs.edit().putString(KEY_DISPLAY_NAME, value).apply()

    /** Snapshot of the contact book (`pubkeyHex -> label`, keys normalized to lowercase hex). */
    fun contacts(): Map<String, String> {
        val raw = prefs.getString(KEY_CONTACTS, null) ?: return emptyMap()
        return try {
            val json = JSONObject(raw)
            val out = LinkedHashMap<String, String>()
            for (key in json.keys()) {
                out[key] = json.optString(key)
            }
            out
        } catch (_: Exception) {
            emptyMap()
        }
    }

    /** Label for [pubkeyHex], or null when unknown. */
    fun label(pubkeyHex: String): String? = contacts()[normalize(pubkeyHex)]

    /** Insert or overwrite a contact. */
    fun putContact(pubkeyHex: String, label: String) {
        val map = contacts().toMutableMap()
        map[normalize(pubkeyHex)] = label
        save(map)
    }

    /** Drop a contact. No-op when absent. */
    fun removeContact(pubkeyHex: String) {
        val map = contacts().toMutableMap()
        if (map.remove(normalize(pubkeyHex)) != null) {
            save(map)
        }
    }

    /** Remove every contact. */
    fun clearContacts() {
        prefs.edit().remove(KEY_CONTACTS).apply()
    }

    private fun save(map: Map<String, String>) {
        val json = JSONObject()
        for ((pubkey, label) in map) {
            json.put(pubkey, label)
        }
        prefs.edit().putString(KEY_CONTACTS, json.toString()).apply()
    }

    private fun normalize(pubkeyHex: String): String =
        pubkeyHex.trim().removePrefix("0x").removePrefix("0X").lowercase()

    companion object {
        const val PREFS: String = "kvnc"
        const val KEY_RELAY_URL: String = "relay_url"
        const val KEY_DISPLAY_NAME: String = "display_name"
        const val KEY_CONTACTS: String = "contacts"

        /** Emulator loopback to the host machine (matches the spec default). */
        const val DEFAULT_RELAY_URL: String = "http://10.0.2.2:8787"
    }
}
