package io.kovanica.light

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.text.InputType
import android.view.View
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import org.json.JSONArray
import org.json.JSONObject
import uniffi.kvnc_light.confirmAccount
import uniffi.kvnc_light.createWallet
import uniffi.kvnc_light.faucetAllowed
import uniffi.kvnc_light.importWallet
import uniffi.kvnc_light.signBondRaw
import uniffi.kvnc_light.signExitRaw
import uniffi.kvnc_light.signTransferRaw
import uniffi.kvnc_light.signWitnessRaw
import uniffi.kvnc_light.verifyHeader
import uniffi.kvnc_light.walletAddress
import java.util.concurrent.Executors

/**
 * Light client. Crypto runs on a background thread because Argon2id uses 64 MiB.
 * A gap since the last header starts a new checkpoint. The desktop
 * `kvnc-lightnode` follows every header and submits witnesses while the phone sleeps.
 */
class MainActivity : Activity() {
    private val io = Executors.newSingleThreadExecutor()
    private lateinit var status: TextView
    private lateinit var rpcField: EditText
    private lateinit var chainField: EditText
    private lateinit var epochField: EditText
    private lateinit var passField: EditText
    private lateinit var mnemonicField: EditText
    private lateinit var toField: EditText
    private lateinit var amountField: EditText
    private lateinit var faucetButton: Button

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val prefs = getSharedPreferences("kvnc", MODE_PRIVATE)
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(32, 32, 32, 32)
        }
        rpcField = field("RPC", prefs.getString("rpc", "http://10.0.2.2:8545")!!)
        chainField = field("Chain id", prefs.getString("chainId", "1337")!!)
        epochField = field("Committee epoch", prefs.getString("epoch", "0")!!)
        passField = field("Passphrase", "").apply { inputType = InputType.TYPE_TEXT_VARIATION_PASSWORD }
        mnemonicField = field("24-word seed (import)", "")
        toField = field("Recipient", "")
        amountField = field("Amount (atoms)", "1")
        status = TextView(this).apply { text = "Kovanica light client" }
        root.addView(rpcField)
        root.addView(chainField)
        root.addView(epochField)
        root.addView(passField)
        root.addView(mnemonicField)
        root.addView(button("Create wallet") { onCreateWallet() })
        root.addView(button("Import wallet") { onImportWallet() })
        root.addView(button("Sync sealed header") { sync() })
        root.addView(button("Balance") { balance() })
        root.addView(toField)
        root.addView(amountField)
        root.addView(button("Send") { send() })
        root.addView(TextView(this).apply {
            text = "Light node: a desktop kvnc-lightnode submits the witness while this phone is asleep. Open the app about once per day."
        })
        root.addView(button("Bond 100 KUNA") { bond() })
        root.addView(button("Submit witness") { witness() })
        root.addView(button("Exit light node") { exitLight() })
        faucetButton = button("Testnet faucet") { faucet() }
        root.addView(faucetButton)
        root.addView(button("Open Messages") { openMessages() })
        root.addView(status)
        setContentView(ScrollView(this).apply { addView(root) })
        refreshFaucet()
    }

    override fun onDestroy() {
        io.shutdown()
        super.onDestroy()
    }

    private fun onCreateWallet() {
        val passphrase = passField.text.toString()
        background("creating wallet") {
            val created = createWallet(passphrase)
            prefs().edit().putString("keystore", created.keystoreJson).apply()
            "address ${created.address}\nmnemonic (shown once):\n${created.mnemonic}"
        }
    }

    private fun onImportWallet() {
        val passphrase = passField.text.toString()
        val mnemonic = mnemonicField.text.toString()
        background("importing wallet") {
            val imported = importWallet(mnemonic, passphrase)
            prefs().edit().putString("keystore", imported.keystoreJson).apply()
            "address ${imported.address}"
        }
    }

    private fun sync() {
        saveConfig()
        background("syncing") {
            val client = client()
            val header = client.call("kvnc_getLeaderHeader", JSONArray()) as? JSONObject
                ?: return@background "no sealed header yet"
            val height = parseQuantity(header.get("committedHeight"))
            val seals = client.call("kvnc_getCommitSeals", JSONArray().put(hex(height)))
            val committee = client.call("kvnc_getCommittee", JSONArray())
            val verified = verifyHeader(
                chainId(),
                epoch(),
                JSONArray(committee.toString()).toString(),
                header.toString(),
                JSONObject(seals.toString()).toString(),
                parentJson(height),
            )
            prefs().edit()
                .putString("header", header.toString())
                .putString("height", verified.committedHeight.toString())
                .apply()
            "verified height ${verified.committedHeight}\nheader ${verified.headerHash}\nepoch ${verified.epoch}"
        }
    }

    private fun balance() {
        background("reading balance") {
            val keystore = keystore()
            val address = walletAddress(keystore)
            val proof = client().call("kvnc_getAccountProof", JSONArray().put(address))
            if (proof == null) {
                return@background "address $address\nbalance unverified (no account proof)"
            }
            val header = prefs().getString("header", null)
                ?: return@background "address $address\nbalance unverified (sync a header first)"
            val snapshot = confirmAccount(header, JSONObject(proof.toString()).toString())
            "address $address\nbalance ${snapshot.balance} atoms\nnonce ${snapshot.nonce}\nverified"
        }
    }

    private fun send() {
        val to = toField.text.toString()
        val amount = amountField.text.toString().toULong()
        val passphrase = passField.text.toString()
        background("sending") {
            val keystore = keystore()
            val address = walletAddress(keystore)
            val node = client()
            val nonce = parseQuantity(node.call("kvnc_getNonce", JSONArray().put(address)))
            val rate = parseQuantity(node.call("kvnc_estimateFee", JSONArray()))
            val raw = signTransferRaw(keystore, passphrase, chainId(), to, amount, nonce, rate)
            val hash = node.call("kvnc_sendRawTransaction", JSONArray().put(raw))
            "submitted $hash"
        }
    }

    private fun bond() {
        val passphrase = passField.text.toString()
        background("bonding") {
            submit("bond", passphrase) { keystore, address, nonce, rate ->
                signBondRaw(keystore, passphrase, chainId(), MIN_LIGHT_BOND, nonce, rate)
                    .also { _ -> address }
            }
        }
    }

    private fun witness() {
        val passphrase = passField.text.toString()
        background("witness") {
            val keystore = keystore()
            val address = walletAddress(keystore)
            val set = JSONObject(client().call("kvnc_getLightSet", JSONArray()).toString())
            if (set.isNull("beacon")) {
                return@background "epoch ${parseQuantity(set.get("epoch"))} has no beacon yet"
            }
            val node = client()
            val nonce = parseQuantity(node.call("kvnc_getNonce", JSONArray().put(address)))
            val rate = parseQuantity(node.call("kvnc_estimateFee", JSONArray()))
            val raw = signWitnessRaw(
                keystore,
                passphrase,
                chainId(),
                parseQuantity(set.get("epoch")),
                set.getString("beacon"),
                nonce,
                rate,
            )
            val hash = node.call("kvnc_sendRawTransaction", JSONArray().put(raw))
            "witness submitted $hash\n${lightLine(set, address)}"
        }
    }

    private fun exitLight() {
        val passphrase = passField.text.toString()
        background("exiting") {
            submit("exit", passphrase) { keystore, _, nonce, rate ->
                signExitRaw(keystore, passphrase, chainId(), nonce, rate)
            }
        }
    }

    private fun faucet() {
        background("faucet") {
            val address = walletAddress(keystore())
            client().postFaucet(address)
        }
    }

    private fun openMessages() {
        startActivity(Intent(this, MessagesActivity::class.java))
    }

    private fun submit(
        label: String,
        passphrase: String,
        sign: (keystore: String, address: String, nonce: ULong, rate: ULong) -> String,
    ): String {
        val keystore = keystore()
        val address = walletAddress(keystore)
        val node = client()
        val nonce = parseQuantity(node.call("kvnc_getNonce", JSONArray().put(address)))
        val rate = parseQuantity(node.call("kvnc_estimateFee", JSONArray()))
        val raw = sign(keystore, address, nonce, rate)
        val hash = node.call("kvnc_sendRawTransaction", JSONArray().put(raw))
        return "$label submitted $hash"
    }

    private fun parentJson(height: ULong): String {
        val stored = prefs().getString("header", null) ?: return ""
        val previous = parseQuantity(JSONObject(stored).get("committedHeight"))
        return if (previous + 1uL == height) stored else ""
    }

    private fun lightLine(set: JSONObject, address: String): String {
        val nodes = set.optJSONArray("nodes") ?: JSONArray()
        for (i in 0 until nodes.length()) {
            val node = nodes.getJSONObject(i)
            if (node.optString("address") == address) {
                return "active ${node.optBoolean("active")} witnessed ${node.optBoolean("witnessed")} selected ${node.optBoolean("selected")}"
            }
        }
        return "bond: not registered"
    }

    private fun background(label: String, work: () -> String) {
        show(label)
        refreshFaucet()
        io.execute {
            val text = try {
                work()
            } catch (err: Exception) {
                err.message ?: err.toString()
            }
            runOnUiThread { show(text) }
        }
    }

    private fun refreshFaucet() {
        val allowed = try {
            faucetAllowed(chainId())
        } catch (_: Exception) {
            false
        }
        faucetButton.visibility = if (allowed) View.VISIBLE else View.GONE
    }

    private fun saveConfig() {
        prefs().edit()
            .putString("rpc", rpcField.text.toString())
            .putString("chainId", chainField.text.toString())
            .putString("epoch", epochField.text.toString())
            .apply()
    }

    private fun client() = NodeClient(rpcField.text.toString())
    private fun prefs() = getSharedPreferences("kvnc", MODE_PRIVATE)
    private fun keystore(): String = prefs().getString("keystore", null) ?: error("create or import a wallet first")
    private fun chainId(): ULong = chainField.text.toString().toULong()
    private fun epoch(): ULong = epochField.text.toString().toULong()
    private fun hex(value: ULong) = "0x" + value.toString(16)
    private fun show(text: String) { status.text = text }

    private fun field(hint: String, value: String) = EditText(this).apply {
        this.hint = hint
        setText(value)
    }

    private fun button(label: String, action: () -> Unit) = Button(this).apply {
        text = label
        setOnClickListener { action() }
    }

    companion object {
        /** Matches `MIN_LIGHT_BOND` (100 KUNA, 9 decimals). */
        private val MIN_LIGHT_BOND: ULong = 100_000_000_000u
    }
}
