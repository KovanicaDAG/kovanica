package io.kovanica.light

import android.app.Activity
import android.app.AlertDialog
import android.content.Context
import android.content.Intent
import android.content.res.ColorStateList
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.ColorDrawable
import android.graphics.drawable.Drawable
import android.graphics.drawable.GradientDrawable
import android.graphics.drawable.RippleDrawable
import android.os.Bundle
import android.text.Editable
import android.text.InputType
import android.text.TextWatcher
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.AdapterView
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.Spinner
import android.widget.TextView
import org.json.JSONArray
import org.json.JSONObject
import uniffi.kovanica_light.confirmAccount
import uniffi.kovanica_light.createWallet
import uniffi.kovanica_light.faucetAllowed
import uniffi.kovanica_light.importWallet
import uniffi.kovanica_light.signBondRaw
import uniffi.kovanica_light.signExitRaw
import uniffi.kovanica_light.signTransferRaw
import uniffi.kovanica_light.signWitnessRaw
import uniffi.kovanica_light.verifyHeader
import uniffi.kovanica_light.walletAddress
import java.math.BigInteger
import java.util.concurrent.Executors
import kotlin.math.roundToInt

/**
 * Light client home. Crypto runs on a background thread because Argon2id needs 64 MiB,
 * and every HTTP call shares one single-thread executor; results return to the UI with
 * `runOnUiThread`, and any failure lands in the status banner. The UI never blocks.
 *
 * The screen now carries the network switch (v0.5.0): a Mainnet / Testnet / Local spinner
 * backed by `NetworkPrefs`, editable RPC + faucet endpoints, and a live chain panel that
 * reads `NodeClient.blockNumber()` / `NodeClient.balance()`. A gap since the last header
 * starts a new checkpoint; the desktop `kovanica-lightnode` follows every header and submits
 * witnesses while the phone sleeps.
 *
 * Laid out programmatically — this app has no `res/` and no Compose.
 */
class MainActivity : Activity() {
    private val io = Executors.newSingleThreadExecutor()

    private lateinit var status: TextView
    private lateinit var networkSpinner: Spinner
    private lateinit var networkNote: TextView
    private lateinit var networkSoon: TextView
    private lateinit var chainState: TextView
    private lateinit var chainMeta: TextView
    private lateinit var rpcField: EditText
    private lateinit var chainField: EditText
    private lateinit var epochField: EditText
    private lateinit var faucetField: EditText
    private lateinit var faucetRow: LinearLayout
    private lateinit var passField: EditText
    private lateinit var mnemonicField: EditText
    private lateinit var toField: EditText
    private lateinit var amountField: EditText
    private lateinit var toValidation: TextView
    private lateinit var amountPreview: TextView
    private lateinit var faucetButton: Button

    // Wallet overview (native KUNA hero).
    private lateinit var overviewValue: TextView
    private lateinit var overviewMeta: TextView
    private lateinit var overviewAddress: TextView
    private lateinit var receiveButton: Button

    // Token watchlist (KVP-102 reads).
    private lateinit var tokensState: TextView
    private lateinit var tokenList: LinearLayout
    private lateinit var addTokenForm: LinearLayout
    private lateinit var addTokenToggle: Button
    private lateinit var addTokenField: EditText
    private lateinit var tokens: TokenStore

    /** Local, per-network log of what this phone submitted (no history RPC exists). */
    private lateinit var history: HistoryStore

    /** Saved recipients, shared across networks (an address is chain-independent). */
    private lateinit var addressBook: AddressBookStore

    /** Contract addresses watched on the active network (lowercase 64-hex). */
    private var watched: List<String> = emptyList()

    /** Latest per-token read result; missing entries render as "not read yet". */
    private var tokenReads: Map<String, TokenRead> = emptyMap()

    /** True while a token read is in flight, so rows show a loading line. */
    private var tokensLoading = false

    /** Contract address whose inline token-send panel is open, if any. */
    private var sendPanelFor: String? = null

    /** Contract address awaiting an inline remove confirmation, if any. */
    private var pendingTokenRemoval: String? = null

    /** Wallet address in `kvnc…dag` form, once a wallet read has resolved it. */
    private var walletAddressDisplay: String? = null

    /** The network the screen is bound to. Read from the IO thread too. */
    @Volatile
    private var current: Network = Networks.DEFAULT

    /** True while we set field text ourselves, so the watchers stay quiet. */
    private var loading = false

    /** True while we move the spinner ourselves, so its callback stays quiet. */
    private var suppressSelection = false

    /** False until the spinner has settled on the restored selection once. */
    private var networkReady = false

    /** `soon` text for a disabled network carried over from an older install. */
    private var staleSoon: String? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        actionBar?.hide()
        window.setBackgroundDrawable(ColorDrawable(INK))
        window.statusBarColor = INK

        current = resolveStartupNetwork()

        val effective = NetworkPrefs.effective(this)
        loading = true
        rpcField = field("RPC endpoint", effective.rpcUrl, mono = true)
        chainField = field("Chain id", effective.chainId.toString(), mono = true)
        epochField = field("Committee epoch", NetworkPrefs.epoch(this).toString(), mono = true)
        faucetField = field("Faucet URL", effective.faucetUrl ?: "", mono = true)
        passField = field("Passphrase", "").apply {
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
        }
        mnemonicField = field("24-word seed (import)", "", mono = true).apply {
            isSingleLine = false
            maxLines = 3
            gravity = Gravity.TOP or Gravity.START
            inputType = InputType.TYPE_CLASS_TEXT or
                InputType.TYPE_TEXT_FLAG_MULTI_LINE or
                InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
        }
        toField = field("Recipient address", "", mono = true)
        amountField = field("Amount (atoms)", "1", mono = true)
        loading = false

        status = TextView(this).apply {
            textSize = 13f
            setLineSpacing(0f, 1.3f)
            setPadding(dp(14), dp(12), dp(14), dp(12))
        }

        val networkCard = card().apply {
            addStack(sectionLabel("Network"))
            networkSpinner = spinner()
            addStack(networkSpinner, 2)
            networkNote = TextView(this@MainActivity).apply {
                textSize = 12f
                typeface = Typeface.DEFAULT_BOLD
                setTextColor(ACCENT)
                setLineSpacing(0f, 1.3f)
            }
            addStack(networkNote, 10)
            networkSoon = TextView(this@MainActivity).apply {
                textSize = 12f
                setTextColor(MUTED)
                setLineSpacing(0f, 1.3f)
            }
            addStack(networkSoon, 4)
            addHairline(14, 14)
            addStack(caption("Endpoints are editable; they default to the chosen network."), 0)
            addStack(rpcField, 10)
            addStack(chainField, 8)
            addStack(epochField, 8)
            faucetRow = LinearLayout(this@MainActivity).apply {
                orientation = LinearLayout.VERTICAL
                addStack(faucetField)
            }
            addStack(faucetRow, 8)
        }

        val chainCard = card().apply {
            addStack(sectionLabel("Chain status"))
            chainState = TextView(this@MainActivity).apply {
                textSize = 16f
                typeface = Typeface.DEFAULT_BOLD
                setLineSpacing(0f, 1.25f)
                setTextColor(MUTED)
            }
            addStack(chainState, 2)
            chainMeta = monoValue("").apply { setTextColor(MUTED) }
            addStack(chainMeta, 8)
            addStack(button("Check connection") { checkConnection() }, 14)
            addStack(button("Sync sealed header", filled = false) { sync() }, 10)
            addStack(button("Verified balance", filled = false) { balance() }, 10)
            addStack(
                caption(
                    "Check connection reads the chain head and your balance from the node you " +
                        "selected. Sync and Verified balance also check the sealed header.",
                ),
                12,
            )
        }

        val walletCard = card().apply {
            addStack(sectionLabel("Create or import"))
            addStack(passField, 2)
            addStack(button("Create wallet") { onCreateWallet() }, 12)
            addStack(caption("Or import an existing wallet from its seed."), 16)
            addStack(mnemonicField, 8)
            addStack(button("Import wallet", filled = false) { onImportWallet() }, 10)
        }

