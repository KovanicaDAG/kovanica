package com.kovanica.lightnode

import android.app.Application
import uniffi.kovanica.LightNode
import uniffi.kovanica.LightConfig

class KovanicaApplication : Application() {

    private var lightNode: LightNode? = null
    private val syncDispatcher = kotlinx.coroutines.CoroutineDispatcher() // will be set in onCreate

    override fun onCreate() {
        super.onCreate()
        // Use a dedicated single-thread dispatcher for Rust FFI calls
        // (per plan: serialized node ops need FIFO)
        syncDispatcher = kotlinx.coroutines.Executors.newSingleThreadExecutor().asCoroutineDispatcher()
    }

    /**
     * Get or create the LightNode instance with live network parameters.
     * Must be called from syncDispatcher context.
     */
    suspend fun getOrCreateLightNode(): LightNode = kotlinx.coroutines.withContext(syncDispatcher) {
        if (lightNode != null) return@withContext lightNode!!
        
        // Live network parameters (from /api/bootstrap + /api/state)
        // k=3, subsidy=200*ATOM, founder_amount=200*ATOM, founder_seed=1, pruning MAX
        val config = LightConfig(
            k = 3,
            subsidy = 20_000_000_000L, // 200 KVNC in atoms
            founderAmount = 20_000_000_000L, // 200 KVNC in atoms
            founderSeed = 1,
            finalityDepth = Long.MAX_VALUE, // MAX pruning
            payloadPruningDepth = Long.MAX_VALUE
        )
        
        val node = LightNode(config)
        lightNode = node
        node
    }

    fun getSyncDispatcher() = syncDispatcher
    
    override fun onTerminate() {
        syncDispatcher.close()
        super.onTerminate()
    }

    companion object {
        const val LIGHT_SYNC_FILE = "light_sync.bin"
        const val OPERATOR_WALLET_FILE = "operator_wallet.key"
    }
}