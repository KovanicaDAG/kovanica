package io.kovanica.light

import android.app.Activity
import android.graphics.Typeface
import android.graphics.drawable.ColorDrawable
import android.graphics.drawable.GradientDrawable
import android.os.Bundle
import android.text.InputType
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.concurrent.Executors
import kotlin.math.roundToInt

/**
 * Sealed 1:1 messages over the user's own relay. Everything here is client-only: the
 * keystore is decrypted on this device, envelopes are sealed with the phone's key, and
 * the relay only ever stores ciphertext. Crypto and HTTP run on one background thread
 * because Argon2id needs 64 MiB.
 *
 * The screen is laid out programmatically on purpose — this app has no `res/`.
 */
class MessagesActivity : Activity() {
    private val io = Executors.newSingleThreadExecutor()
    private val stamps = SimpleDateFormat("d MMM · HH:mm", Locale.getDefault())

    private lateinit var store: CommsStore
    private lateinit var status: TextView
    private lateinit var commsIdValue: TextView
    private lateinit var pubkeyValue: TextView
    private lateinit var relayField: EditText
    private lateinit var passField: EditText
    private lateinit var recipientField: EditText
    private lateinit var messageField: EditText
    private lateinit var inbox: LinearLayout

    /** Set once the keystore is unlocked; null until then. */
    private var client: CommsClient? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Make the system chrome part of the surface instead of fighting it.
        actionBar?.hide()
        window.setBackgroundDrawable(ColorDrawable(INK))
        window.statusBarColor = INK