        val sendCard = card().apply {
            addStack(sectionLabel("Send KUNA"))
            addStack(toField, 2)
            val recipientActions = LinearLayout(this@MainActivity).apply {
                orientation = LinearLayout.HORIZONTAL
                addView(
                    button("Contacts", filled = false) { openAddressBook() },
                    LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f),
                )
                addView(
                    button("Scan QR", filled = false) { openScanner() },
                    LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f)
                        .apply { leftMargin = dp(10) },
                )
            }
            addStack(recipientActions, 8)
            toValidation = TextView(this@MainActivity).apply {
                textSize = 12f
                setLineSpacing(0f, 1.3f)
                visibility = View.GONE
            }
            addStack(toValidation, 8)
            addStack(amountField, 12)
            amountPreview = TextView(this@MainActivity).apply {
                textSize = 12f
                typeface = Typeface.MONOSPACE
                setTextColor(MUTED)
                visibility = View.GONE
            }
            addStack(amountPreview, 8)
            addStack(button("Send") { send() }, 12)
            addStack(
                caption(
                    "Amounts are in atoms — 1 KUNA is 1,000,000,000 atoms. Contacts fills the " +
                        "field from your saved list; Scan QR reads an address from a code.",
                ),
                12,
            )
        }

        val lightCard = card().apply {
            addStack(sectionLabel("Light node"))
            addStack(
                caption(
                    "A desktop kovanica-lightnode submits the witness while the phone sleeps. " +
                        "Open the app about once per day.",
                ),
                0,
            )
            addStack(button("Bond 100 KUNA") { bond() }, 14)
            addStack(button("Submit witness", filled = false) { witness() }, 10)
            addStack(button("Exit light node", filled = false) { exitLight() }, 10)
            faucetButton = button("Request faucet") { faucet() }
            addStack(faucetButton, 10)
        }

        val messagesCard = card().apply {
            addStack(sectionLabel("Messages"))
            addStack(caption("Sealed one-to-one messages over your own relay."), 0)
            addStack(button("Open Messages", filled = false) { openMessages() }, 12)
        }

        val overviewCard = walletOverviewCard()
        val tokensCard = tokensCard()

        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(20), dp(24), dp(20), dp(36))
            setBackgroundColor(INK)
            addStack(header())
            addView(rule(), LinearLayout.LayoutParams(dp(40), dp(3)).apply { topMargin = dp(14) })
            addStack(status, 18)
            addStack(guideCard(), 18)
            addStack(overviewCard, 18)
            addStack(tokensCard, 18)
            addStack(sendCard, 18)
            addStack(networkCard, 18)
            addStack(chainCard, 18)
            addStack(walletCard, 18)
            addStack(lightCard, 18)
            addStack(messagesCard, 18)
            addStack(moreCard(), 18)
        }

        setContentView(ScrollView(this).apply {
            isFillViewport = true
            setBackgroundColor(INK)
            addView(root)
        })

        // Watchers attach last, so the first prefill does not write back.
        listOf(rpcField, chainField, epochField, faucetField).forEach {
            it.addTextChangedListener(fieldWatcher())
        }
        toField.addTextChangedListener(recipientWatcher())
        amountField.addTextChangedListener(amountWatcher())
        initNetworkSelection()
        updateNetworkNote()
        refreshFaucet()
        resetChainPanel()
        tokens = TokenStore(this)
        history = HistoryStore(this)
        addressBook = AddressBookStore(this)
        watched = tokens.watched(current.id)
        renderTokens()
        renderRecipientCheck()
        renderAmountPreview()
        renderOverview(
            if (keystoreOrNull() == null) OverviewState.NoWallet else OverviewState.Idle,
        )
        setStatus(startupMessage(), if (staleSoon == null) Tone.NEUTRAL else Tone.WARN)
    }

    override fun onDestroy() {
        io.shutdown()
        super.onDestroy()
    }

    override fun onResume() {
        super.onResume()
        refreshFaucet()
        reloadWatchlist()
    }

    /**
     * Pick up tokens added on another screen (for example Create token, which
     * writes straight to [TokenStore]) and read them. The initial `onCreate`
     * load is not enough on its own, since a screen we return from can change
     * the watched list behind us.
     */
    private fun reloadWatchlist() {
        val latest = tokens.watched(current.id)
        if (latest == watched) return
        watched = latest
        sendPanelFor = null
        pendingTokenRemoval = null
        if (keystoreOrNull() == null) {
            renderTokens()
        } else {
            refreshWallet()
        }
    }

    // --- network switch ------------------------------------------------------------------

    /** Stored network, or the first enabled one when the stored pick is disabled. */
    private fun resolveStartupNetwork(): Network {
        val stored = NetworkPrefs.selected(this)
        if (stored.enabled) return stored
        val fallback = Networks.ALL.firstOrNull { it.enabled } ?: Networks.DEFAULT
        staleSoon = stored.soon ?: "${stored.label} is not open yet."
        NetworkPrefs.select(this, fallback)
        return fallback
    }

    private fun startupMessage(): String {
        val soon = staleSoon
        if (soon != null) {
            return "$soon\nStarted on ${current.label} — chain id ${current.chainId}."
        }
        return "${current.label} — chain id ${current.chainId}.\n" +
            "Check the connection, then create or import a wallet."
    }

    private fun spinner(): Spinner = Spinner(this).apply {
        adapter = NetworkAdapter(this@MainActivity, Networks.ALL)
        background = rounded(SURFACE_2, 12, LINE, 1)
        setPopupBackgroundDrawable(ColorDrawable(SURFACE))
        setPadding(dp(6), dp(2), dp(6), dp(2))
        onItemSelectedListener = object : AdapterView.OnItemSelectedListener {
            override fun onItemSelected(
                parent: AdapterView<*>?,
                view: View?,
                position: Int,
                id: Long,
            ) {
                onNetworkPicked(position)
            }

            override fun onNothingSelected(parent: AdapterView<*>?) {}
        }
    }

    /** Restore the spinner to the stored network, quietly. */
    private fun initNetworkSelection() {
        suppress { networkSpinner.setSelection(indexOf(current)) }
        networkReady = true
    }

    private fun onNetworkPicked(position: Int) {
        if (suppressSelection || position !in Networks.ALL.indices) return
        val picked = Networks.ALL[position]
        if (picked.id == current.id) {
            networkReady = true
            return
        }
        if (!networkReady) {
            return
        }
        if (!picked.enabled) {
            val soon = picked.soon ?: "${picked.label} is not open yet."
            suppress { networkSpinner.setSelection(indexOf(current)) }
            setStatus("$soon\nStaying on ${current.label}.", Tone.WARN)
            return
        }
        applyNetwork(picked)
    }

    private fun applyNetwork(network: Network) {
        NetworkPrefs.select(this, network)
        current = network
        loading = true
        rpcField.setText(network.rpcUrl)
        chainField.setText(network.chainId.toString())
        faucetField.setText(network.faucetUrl ?: "")
        loading = false
        refreshFaucet()
        updateNetworkNote()
        resetChainPanel()
        // Tokens are per-chain: swap the watchlist and drop reads from the old chain.
        watched = tokens.watched(network.id)
        tokenReads = emptyMap()
        tokensLoading = false
        sendPanelFor = null
        pendingTokenRemoval = null
        walletAddressDisplay = null
        renderTokens()
        renderOverview(
            if (keystoreOrNull() == null) OverviewState.NoWallet else OverviewState.Idle,
        )
        setStatus("Switched to ${network.label} — chain id ${network.chainId}.", Tone.OK)
    }

    private fun indexOf(network: Network): Int =
        Networks.ALL.indexOfFirst { it.id == network.id }.coerceAtLeast(0)

    private fun updateNetworkNote() {
        val faucet = if (current.faucetUrl == null) "no faucet here" else "faucet available"
        networkNote.text = "${current.label}  ·  chain id ${current.chainId}  ·  $faucet"
        val locked = Networks.ALL.filter { !it.enabled }
        networkSoon.text = locked.joinToString("\n") { "${it.label}: ${it.soon ?: "not open yet"}" }
        networkSoon.visibility = if (locked.isEmpty()) View.GONE else View.VISIBLE
    }

    private fun suppress(block: () -> Unit) {
        val previous = suppressSelection
        suppressSelection = true
        try {
            block()
        } finally {
            suppressSelection = previous
        }
    }

    // --- actions -------------------------------------------------------------------------

    private fun onCreateWallet() {
        val passphrase = passField.text.toString()
        background(
            label = "Creating wallet — this takes a few seconds…",
            done = { line ->
                pushStatus(line)
                onWalletChanged()
            },
        ) { _ ->
            val created = createWallet(passphrase)
            prefs().edit().putString("keystore", created.keystoreJson).apply()
            ok(
                "Wallet created.\naddress ${created.address}\n" +
                    "seed words (shown once, write them down):\n${created.mnemonic}",
            )
        }
    }

    private fun onImportWallet() {
        val passphrase = passField.text.toString()
        val seed = mnemonicField.text.toString().trim()
        if (seed.isEmpty()) {
            setStatus("Paste the 24-word seed before importing.", Tone.ERROR)
            return
        }
        background(
            label = "Importing wallet…",
            done = { line ->
                pushStatus(line)
                onWalletChanged()
            },
        ) { _ ->
            val imported = importWallet(seed, passphrase)
            prefs().edit().putString("keystore", imported.keystoreJson).apply()
            ok("Wallet imported.\naddress ${imported.address}")
        }
    }

    private fun sync() {
        background("Syncing sealed header…") { env ->
            val node = client(env)
            val header = node.call("kovanica_getLeaderHeader", JSONArray()) as? JSONObject
                ?: return@background warn("No sealed header yet on ${env.network}.")
            val height = parseQuantity(header.get("committedHeight"))
            val seals = node.call("kovanica_getCommitSeals", JSONArray().put(hex(height)))
            val committee = node.call("kovanica_getCommittee", JSONArray())
            val verified = verifyHeader(
                env.chainId,
                env.epoch,
                JSONArray(committee.toString()).toString(),
                header.toString(),
                JSONObject(seals.toString()).toString(),
                parentJson(height),
            )
            prefs().edit()
                .putString("header", header.toString())
                .putString("height", verified.committedHeight.toString())
                .apply()
            ok(
                "Verified height ${verified.committedHeight}\n" +
                    "header ${verified.headerHash}\nepoch ${verified.epoch}",
            )
        }
    }

    private fun balance() {
        background("Reading balance…") { env ->
            val keystore = keystore()
            val address = walletAddress(keystore)
            val proof = client(env).call("kovanica_getAccountProof", JSONArray().put(address))
            if (proof == null) {
                return@background warn("address $address\nbalance unverified (no account proof)")
            }
            val header = prefs().getString("header", null)
                ?: return@background warn("address $address\nbalance unverified (sync a header first)")
            val snapshot = confirmAccount(header, JSONObject(proof.toString()).toString())
            ok("address $address\nbalance ${snapshot.balance} atoms\nnonce ${snapshot.nonce}\nverified")
        }
    }

    /** Live read of head height and balance straight from the selected RPC. */
    private fun checkConnection() {
        background(
            label = "Checking connection…",
            work = { env ->
                val node = client(env)
                val height = node.blockNumber()
                val address = keystoreOrNull()?.let { walletAddress(it) }
                val body = buildString {
                    append("chain id ${env.chainId}  ·  ${env.network}")
                    append('\n')
                    append(env.rpc)
                    if (address == null) {
                        append("\nno wallet yet; create or import one to see its balance")
                    } else {
                        append("\n$address")
                        append("\nbalance ${node.balance(address)} atoms")
                    }
                }
                ok("Connected — height $height\n$body")
            },
            done = { line ->
                pushStatus(line)
                setChainState(line.text.lineSequence().first(), line.tone)
                chainMeta.text = line.text.lineSequence().drop(1).joinToString("\n").trim()
                    .ifEmpty { chainPreview() }
            },
        )
    }

    /**
     * Validate the recipient and amount, read the fee rate from the node, then
     * show a review before anything is signed. The passphrase is captured on the
     * UI thread and only reaches the signer once the user confirms.
     */
    private fun send() {
        val to = toField.text.toString().trim()
        val amount = amountField.text.toString().trim().toULongOrNull()
        val passphrase = passField.text.toString()
        if (to.isEmpty()) {
            setStatus("Enter the recipient address.", Tone.ERROR)
            return
        }
        // Accept bare 64-hex or any `kvnc…dag` wrapper (the app's own
        // `kvnc<64hex>dag` label form and the canonical `kvnc<72hex>dag`); the
        // signer only accepts the 64-hex key. Mirrors `attemptTokenSend`.
        val toHex = KvncAddress.normalizeHex(to)
        if (toHex == null) {
            setStatus("Enter a valid recipient address.", Tone.ERROR)
            return
        }
        if (amount == null) {
            setStatus("Amount must be a whole number of atoms, like 1.", Tone.ERROR)
            return
        }
        if (amount == 0uL) {
            setStatus("Amount must be more than zero.", Tone.ERROR)
            return
        }
        setStatus("Checking the fee…", Tone.NEUTRAL)
        val env = env()
        io.execute {
            val rate = try {
                parseQuantity(client(env).call("kovanica_estimateFee", JSONArray()))
            } catch (error: Exception) {
                runOnUiThread {
                    setStatus(error.message ?: "Couldn't read the fee from the node.", Tone.ERROR)
                }
                return@execute
            }
            runOnUiThread { reviewSend(toHex, amount, rate, passphrase) }
        }
    }

    /** One confirmation between the form and the chain: recipient, amount, fee rate. */
    private fun reviewSend(toHex: String, amount: ULong, rate: ULong, passphrase: String) {
        val kuna = Amounts.format(BigInteger(amount.toString()), NATIVE_DECIMALS)
        val known = addressBook.label(toHex)
        val name = if (known == null) "" else "\nSaved as: $known"
        AlertDialog.Builder(this)
            .setTitle("Send KUNA?")
            .setMessage(
                "To: ${KvncAddress.short(toHex)}$name\n" +
                    "Amount: $kuna KUNA\n" +
                    "Fee rate: $rate atoms/byte",
            )
            .setNegativeButton("Cancel", null)
            .setPositiveButton("Send") { _, _ -> broadcastSend(toHex, amount, rate, passphrase) }
            .show()
    }

    private fun broadcastSend(toHex: String, amount: ULong, rate: ULong, passphrase: String) {
        background("Sending…") { env ->
            val keystore = keystore()
            val address = walletAddress(keystore)
            val node = client(env)
            val nonce = parseQuantity(node.call("kovanica_getNonce", JSONArray().put(address)))
            val raw = signTransferRaw(keystore, passphrase, env.chainId, toHex, amount, nonce, rate)
            val hash = node.call("kovanica_sendRawTransaction", JSONArray().put(raw))
            record(
                kind = HistoryEntry.KIND_SEND,
                hash = hash,
                to = toHex,
                amount = Amounts.format(BigInteger(amount.toString()), NATIVE_DECIMALS),
                symbol = NATIVE_SYMBOL,
            )
            ok("Submitted — $hash")
        }
    }

    private fun bond() {
        val passphrase = passField.text.toString()
        background("Bonding 100 KUNA…") { env ->
            submit(
                env,
                "bond",
                amount = Amounts.format(BigInteger(MIN_LIGHT_BOND.toString()), NATIVE_DECIMALS),
            ) { keystore, _, nonce, rate ->
                signBondRaw(keystore, passphrase, env.chainId, MIN_LIGHT_BOND, nonce, rate)
            }
        }
    }

    private fun witness() {
        val passphrase = passField.text.toString()
        background("Submitting witness…") { env ->
            val keystore = keystore()
            val address = walletAddress(keystore)
            val node = client(env)
            val set = JSONObject(node.call("kovanica_getLightSet", JSONArray()).toString())
            if (set.isNull("beacon")) {
                return@background warn("epoch ${parseQuantity(set.get("epoch"))} has no beacon yet")
            }
            val nonce = parseQuantity(node.call("kovanica_getNonce", JSONArray().put(address)))
            val rate = parseQuantity(node.call("kovanica_estimateFee", JSONArray()))
            val raw = signWitnessRaw(
                keystore,
                passphrase,
                env.chainId,
                parseQuantity(set.get("epoch")),
                set.getString("beacon"),
                nonce,
                rate,
            )
            val hash = node.call("kovanica_sendRawTransaction", JSONArray().put(raw))
            record(kind = HistoryEntry.KIND_WITNESS, hash = hash)
            ok("Witness submitted — $hash\n${lightLine(set, address)}")
        }
    }

    private fun exitLight() {
        val passphrase = passField.text.toString()
        background("Exiting light node…") { env ->
            submit(env, "exit") { keystore, _, nonce, rate ->
                signExitRaw(keystore, passphrase, env.chainId, nonce, rate)
            }
        }
    }

    private fun faucet() {
        background("Requesting faucet…") { env ->
            val address = walletAddress(keystore())
            val result = client(env).postFaucet(address)
            HistoryStore.extractTxHash(result)?.let { hash ->
                record(kind = HistoryEntry.KIND_FAUCET, hash = hash)
            }
            ok("Faucet: $result")
        }
    }

    private fun openMessages() {
        startActivity(Intent(this, MessagesActivity::class.java))
    }

    private fun openGuide() {
        startActivity(Intent(this, HowToUseActivity::class.java))
    }

    private fun openHistory() {
        startActivity(Intent(this, HistoryActivity::class.java))
    }

    private fun openSettings() {
        startActivity(Intent(this, SettingsActivity::class.java))
    }

    /** Open the contact list as a picker; the chosen address comes back below. */
    private fun openAddressBook() {
        startActivityForResult(
            Intent(this, AddressBookActivity::class.java)
                .putExtra(AddressBookActivity.EXTRA_PICK, true),
            REQUEST_PICK_CONTACT,
        )
    }

    private fun openScanner() {
        startActivityForResult(Intent(this, ScanActivity::class.java), REQUEST_SCAN_QR)
    }

    /**
     * Open the KVP-102 token creator. A token is signed with this phone's wallet,
     * so there is nothing to create until one exists — same gate as Receive.
     */
    private fun openCreateToken() {
        if (keystoreOrNull() == null) {
            setStatus("Create or import a wallet first — a token needs one to sign with.", Tone.WARN)
            return
        }
        startActivity(Intent(this, CreateTokenActivity::class.java))
    }

    /**
     * The classic result contract: this app carries no androidx, so the contacts
     * picker and the QR scanner both hand their address back here.
     */
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (resultCode != RESULT_OK) return
        when (requestCode) {
            REQUEST_PICK_CONTACT -> {
                val address = data?.getStringExtra(AddressBookActivity.EXTRA_RESULT_ADDRESS)
                if (!address.isNullOrBlank()) {
                    fillRecipient(address, "Recipient filled from your contacts.")
                }
            }
            REQUEST_SCAN_QR -> {
                val address = data?.getStringExtra(ScanActivity.EXTRA_RESULT_ADDRESS)
                if (!address.isNullOrBlank()) {
                    fillRecipient(address, "Address read from the code.")
                }
            }
        }
    }

    private fun fillRecipient(address: String, message: String) {
        toField.setText(address)
        toField.setSelection(toField.text.length)
        renderRecipientCheck()
        setStatus(message, Tone.OK)
    }

    // --- wallet overview -----------------------------------------------------------------

    /** The hero card: native KUNA balance, the address, and the Receive entry point. */
    private fun walletOverviewCard(): LinearLayout = card().apply {
        addStack(sectionLabel("Wallet"))
        overviewValue = TextView(this@MainActivity).apply {
            textSize = 34f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(TEXT)
            setLineSpacing(0f, 1.05f)
        }
        addStack(overviewValue, 2)
        overviewMeta = TextView(this@MainActivity).apply {
            textSize = 12f
            setTextColor(MUTED)
            setLineSpacing(0f, 1.3f)
        }
        addStack(overviewMeta, 6)
        overviewAddress = TextView(this@MainActivity).apply {
            textSize = 13f
            typeface = Typeface.MONOSPACE
            setTextColor(TEXT)
            setTextIsSelectable(true)
            setLineSpacing(0f, 1.3f)
        }
        addStack(overviewAddress, 12)

        val actions = LinearLayout(this@MainActivity).apply { orientation = LinearLayout.HORIZONTAL }
        receiveButton = button("Receive") { openReceive() }
        actions.addView(
            receiveButton,
            LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f),
        )
        actions.addView(
            button("Refresh", filled = false) { refreshWallet() },
            LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f).apply {
                leftMargin = dp(10)
            },
        )
        addStack(actions, 14)
        addStack(
            caption(
                "Read live from the node you selected. This is native KUNA — KVP-102 tokens " +
                    "live in their own contracts and show under Tokens.",
            ),
            12,
        )
    }

    /** Live read of the native balance, plus every watched token, in one pass. */
    private fun refreshWallet() {
        persistFields()
        val env = env()
        val keystore = keystoreOrNull()
        if (keystore == null) {
            renderOverview(OverviewState.NoWallet)
            setStatus("Create or import a wallet first — there's nothing to read yet.", Tone.WARN)
            return
        }
        val contracts = watched
        renderOverview(OverviewState.Loading)
        tokensLoading = true
        tokenReads = emptyMap()
        renderTokens()
        setStatus("Reading your wallet from ${env.network}…", Tone.NEUTRAL)
        io.execute {
            val snapshot = readWallet(env, keystore, contracts)
            runOnUiThread {
                walletAddressDisplay = snapshot.address
                tokenReads = snapshot.reads
                tokensLoading = false
                renderOverview(snapshot.overview)
                renderTokens()
                val (line, tone) = overviewLine(snapshot.overview)
                setStatus(line, tone)
            }
        }
    }

    /** Background half of [refreshWallet]; never throws. */
    private fun readWallet(env: Env, keystore: String, contracts: List<String>): WalletSnapshot {
        val display = try {
            walletAddress(keystore)
        } catch (error: Exception) {
            return WalletSnapshot(
                OverviewState.Failed(error.message ?: "the keystore couldn't be read"),
                emptyMap(),
                null,
            )
        }
        val node = client(env)
        val height = try {
            node.blockNumber()
        } catch (_: Exception) {
            null
        }
        var atoms: ULong? = null
        var failure: String? = null
        try {
            atoms = node.balance(display)
        } catch (error: Exception) {
            failure = error.message ?: "the node didn't answer"
        }
        val tokenClient = TokenClient(node)
        val reads = LinkedHashMap<String, TokenRead>(contracts.size)
        for (contract in contracts) {
            reads[contract] = tokenClient.read(contract)
        }
        val overview = when {
            failure != null -> OverviewState.Failed(failure!!)
            atoms != null -> OverviewState.Ready(atoms!!, height)
            else -> OverviewState.Failed("the node didn't answer")
        }
        return WalletSnapshot(overview, reads, display)
    }

    private fun overviewLine(state: OverviewState): Pair<String, Tone> = when (state) {
        is OverviewState.Ready ->
            "Balance read from ${current.label} — ${state.atoms} atoms." to Tone.OK
        is OverviewState.Failed -> "Couldn't read the wallet: ${state.message}" to Tone.ERROR
        OverviewState.NoWallet -> "No wallet on this phone yet." to Tone.WARN
        OverviewState.Loading -> "Reading…" to Tone.NEUTRAL
        OverviewState.Idle -> "Tap Refresh to read your balance." to Tone.NEUTRAL
    }

    private fun renderOverview(state: OverviewState) {
        when (state) {
            OverviewState.NoWallet -> {
                hero("—", MUTED)
                overviewMeta.text =
                    "No wallet on this phone yet. Create or import one below to see a balance."
                overviewMeta.setTextColor(MUTED)
                showAddress(false)
                setReceiveEnabled(false)
            }
            OverviewState.Idle -> {
                hero("—", TEXT)
                overviewMeta.text = "Tap Refresh to read your balance from ${current.label}."
                overviewMeta.setTextColor(MUTED)
                showAddress(walletAddressDisplay != null)
                setReceiveEnabled(walletAddressDisplay != null)
            }
            OverviewState.Loading -> {
                hero("…", TEXT)
                overviewMeta.text = "Reading your balance from ${current.label}…"
                overviewMeta.setTextColor(MUTED)
                showAddress(walletAddressDisplay != null)
                setReceiveEnabled(walletAddressDisplay != null)
            }
            is OverviewState.Ready -> {
                hero("${Amounts.format(BigInteger(state.atoms.toString()), NATIVE_DECIMALS)} KUNA", TEXT)
                val heightLine = state.height?.let { "height $it" } ?: "height unknown"
                overviewMeta.text = "${state.atoms} atoms  ·  $heightLine  ·  ${current.label}"
                overviewMeta.setTextColor(MUTED)
                showAddress(true)
                setReceiveEnabled(true)
            }
            is OverviewState.Failed -> {
                hero("—", ACCENT)
                overviewMeta.text = state.message
                overviewMeta.setTextColor(ACCENT)
                showAddress(walletAddressDisplay != null)
                setReceiveEnabled(walletAddressDisplay != null)
            }
        }
    }

    private fun hero(text: String, color: Int) {
        overviewValue.text = text
        overviewValue.setTextColor(color)
    }

    private fun showAddress(visible: Boolean) {
        overviewAddress.visibility = if (visible) View.VISIBLE else View.GONE
        overviewAddress.text = walletAddressDisplay.orEmpty()
        overviewAddress.contentDescription = walletAddressDisplay.orEmpty()
    }

    private fun setReceiveEnabled(enabled: Boolean) {
        receiveButton.isEnabled = enabled
        receiveButton.alpha = if (enabled) 1f else 0.5f
    }

    /** Re-read the wallet address from the local store after create/import. */
    private fun onWalletChanged() {
        val keystore = keystoreOrNull()
        walletAddressDisplay = if (keystore == null) {
            null
        } else {
            try {
                walletAddress(keystore)
            } catch (_: Exception) {
                null
            }
        }
        renderOverview(if (keystore == null) OverviewState.NoWallet else OverviewState.Idle)
        renderTokens()
    }

    private fun openReceive() {
        val address = walletAddressDisplay
        if (address == null) {
            setStatus("Create or import a wallet first — there's no address to receive at yet.", Tone.WARN)
            return
        }
        startActivity(
            Intent(this, ReceiveActivity::class.java)
                .putExtra(ReceiveActivity.EXTRA_ADDRESS, address)
                .putExtra(ReceiveActivity.EXTRA_NETWORK, current.label),
        )
    }

    // --- tokens --------------------------------------------------------------------------

    /** The token watchlist card: add form plus one row per watched contract. */
    private fun tokensCard(): LinearLayout = card().apply {
        addStack(tokensHeader())
        addStack(button("Create token", filled = false) { openCreateToken() }, 10)
        tokensState = TextView(this@MainActivity).apply {
            textSize = 12f
            setTextColor(MUTED)
            setLineSpacing(0f, 1.3f)
        }
        addStack(tokensState, 4)
        tokenList = LinearLayout(this@MainActivity).apply { orientation = LinearLayout.VERTICAL }
        addStack(tokenList, 12)
        addTokenForm = addTokenFormView()
        addStack(addTokenForm, 14)
    }

    private fun tokensHeader(): LinearLayout = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        addView(
            TextView(this@MainActivity).apply {
                text = "TOKENS"
                setTextColor(MUTED)
                textSize = 11f
                letterSpacing = 0.18f
                typeface = Typeface.DEFAULT_BOLD
            },
            LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f),
        )
        addTokenToggle = button("Add", filled = false) { toggleAddToken() }
        addView(
            addTokenToggle,
            LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT,
                LinearLayout.LayoutParams.WRAP_CONTENT,
            ),
        )
    }

    private fun addTokenFormView(): LinearLayout = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        visibility = View.GONE
        addStack(caption("Watch a KVP-102 token by its contract address."))
        addTokenField = field("Contract address (kvnc…dag or 64 hex)", "", mono = true)
        addStack(addTokenField, 8)
        addStack(button("Add token") { addToken() }, 12)
        addStack(
            caption(
                "Balances are read straight from the contract's own storage. Token amounts are " +
                    "separate from your KUNA balance and never count towards it.",
            ),
            12,
        )
    }

    private fun toggleAddToken() {
        setAddTokenVisible(addTokenForm.visibility != View.VISIBLE)
    }

    private fun setAddTokenVisible(visible: Boolean) {
        addTokenForm.visibility = if (visible) View.VISIBLE else View.GONE
        addTokenToggle.text = if (visible) "Done" else "Add"
    }

    private fun addToken() {
        val hex = KvncAddress.normalizeHex(addTokenField.text.toString())
        if (hex == null) {
            setStatus(
                "That isn't a valid contract address — paste a kvnc…dag address or 64 hex characters.",
                Tone.ERROR,
            )
            return
        }
        if (watched.contains(hex)) {
            setStatus("That token is already on your list.", Tone.WARN)
            return
        }
        tokens.add(current.id, hex)
        addTokenField.setText("")
        setAddTokenVisible(false)
        sendPanelFor = null
        pendingTokenRemoval = null
        watched = tokens.watched(current.id)
        renderTokens()
        if (keystoreOrNull() == null) {
            setStatus("Added ${KvncAddress.short(hex)}. Create or import a wallet to see your balance.", Tone.NEUTRAL)
        } else {
            setStatus("Added ${KvncAddress.short(hex)}. Reading it now…", Tone.NEUTRAL)
            refreshWallet()
        }
    }

    private fun removeToken(contract: String) {
        tokens.remove(current.id, contract)
        watched = tokens.watched(current.id)
        tokenReads = tokenReads - contract
        pendingTokenRemoval = null
        if (sendPanelFor == contract) sendPanelFor = null
        renderTokens()
        setStatus("Stopped watching ${KvncAddress.short(contract)}.", Tone.NEUTRAL)
    }

    private fun renderTokens() {
        tokenList.removeAllViews()
        tokensState.text = when {
            watched.isEmpty() ->
                "No tokens watched on ${current.label}. Add one by address, or create one."
            watched.size == 1 -> "1 token watched on ${current.label}."
            else -> "${watched.size} tokens watched on ${current.label}."
        }
        tokensState.setTextColor(MUTED)
        if (watched.isEmpty()) {
            tokenList.addStack(
                caption(
                    "Tokens are held by their own contracts, not in your KUNA balance. " +
                        "Tap Add to watch one by address, or Create token to issue a new one.",
                ),
                0,
            )
            return
        }
        watched.forEachIndexed { index, contract ->
            tokenList.addStack(tokenRow(contract), if (index == 0) 0 else 10)
        }
    }

    private fun tokenRow(contract: String): LinearLayout {
        if (contract == pendingTokenRemoval) return tokenRemovalRow(contract)
        val read = tokenReads[contract]
        val info = (read as? TokenRead.Ok)?.info

        val row = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            background = rounded(SURFACE_2, 14, LINE, 1)
            setPadding(dp(14), dp(12), dp(14), dp(12))
            isClickable = true
            isFocusable = true
            setOnClickListener { toggleSendPanel(contract) }
            setOnLongClickListener {
                pendingTokenRemoval = contract
                sendPanelFor = null
                renderTokens()
                true
            }
        }

        val top = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }
        top.addView(
            TextView(this).apply {
                text = when {
                    info != null -> info.symbol
                    read is TokenRead.NotAToken -> "Not a token"
                    read is TokenRead.Unreadable -> "Unreadable"
                    read is TokenRead.Failed -> "Unavailable"
                    else -> "…"
                }
                setTextColor(if (info != null) TEXT else ACCENT)
                textSize = 16f
                typeface = Typeface.DEFAULT_BOLD
            },
            LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f),
        )
        if (info != null && info.paused) top.addView(badge("PAUSED", ACCENT, ACCENT_BG))
        row.addView(top)

        if (info != null) {
            if (info.name.isNotBlank() && info.name != info.symbol) {
                row.addView(TextView(this).apply {
                    text = info.name
                    setTextColor(MUTED)
                    textSize = 13f
                    setPadding(0, dp(2), 0, 0)
                })
            }
            if (activeViewerHex() != null) {
                row.addView(TextView(this).apply {
                    text = balanceLine(info)
                    setTextColor(TEXT)
                    textSize = 22f
                    typeface = Typeface.DEFAULT_BOLD
                    setPadding(0, dp(8), 0, 0)
                })
            } else {
                row.addView(TextView(this).apply {
                    text = "Create or import a wallet to see your balance of this token."
                    setTextColor(ACCENT)
                    textSize = 13f
                    setLineSpacing(0f, 1.3f)
                    setPadding(0, dp(8), 0, 0)
                })
            }
            row.addView(TextView(this).apply {
                text = detailLine(info)
                setTextColor(MUTED)
                textSize = 12f
                setLineSpacing(0f, 1.3f)
                setPadding(0, dp(6), 0, 0)
            })
        } else {
            row.addView(TextView(this).apply {
                text = when {
                    tokensLoading -> "Reading…"
                    read == null ->
                        "Not read yet on this phone — tap Refresh to load this token's state."
                    read is TokenRead.NotAToken ->
                        "Nothing is deployed at this address on ${current.label}, or it holds no token state."
                    read is TokenRead.Unreadable ->
                        "This address holds something that isn't a KVP-102 token."
                    read is TokenRead.Failed -> "Couldn't read it — ${read.message}"
                    else -> ""
                }
                setTextColor(MUTED)
                textSize = 13f
                setLineSpacing(0f, 1.3f)
                setPadding(0, dp(6), 0, 0)
            })
        }

        row.addView(TextView(this).apply {
            text = KvncAddress.short(contract)
            contentDescription = contract
            setTextColor(MUTED)
            typeface = Typeface.MONOSPACE
            textSize = 12f
            setPadding(0, dp(10), 0, 0)
        })

        val actions = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(0, dp(10), 0, 0)
        }
        actions.addView(
            button(
                if (sendPanelFor == contract) "Close" else "Send",
                filled = false,
            ) { toggleSendPanel(contract) },
            LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT,
                LinearLayout.LayoutParams.WRAP_CONTENT,
            ),
        )
        actions.addView(
            button("Stop watching", filled = false) {
                pendingTokenRemoval = contract
                sendPanelFor = null
                renderTokens()
            }.apply { setTextColor(BAD) },
            LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT,
                LinearLayout.LayoutParams.WRAP_CONTENT,
            ).apply { leftMargin = dp(10) },
        )
        row.addView(actions)

        if (sendPanelFor == contract) row.addView(sendPanel(contract, info))

        return row
    }

    /** This phone's wallet as canonical lowercase hex, or null when there's no wallet. */
    private fun activeViewerHex(): String? =
        walletAddressDisplay?.let { KvncAddress.normalizeHex(it) }

    private fun balanceLine(info: TokenInfo): String {
        val viewer = activeViewerHex() ?: return "—"
        return "${Amounts.format(info.balanceOf(viewer), info.decimals)} ${info.symbol}"
    }

    private fun detailLine(info: TokenInfo): String {
        val supply = Amounts.format(info.totalSupply, info.decimals)
        return "${info.decimals} decimals  ·  supply $supply ${info.symbol}  ·  ${info.holders} holder" +
            (if (info.holders == 1) "" else "s")
    }

    private fun toggleSendPanel(contract: String) {
        sendPanelFor = if (sendPanelFor == contract) null else contract
        renderTokens()
    }

    /**
     * Inline token-send form. Send calls [TokenClient.sendTransfer], which
     * encodes the `token_transfer` calldata, signs it through the light FFI's
     * `signCallRaw` and submits it; the tx hash (or a clear error) lands in the
     * status banner. This moves the token's own balances — never native KUNA.
     */
    private fun sendPanel(contract: String, info: TokenInfo?): LinearLayout =
        LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            background = rounded(SURFACE, 14, LINE, 1)
            setPadding(dp(14), dp(12), dp(14), dp(12))
            val symbol = info?.symbol ?: "token"
            addStack(
                caption(
                    "Sends $symbol held by this contract — this is a token transfer, not a KUNA payment.",
                ),
            )
            val to = field("Recipient address", "", mono = true)
            addStack(to, 10)
            val amount = field("Amount (base units)", "1", mono = true)
            addStack(amount, 8)
            addStack(
                button("Send $symbol") {
                    attemptTokenSend(contract, to.text.toString(), amount.text.toString(), info)
                },
                12,
            )
            isClickable = true // swallow taps on the panel itself
        }

    private fun attemptTokenSend(contract: String, to: String, amountText: String, info: TokenInfo?) {
        val toHex = KvncAddress.normalizeHex(to)
        if (toHex == null) {
            setStatus("Enter a valid recipient address.", Tone.ERROR)
            return
        }
        val amount = amountText.trim().toBigIntegerOrNull()
        if (amount == null || amount.signum() <= 0) {
            setStatus("Enter an amount in the token's base units, like 1.", Tone.ERROR)
            return
        }
        val keystore = keystoreOrNull()
        if (keystore == null) {
            setStatus("Create or import a wallet first.", Tone.ERROR)
            return
        }
        // Snapshot the UI-only inputs before hopping to the IO thread.
        val passphrase = passField.text.toString()
        val decimals = info?.decimals ?: 0
        val symbol = info?.symbol ?: "token"
        val env = env()
        background("Preparing token transfer…") { _ ->
            val hash = TokenClient(client(env)).sendTransfer(
                keystore,
                passphrase,
                env.chainId,
                contract,
                toHex,
                amount,
                decimals,
            )
            record(
                kind = HistoryEntry.KIND_TOKEN_SEND,
                hash = hash,
                to = toHex,
                contract = contract,
                amount = Amounts.format(amount, decimals),
                symbol = symbol,
            )
            ok("$symbol transfer submitted — $hash")
        }
    }

    private fun tokenRemovalRow(contract: String): LinearLayout = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        background = rounded(SURFACE_2, 14, BAD, 1)
        setPadding(dp(14), dp(12), dp(14), dp(12))
        isClickable = true // only the two buttons act
        addView(TextView(this@MainActivity).apply {
            text = "Stop watching ${KvncAddress.short(contract)}?\nThis only clears it from this phone."
            setTextColor(TEXT)
            textSize = 14f
            setLineSpacing(0f, 1.3f)
        })
        val actions = LinearLayout(this@MainActivity).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(0, dp(10), 0, 0)
        }
        actions.addView(
            button("Stop watching", filled = false) { removeToken(contract) }
                .apply { setTextColor(BAD) },
            LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f),
        )
        actions.addView(
            button("Keep", filled = false) {
                pendingTokenRemoval = null
                renderTokens()
            },
            LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f).apply {
                leftMargin = dp(10)
            },
        )
        addView(actions)
    }

    private fun badge(text: String, foreground: Int, background: Int): TextView = TextView(this).apply {
        this.text = text
        setTextColor(foreground)
        textSize = 10f
        letterSpacing = 0.14f
        typeface = Typeface.DEFAULT_BOLD
        setPadding(dp(8), dp(3), dp(8), dp(3))
        this.background = rounded(background, 8)
    }

    /** Shared nonce/rate/sign/submit path for bond and exit. */
    private fun submit(
        env: Env,
        label: String,
        amount: String? = null,
        sign: (keystore: String, address: String, nonce: ULong, rate: ULong) -> String,
    ): Line {
        val keystore = keystore()
        val address = walletAddress(keystore)
        val node = client(env)
        val nonce = parseQuantity(node.call("kovanica_getNonce", JSONArray().put(address)))
        val rate = parseQuantity(node.call("kovanica_estimateFee", JSONArray()))
        val raw = sign(keystore, address, nonce, rate)
        val hash = node.call("kovanica_sendRawTransaction", JSONArray().put(raw))
        record(
            kind = label,
            hash = hash,
            amount = amount,
            symbol = if (amount == null) null else NATIVE_SYMBOL,
        )
        return ok("$label submitted — $hash")
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
                return "active ${node.optBoolean("active")} witnessed ${node.optBoolean("witnessed")} " +
                    "selected ${node.optBoolean("selected")}"
            }
        }
        return "bond: not registered"
    }

    // --- io ------------------------------------------------------------------------------

    /**
     * Snapshot the visible config on the UI thread, run [work] on the single IO thread, and
     * hand its [Line] back on the UI thread. Any exception becomes an ERROR line, so the UI
     * thread never sees it.
     */
    private fun background(
        label: String,
        done: (Line) -> Unit = ::pushStatus,
        work: (Env) -> Line,
    ) {
        persistFields()
        val env = env()
        pushStatus(Line(label, Tone.NEUTRAL))
        refreshFaucet()
        io.execute {
            val line = try {
                work(env)
            } catch (err: Exception) {
                Line(err.message ?: err.toString(), Tone.ERROR)
            }
            runOnUiThread { done(line) }
        }
    }

    /** One immutable snapshot of everything an IO task needs from the form. */
    private data class Env(
        val rpc: String,
        val faucet: String?,
        val chainId: ULong,
        val epoch: ULong,
        val network: String,
    )

    private fun env(): Env = Env(
        rpc = rpcField.text.toString().trim(),
        faucet = faucetField.text.toString().trim().ifEmpty { null },
        chainId = chainIdField(),
        epoch = epochField.text.toString().trim().toULongOrNull() ?: 0uL,
        network = current.label,
    )

    private fun client(env: Env) = NodeClient(env.rpc, env.faucet)

    /** Persist the effective overrides through `NetworkPrefs`, never hand-rolled prefs. */
    private fun persistFields() {
        if (loading) return
        NetworkPrefs.setRpc(this, rpcField.text.toString().trim())
        NetworkPrefs.setFaucet(this, faucetField.text.toString().trim().ifEmpty { null })
        chainField.text.toString().trim().toLongOrNull()?.let { NetworkPrefs.setChainId(this, it) }
        epochField.text.toString().trim().toLongOrNull()?.let { NetworkPrefs.setEpoch(this, it) }
    }

    private fun fieldWatcher(): TextWatcher = object : TextWatcher {
        override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(s: Editable?) {
            if (loading) return
            persistFields()
            refreshFaucet()
            chainMeta.text = chainPreview()
        }
    }

    /** Live check on the recipient: a clear yes or no as the address is typed or filled. */
    private fun recipientWatcher(): TextWatcher = object : TextWatcher {
        override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(s: Editable?) {
            renderRecipientCheck()
        }
    }

    /** Shows what the entered atoms add up to in KUNA, so the field means something readable. */
    private fun amountWatcher(): TextWatcher = object : TextWatcher {
        override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(s: Editable?) {
            renderAmountPreview()
        }
    }

    private fun renderRecipientCheck() {
        val raw = toField.text.toString().trim()
        when {
            raw.isEmpty() -> toValidation.visibility = View.GONE
            KvncAddress.isValid(raw) -> {
                toValidation.text = "Looks like a valid address — ${KvncAddress.short(raw)}."
                toValidation.setTextColor(GOOD)
                toValidation.visibility = View.VISIBLE
            }
            else -> {
                toValidation.text = "That is not a Kovanica address yet."
                toValidation.setTextColor(BAD)
                toValidation.visibility = View.VISIBLE
            }
        }
    }

    private fun renderAmountPreview() {
        val atoms = amountField.text.toString().trim().toULongOrNull()
        if (atoms == null) {
            amountPreview.visibility = View.GONE
            return
        }
        amountPreview.text = "= ${Amounts.format(BigInteger(atoms.toString()), NATIVE_DECIMALS)} KUNA"
        amountPreview.visibility = View.VISIBLE
    }

    // --- rendering -----------------------------------------------------------------------

    private enum class Tone { NEUTRAL, OK, WARN, ERROR }

    private data class Line(val text: String, val tone: Tone)

    /**
     * Native-balance panel state. `Idle` means a wallet exists but nothing has
     * been read yet, so the screen never implies a balance it hasn't seen.
     */
    private sealed interface OverviewState {
        data object NoWallet : OverviewState
        data object Idle : OverviewState
        data object Loading : OverviewState
        data class Ready(val atoms: ULong, val height: ULong?) : OverviewState
        data class Failed(val message: String) : OverviewState
    }

    /** Everything one wallet read produces, handed back to the UI thread as one unit. */
    private data class WalletSnapshot(
        val overview: OverviewState,
        val reads: Map<String, TokenRead>,
        val address: String?,
    )

    private fun ok(text: String) = Line(text, Tone.OK)

    private fun warn(text: String) = Line(text, Tone.WARN)

    private fun pushStatus(line: Line) = setStatus(line.text, line.tone)

    private fun setStatus(text: String, tone: Tone) {
        status.text = text
        val (foreground, background) = when (tone) {
            Tone.NEUTRAL -> MUTED to SURFACE
            Tone.OK -> GOOD to GOOD_BG
            Tone.WARN -> ACCENT to ACCENT_BG
            Tone.ERROR -> BAD to BAD_BG
        }
        status.setTextColor(foreground)
        status.background = rounded(background, 14, LINE, 1)
    }

    private fun setChainState(text: String, tone: Tone) {
        chainState.text = text
        chainState.setTextColor(
            when (tone) {
                Tone.NEUTRAL -> MUTED
                Tone.OK -> GOOD
                Tone.WARN -> ACCENT
                Tone.ERROR -> BAD
            },
        )
    }

    private fun resetChainPanel() {
        setChainState("Not checked yet.", Tone.NEUTRAL)
        chainMeta.text = chainPreview()
    }

    private fun chainPreview(): String =
        "chain id ${chainField.text.toString().trim()}  ·  ${current.label}\n" +
            rpcField.text.toString().trim()

    private fun refreshFaucet() {
        val configured = current.faucetUrl != null && faucetField.text.toString().trim().isNotEmpty()
        val allowed = configured && try {
            faucetAllowed(chainIdField())
        } catch (_: Exception) {
            false
        }
        faucetButton.visibility = if (allowed) View.VISIBLE else View.GONE
        faucetField.isEnabled = current.faucetUrl != null
        faucetRow.visibility = if (current.faucetUrl == null) View.GONE else View.VISIBLE
    }

    private fun chainIdField(): ULong = chainField.text.toString().trim().toULongOrNull() ?: 0uL

    // --- widgets -------------------------------------------------------------------------

    private fun header(): LinearLayout = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        addView(TextView(this@MainActivity).apply {
            text = "Kovanica · LIGHT"
            setTextColor(ACCENT)
            textSize = 11f
            letterSpacing = 0.22f
            typeface = Typeface.DEFAULT_BOLD
        })
        addView(TextView(this@MainActivity).apply {
            text = "Kovanica Lightnode"
            setTextColor(TEXT)
            textSize = 30f
            typeface = Typeface.DEFAULT_BOLD
            setPadding(0, dp(6), 0, 0)
        })
        addView(TextView(this@MainActivity).apply {
            text = "Pick a network, check the chain, and move KUNA. Your keys stay on this phone."
            setTextColor(MUTED)
            textSize = 13f
            setLineSpacing(0f, 1.3f)
            setPadding(0, dp(8), 0, 0)
        })
    }

    private fun rule(): View = View(this).apply { background = rounded(ACCENT, 2) }

    /**
     * First-run pointer into the guide screen. The whole card is tappable, with the
     * amber press tint used by the tappable rows on Messages.
     */
    private fun guideCard(): LinearLayout = card().apply {
        isClickable = true
        isFocusable = true
        background = ripple(SURFACE, 18)
        setOnClickListener { openGuide() }

        val head = LinearLayout(this@MainActivity).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }
        head.addView(
            TextView(this@MainActivity).apply {
                text = "BEFORE YOU START"
                setTextColor(MUTED)
                textSize = 11f
                letterSpacing = 0.18f
                typeface = Typeface.DEFAULT_BOLD
            },
            LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f),
        )
        head.addView(
            TextView(this@MainActivity).apply {
                text = "›"
                setTextColor(ACCENT)
                textSize = 24f
                typeface = Typeface.DEFAULT_BOLD
            },
        )
        addStack(head, 0)
        addStack(
            TextView(this@MainActivity).apply {
                text = "How to use this app"
                setTextColor(TEXT)
                textSize = 18f
                typeface = Typeface.DEFAULT_BOLD
                setPadding(0, dp(8), 0, 0)
            },
            0,
        )
        addStack(
            caption(
                "A short walkthrough of every screen: network, wallet, receive, send, " +
                    "tokens, and messages. Your keys stay on this phone.",
            ),
            8,
        )
    }

    /**
     * Entry into Settings and the local transaction history. Two plain buttons so
     * "what this phone sent" and "how to protect it" sit together, away from the
     * day-to-day send and receive flow.
     */
    private fun moreCard(): LinearLayout = card().apply {
        addStack(sectionLabel("More"))
        addStack(
            caption(
                "History is a local record kept on this phone. Settings covers your seed " +
                    "backup and the passphrase that protects it.",
            ),
            0,
        )
        addStack(button("Transaction history", filled = false) { openHistory() }, 12)
        addStack(button("Settings", filled = false) { openSettings() }, 10)
    }

    private fun card(): LinearLayout = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        background = rounded(SURFACE, 18, LINE, 1)
        setPadding(dp(16), dp(16), dp(16), dp(16))
    }

    private fun sectionLabel(caption: String): TextView = TextView(this).apply {
        text = caption.uppercase()
        setTextColor(MUTED)
        textSize = 11f
        letterSpacing = 0.18f
        typeface = Typeface.DEFAULT_BOLD
        setPadding(0, 0, 0, dp(10))
    }

    private fun caption(text: String): TextView = TextView(this).apply {
        this.text = text
        setTextColor(MUTED)
        textSize = 12f
        setLineSpacing(0f, 1.3f)
    }

    private fun monoValue(text: String): TextView = TextView(this).apply {
        this.text = text
        setTextColor(TEXT)
        typeface = Typeface.MONOSPACE
        textSize = 12f
        setTextIsSelectable(true)
        setLineSpacing(0f, 1.25f)
    }

    private fun field(hint: String, value: String = "", mono: Boolean = false): EditText =
        EditText(this).apply {
            this.hint = hint
            setText(value)
            setTextColor(TEXT)
            setHintTextColor(MUTED)
            textSize = 15f
            if (mono) typeface = Typeface.MONOSPACE
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
            background = rounded(SURFACE_2, 12, LINE, 1)
            setPadding(dp(14), dp(13), dp(14), dp(13))
            maxLines = 1
        }

    private fun button(caption: String, filled: Boolean = true, action: () -> Unit): Button =
        Button(this).apply {
            text = caption
            isAllCaps = false
            textSize = 15f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(if (filled) ACCENT_INK else ACCENT)
            background = if (filled) rounded(ACCENT, 12) else rounded(SURFACE_2, 12, LINE, 1)
            setPadding(dp(18), dp(12), dp(18), dp(12))
            minHeight = dp(48)
            stateListAnimator = null
            setOnClickListener { action() }
        }

    private fun rounded(fill: Int, radiusDp: Int, stroke: Int? = null, strokeDp: Int = 1): GradientDrawable =
        GradientDrawable().apply {
            setColor(fill)
            cornerRadius = dp(radiusDp).toFloat()
            if (stroke != null) setStroke(dp(strokeDp), stroke)
        }

    /** Amber press tint, matching the tappable rows on the Messages screen. */
    private fun ripple(fill: Int, radiusDp: Int): Drawable = RippleDrawable(
        ColorStateList.valueOf(RIPPLE),
        rounded(fill, radiusDp, LINE, 1),
        null,
    )

    private fun ViewGroup.addStack(child: View, top: Int = 0) {
        addView(
            child,
            LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT,
            ).apply { topMargin = dp(top) },
        )
    }

    /** A one-pixel separator with breathing room above and below. */
    private fun ViewGroup.addHairline(top: Int, bottom: Int) {
        addView(
            View(this@MainActivity).apply { background = rounded(LINE, 1) },
            LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, dp(1)).apply {
                topMargin = dp(top)
                bottomMargin = dp(bottom)
            },
        )
    }

    private fun dp(value: Int): Int = (value * resources.displayMetrics.density).roundToInt()

    // --- prefs ---------------------------------------------------------------------------

    private fun prefs() = getSharedPreferences(CommsStore.PREFS, MODE_PRIVATE)

    private fun keystoreOrNull(): String? = prefs().getString("keystore", null)

    private fun keystore(): String = keystoreOrNull() ?: error("Create or import a wallet first.")

    /**
     * Append one row to the local history for the active network. Called from the
     * IO thread at each submission point; a blank hash is ignored so a failed send
     * never leaves a mystery row behind.
     */
    private fun record(
        kind: String,
        hash: Any?,
        to: String? = null,
        contract: String? = null,
        amount: String? = null,
        symbol: String? = null,
    ) {
        val value = hash?.toString()?.trim().orEmpty()
        if (value.isEmpty()) return
        history.add(
            current.id,
            HistoryEntry(
                kind = kind,
                hash = value,
                networkId = current.id,
                timestampMs = System.currentTimeMillis(),
                to = to,
                contract = contract,
                amount = amount,
                symbol = symbol,
            ),
        )
    }

    private fun hex(value: ULong) = "0x" + value.toString(16)

    /**
     * Network rows for the spinner. Disabled profiles stay visible, greyed, with their
     * `soon` note; reporting `isEnabled == false` makes the popup refuse the tap, and
     * [onNetworkPicked] double-checks.
     */
    private inner class NetworkAdapter(
        context: Context,
        private val entries: List<Network>,
    ) : ArrayAdapter<Network>(context, android.R.layout.simple_spinner_item, entries) {
        init {
            setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item)
        }

        override fun isEnabled(position: Int): Boolean = entries[position].enabled

        override fun getView(position: Int, convertView: View?, parent: ViewGroup): View {
            val view = super.getView(position, convertView, parent) as TextView
            styleNetworkItem(view, entries[position], dropdown = false)
            return view
        }

        override fun getDropDownView(position: Int, convertView: View?, parent: ViewGroup): View {
            val view = super.getDropDownView(position, convertView, parent) as TextView
            styleNetworkItem(view, entries[position], dropdown = true)
            return view
        }
    }

    private fun styleNetworkItem(view: TextView, network: Network, dropdown: Boolean) {
        view.text = if (network.enabled) {
            "${network.label} · chain ${network.chainId}"
        } else {
            "${network.label} — ${network.soon ?: "not open yet"}"
        }
        view.setTextColor(if (network.enabled) TEXT else MUTED)
        view.textSize = 15f
        view.typeface = if (network.enabled) Typeface.DEFAULT_BOLD else Typeface.DEFAULT
        view.setPadding(dp(14), dp(12), dp(14), dp(12))
        view.background = if (dropdown) {
            if (network.enabled) ripple(SURFACE, 10) else rounded(SURFACE_2, 10)
        } else {
            ColorDrawable(Color.TRANSPARENT)
        }
    }

    companion object {
        /** Matches `MIN_LIGHT_BOND` (100 KUNA, 9 decimals). */
        private val MIN_LIGHT_BOND: ULong = 100_000_000_000u

        /** KUNA has 9 decimals; token decimals come from each token's own state. */
        private const val NATIVE_DECIMALS: Int = 9

        /** Native ticker shown next to amounts in the local history. */
        private const val NATIVE_SYMBOL: String = "KUNA"

        // Palette lives in WalletUi so new surfaces cannot drift from the shipped theme.
        private val INK = WalletUi.INK
        private val SURFACE = WalletUi.SURFACE
        private val SURFACE_2 = WalletUi.SURFACE_2
        private val LINE = WalletUi.LINE
        private val TEXT = WalletUi.TEXT
        private val MUTED = WalletUi.MUTED
        private val ACCENT = WalletUi.ACCENT
        private val ACCENT_INK = WalletUi.ACCENT_INK
        private val ACCENT_BG = WalletUi.ACCENT_BG
        private val GOOD = WalletUi.GOOD
        private val GOOD_BG = WalletUi.GOOD_BG
        private val BAD = WalletUi.BAD
        private val BAD_BG = WalletUi.BAD_BG

        /** Press tint for the network rows: the accent at about 18% alpha. */
        private val RIPPLE = WalletUi.RIPPLE

        /** Request codes for the send-card pickers, via the classic result contract. */
        private const val REQUEST_PICK_CONTACT: Int = 1001
        private const val REQUEST_SCAN_QR: Int = 1002
    }
}