        store = CommsStore(this)
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(20), dp(24), dp(20), dp(36))
            setBackgroundColor(INK)
        }

        root.addStack(header())
        root.addView(rule(), LinearLayout.LayoutParams(dp(40), dp(3)).apply { topMargin = dp(14) })

        status = TextView(this).apply {
            textSize = 13f
            setLineSpacing(0f, 1.3f)
            setPadding(dp(14), dp(12), dp(14), dp(12))
        }
        root.addStack(status, 18)
        setStatus("Enter your passphrase and unlock to load your identity.", Tone.NEUTRAL)

        relayField = field(CommsStore.DEFAULT_RELAY_URL, store.relayUrl)
        commsIdValue = monoValue("Unlock to reveal")
        commsIdValue.setTextColor(MUTED)
        pubkeyValue = monoValue("Unlock to reveal")
        pubkeyValue.setTextColor(MUTED)
        passField = field("Passphrase").apply {
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
        }
        recipientField = field("Recipient public key (hex)").apply {
            typeface = Typeface.MONOSPACE
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
        }
        messageField = field("Write your message").apply {
            setSingleLine(false)
            inputType = InputType.TYPE_CLASS_TEXT or
                InputType.TYPE_TEXT_FLAG_MULTI_LINE or
                InputType.TYPE_TEXT_FLAG_CAP_SENTENCES
            gravity = Gravity.TOP or Gravity.START
            minLines = 3
            maxLines = 8
        }

        val identity = card().apply {
            addStack(sectionLabel("Identity"))
            addStack(labelled("Comms id", commsIdValue), 2)
            addStack(labelled("Wallet public key", pubkeyValue), 14)
        }
        root.addStack(identity, 18)

        val relay = card().apply {
            addStack(sectionLabel("Relay"))
            addStack(relayField, 2)
        }
        root.addStack(relay, 18)

        val unlock = card().apply {
            addStack(sectionLabel("Keystore"))
            addStack(passField, 2)
            addStack(button("Unlock") { unlock() }, 12)
        }
        root.addStack(unlock, 18)

        val compose = card().apply {
            addStack(sectionLabel("New message"))
            addStack(recipientField, 2)
            addStack(messageField, 10)
            addStack(button("Send") { send() }, 12)
        }
        root.addStack(compose, 18)

        inbox = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        val inboxCard = card().apply {
            addStack(inboxHeader())
            addStack(inbox, 12)
        }
        root.addStack(inboxCard, 18)
        renderInbox(emptyList())

        setContentView(ScrollView(this).apply {
            isFillViewport = true
            setBackgroundColor(INK)
            addView(root)
        })
    }

    override fun onDestroy() {
        io.shutdown()
        super.onDestroy()
    }

    // --- actions -------------------------------------------------------------------------

    private fun unlock() {
        val relay = relayField.text.toString().trim()
        val passphrase = passField.text.toString()
        if (relay.isEmpty()) {
            setStatus("Enter the relay address, like http://10.0.2.2:8787.", Tone.ERROR)
            return
        }
        saveRelay(relay)
        task("Unlocking your keystore…", work = {
            val keystore = keystore()
            val relayClient = RelayClient(relay)
            val comms = CommsClient(keystore, passphrase, relayClient)
            val pubkey = comms.myPubkeyHex()
            val commsId = comms.myCommsId()
            val relayOk = try {
                relayClient.health()
            } catch (_: Exception) {
                false
            }
            Unlocked(comms, pubkey, commsId, relayOk)
        }) { unlocked ->
            client = unlocked.comms
            showIdentity(unlocked.pubkey, unlocked.commsId)
            val note = if (unlocked.relayOk) {
                "Relay reachable."
            } else {
                "Relay not reachable — you can still unlock, but sending and refreshing need it."
            }
            setStatus("Unlocked.\n$note", if (unlocked.relayOk) Tone.OK else Tone.WARN)
        }
    }

    private fun send() {
        val recipient = recipientField.text.toString().trim()
        val body = messageField.text.toString().trim()
        val comms = client
        if (comms == null) {
            setStatus("Unlock first, so your key is available to seal the message.", Tone.ERROR)
            return
        }
        if (recipient.isEmpty()) {
            setStatus("Enter the recipient's public key (64 hex characters).", Tone.ERROR)
            return
        }
        if (body.isEmpty()) {
            setStatus("Type a message before sending.", Tone.ERROR)
            return
        }
        task("Sealing and sending…", work = {
            comms.sendEnvelope(recipient, body)
        }) { id ->
            messageField.setText("")
            setStatus("Sent. Envelope ${shortId(id)} is waiting on the relay.", Tone.OK)
        }
    }

    private fun refreshInbox() {
        val comms = client
        if (comms == null) {
            setStatus("Unlock first to load your inbox.", Tone.ERROR)
            return
        }
        saveRelay(relayField.text.toString().trim())
        task("Refreshing inbox…", work = {
            val messages = comms.fetchInbox()
            if (messages.isNotEmpty()) {
                comms.ackMessages(messages.map { it.envelopeId })
            }
            messages
        }) { messages ->
            renderInbox(messages)
            val line = when (messages.size) {
                0 -> "Inbox is empty."
                1 -> "Received 1 message."
                else -> "Received ${messages.size} messages."
            }
            setStatus(line, Tone.OK)
        }
    }

    // --- rendering -----------------------------------------------------------------------

    private fun showIdentity(pubkey: String, commsId: String) {
        commsIdValue.apply {
            text = commsId
            setTextColor(TEXT)
        }
        pubkeyValue.apply {
            text = pubkey
            setTextColor(TEXT)
        }
    }

    private fun renderInbox(messages: List<ReceivedMessage>) {
        inbox.removeAllViews()
        if (messages.isEmpty()) {
            inbox.addStack(TextView(this).apply {
                text = "Nothing here yet. Messages sent to you appear after Refresh."
                setTextColor(MUTED)
                textSize = 13f
                setLineSpacing(0f, 1.25f)
            })
            return
        }
        messages.forEachIndexed { index, message ->
            val row = LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                background = rounded(SURFACE_2, 14, LINE, 1)
                setPadding(dp(14), dp(12), dp(14), dp(12))
            }
            val meta = LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
            }
            meta.addView(TextView(this).apply {
                text = shortId(message.senderCommsId)
                contentDescription = message.senderCommsId
                setTextColor(ACCENT)
                typeface = Typeface.MONOSPACE
                textSize = 12f
            }, LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f))
            meta.addView(TextView(this).apply {
                text = stamp(message.sentAtMs)
                setTextColor(MUTED)
                textSize = 11f
            })
            row.addView(meta)
            row.addView(TextView(this).apply {
                text = bodyOf(message.payloadJson)
                setTextColor(TEXT)
                textSize = 15f
                setLineSpacing(0f, 1.25f)
                setPadding(0, dp(8), 0, 0)
            })
            inbox.addStack(row, if (index == 0) 0 else 8)
        }
    }

    private fun bodyOf(payloadJson: String): String = try {
        JSONObject(payloadJson).optString("body").ifEmpty { payloadJson }
    } catch (_: Exception) {
        payloadJson
    }

    private fun stamp(sentAtMs: Long): String = stamps.format(Date(sentAtMs))

    private fun shortId(value: String): String =
        if (value.length > 16) "${value.take(8)}…${value.takeLast(6)}" else value

    // --- status --------------------------------------------------------------------------

    private enum class Tone { NEUTRAL, OK, WARN, ERROR }

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

    // --- background helper ---------------------------------------------------------------

    private fun <T> task(start: String, work: () -> T, done: (T) -> Unit) {
        setStatus(start, Tone.NEUTRAL)
        io.execute {
            val outcome = try {
                Result.success(work())
            } catch (exception: Exception) {
                Result.failure(exception)
            }
            runOnUiThread {
                outcome.fold(
                    onSuccess = done,
                    onFailure = { setStatus(it.message ?: it.toString(), Tone.ERROR) },
                )
            }
        }
    }

    // --- widgets -------------------------------------------------------------------------

    private fun header(): LinearLayout = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        addView(TextView(this@MessagesActivity).apply {
            text = "KVNC · COMMS"
            setTextColor(ACCENT)
            textSize = 11f
            letterSpacing = 0.22f
            typeface = Typeface.DEFAULT_BOLD
        })
        addView(TextView(this@MessagesActivity).apply {
            text = "Messages"
            setTextColor(TEXT)
            textSize = 30f
            typeface = Typeface.DEFAULT_BOLD
            setPadding(0, dp(6), 0, 0)
        })
        addView(TextView(this@MessagesActivity).apply {
            text = "Sealed messages over your own relay. Your keys never leave this phone."
            setTextColor(MUTED)
            textSize = 13f
            setLineSpacing(0f, 1.3f)
            setPadding(0, dp(8), 0, 0)
        })
    }

    private fun rule(): View = View(this).apply { background = rounded(ACCENT, 2) }

    private fun card(): LinearLayout = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        background = rounded(SURFACE, 18, LINE, 1)
        setPadding(dp(16), dp(16), dp(16), dp(16))
    }

    private fun inboxHeader(): LinearLayout = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        addView(
            sectionLabel("Inbox", bottomPadDp = 0),
            LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f),
        )
        addView(
            button("Refresh", filled = false) { refreshInbox() },
            LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT,
                LinearLayout.LayoutParams.WRAP_CONTENT,
            ),
        )
    }

    private fun labelled(caption: String, value: TextView): LinearLayout = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        addView(TextView(this@MessagesActivity).apply {
            text = caption
            setTextColor(MUTED)
            textSize = 12f
            setPadding(0, 0, 0, dp(4))
        })
        addStack(value)
    }

    private fun sectionLabel(caption: String, bottomPadDp: Int = 10): TextView = TextView(this).apply {
        text = caption.uppercase(Locale.US)
        setTextColor(MUTED)
        textSize = 11f
        letterSpacing = 0.18f
        typeface = Typeface.DEFAULT_BOLD
        setPadding(0, 0, 0, dp(bottomPadDp))
    }

    private fun monoValue(text: String): TextView = TextView(this).apply {
        this.text = text
        setTextColor(TEXT)
        typeface = Typeface.MONOSPACE
        textSize = 13f
        setTextIsSelectable(true)
        setLineSpacing(0f, 1.2f)
    }

    private fun field(hint: String, value: String = ""): EditText = EditText(this).apply {
        this.hint = hint
        setText(value)
        setTextColor(TEXT)
        setHintTextColor(MUTED)
        textSize = 15f
        background = rounded(SURFACE_2, 12, LINE, 1)
        setPadding(dp(14), dp(13), dp(14), dp(13))
        inputType = InputType.TYPE_CLASS_TEXT
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

    private fun ViewGroup.addStack(child: View, top: Int = 0) {
        addView(
            child,
            LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT,
            ).apply { topMargin = dp(top) },
        )
    }

    // --- io helpers ----------------------------------------------------------------------

    private fun saveRelay(relay: String) {
        store.relayUrl = relay
    }

    private fun prefs() = getSharedPreferences("kvnc", MODE_PRIVATE)

    private fun keystore(): String =
        prefs().getString("keystore", null) ?: error("Create or import a wallet first.")

    private fun dp(value: Int): Int = (value * resources.displayMetrics.density).roundToInt()

    private data class Unlocked(
        val comms: CommsClient,
        val pubkey: String,
        val commsId: String,
        val relayOk: Boolean,
    )

    companion object {
        // Ink-toned surface. Explicit so the screen reads the same on any device theme.
        private val INK = 0xFF0E1116.toInt()
        private val SURFACE = 0xFF171C23.toInt()
        private val SURFACE_2 = 0xFF1E242C.toInt()
        private val LINE = 0xFF2A323C.toInt()
        private val TEXT = 0xFFEDF1F6.toInt()
        private val MUTED = 0xFF95A0AE.toInt()
        private val ACCENT = 0xFFF0B429.toInt()
        private val ACCENT_INK = 0xFF1A1E25.toInt()
        private val ACCENT_BG = 0x1AF0B429.toInt()
        private val GOOD = 0xFF5BD6A0.toInt()
        private val GOOD_BG = 0x1A5BD6A0.toInt()
        private val BAD = 0xFFFF7B84.toInt()
        private val BAD_BG = 0x1AFF7B84.toInt()
    }
}
