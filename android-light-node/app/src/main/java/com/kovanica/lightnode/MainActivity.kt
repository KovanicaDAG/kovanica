package com.kovanica.lightnode

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.Column
import androidx.compose.material3.ProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.livedata.observeAsState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.fillMaxSize
import androidx.compose.ui.layout.fillMaxWidth
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.lifecycleScope
import com.kovanica.lightnode.data.LightNodeRepository
import com.kovanica.lightnode.ui.WalletViewModel
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {

    private val lightNodeRepository: LightNodeRepository by lazy {
        val app = application as KovanicaApplication
        LightNodeRepository(this, app.getSyncDispatcher())
    }

    private val walletViewModel: WalletViewModel by viewModels {
        androidx.lifecycle.ViewModelProvider.Factory { clazz ->
            WalletViewModel(lightNodeRepository, WalletRepository(lightNodeRepository, app.getSyncDispatcher(), okhttp3.OkHttpClient()))
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            KovanicaLightNodeTheme {
                MainScreen(viewModel = walletViewModel)
            }
        }
        
        // Initialize light node on first launch
        lifecycleScope.launch {
            walletViewModel.initialize()
        }
    }
}

@Composable
fun MainScreen(viewModel: WalletViewModel) {
    val nodeState by viewModel.nodeState.observeAsState()
    
    when {
        nodeState == null || !nodeState.isInitialized -> {
            GenesisGateScreen(onVerified = { viewModel.sync() })
        }
        else -> {
            WalletScreen(viewModel = viewModel)
        }
    }
}

@Composable
fun GenesisGateScreen(onVerified: () -> Unit) {
    var status by remember { mutableStateOf("Connecting to live network...") }
    
    androidx.compose.material3.Scaffold(
        containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surface
    ) { padding ->
        androidx.compose.material3.Box(
            modifier = androidx.compose.ui.Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(24.dp),
            contentAlignment = androidx.compose.ui.Alignment.Center
        ) {
            androidx.compose.material3.Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = androidx.compose.foundation.layout.Arrangement.Center
            ) {
                androidx.compose.material3.Icon(
                    imageVector = androidx.compose.material.icons.Icons.Filled.Sync,
                    contentDescription = "Syncing",
                    modifier = androidx.compose.ui.Modifier.size(64.dp),
                    tint = androidx.compose.material3.MaterialTheme.colorScheme.primary
                )
                androidx.compose.foundation.layout.Spacer(modifier = androidx.compose.ui.Modifier.padding(top = 16.dp))
                androidx.compose.material3.Text(
                    text = "Kovanica Light Node",
                    fontSize = 28.sp,
                    fontWeight = FontWeight.Bold,
                    color = androidx.compose.material3.MaterialTheme.colorScheme.onSurface
                )
                androidx.compose.foundation.layout.Spacer(modifier = androidx.compose.ui.Modifier.padding(top = 8.dp))
                androidx.compose.material3.Text(
                    text = "Genesis gate: verifying live network parameters...",
                    fontSize = 16.sp,
                    color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant,
                    textAlign = androidx.compose.ui.text.TextAlign.Center
                )
                androidx.compose.foundation.layout.Spacer(modifier = androidx.compose.ui.Modifier.padding(top = 24.dp))
                androidx.compose.material3.ProgressIndicator(
                    modifier = androidx.compose.ui.Modifier.size(48.dp)
                )
                androidx.compose.foundation.layout.Spacer(modifier = androidx.compose.ui.Modifier.padding(top = 16.dp))
                androidx.compose.material3.Text(
                    text = status,
                    fontSize = 14.sp,
                    color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
fun WalletScreen(viewModel: WalletViewModel) {
    val nodeState by viewModel.nodeState.observeAsState()
    
    androidx.compose.material3.Scaffold(
        topBar = {
            androidx.compose.material3.TopAppBar(
                title = { androidx.compose.material3.Text("Kovanica Light Node") },
                colors = androidx.compose.material3.TopAppBarDefaults.topAppBarColors(
                    containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surfaceContainer
                )
            )
        }
    ) { padding ->
        androidx.compose.material3.Box(
            modifier = androidx.compose.ui.Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            androidx.compose.material3.Column(
                modifier = androidx.compose.ui.Modifier
                    .fillMaxSize()
                    .padding(16.dp),
                verticalArrangement = androidx.compose.foundation.layout.Arrangement.spacedBy(16.dp)
            ) {
                // Status card
                StatusCard(state = nodeState)
                
                // Balance card
                BalanceCard()
                
                // Action buttons
                ActionButtons(viewModel = viewModel)
                
                // Staking section
                StakingSection(viewModel = viewModel)
            }
        }
    }
}

@Composable
fun StatusCard(state: com.kovanica.lightnode.data.LightNodeRepository.NodeState?) {
    androidx.compose.material3.Card(
        modifier = androidx.compose.ui.Modifier.fillMaxWidth(),
        colors = androidx.compose.material3.CardDefaults.cardColors(
            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surfaceContainerHigh
        )
    ) {
        androidx.compose.material3.Column(
            modifier = androidx.compose.ui.Modifier.padding(16.dp),
            verticalArrangement = androidx.compose.foundation.layout.Arrangement.spacedBy(8.dp)
        ) {
            androidx.compose.material3.Row(
                modifier = androidx.compose.ui.Modifier.fillMaxWidth(),
                horizontalArrangement = androidx.compose.foundation.layout.Arrangement.SpaceBetween
            ) {
                androidx.compose.material3.Text("Node Status", fontWeight = FontWeight.Bold, fontSize = 18.sp)
                androidx.compose.material3.Text(
                    if (state?.isSyncing == true) "Syncing..." else "Ready",
                    color = if (state?.isSyncing == true) Color.Orange else Color.Green
                )
            }
            state?.let { s ->
                androidx.compose.material3.Row(
                    modifier = androidx.compose.ui.Modifier.fillMaxWidth(),
                    horizontalArrangement = androidx.compose.foundation.layout.Arrangement.SpaceBetween
                ) {
                    androidx.compose.material3.Text("Height: ${s.blockHeight}")
                    androidx.compose.material3.Text("Peers: ${s.peerCount}")
                }
            }
        }
    }
}

@Composable
fun BalanceCard() {
    androidx.compose.material3.Card(
        modifier = androidx.compose.ui.Modifier.fillMaxWidth(),
        colors = androidx.compose.material3.CardDefaults.cardColors(
            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.primaryContainer
        )
    ) {
        androidx.compose.material3.Column(
            modifier = androidx.compose.ui.Modifier.padding(16.dp),
            verticalArrangement = androidx.compose.foundation.layout.Arrangement.spacedBy(8.dp)
        ) {
            androidx.compose.material3.Text("Balance", fontSize = 16.sp, color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant)
            androidx.compose.material3.Text("0.00000000 KVNC", fontSize = 32.sp, fontWeight = FontWeight.Bold)
            androidx.compose.material3.Text("Tap to refresh", fontSize = 12.sp, color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
fun ActionButtons(viewModel: WalletViewModel) {
    androidx.compose.material3.Row(
        modifier = androidx.compose.ui.Modifier.fillMaxWidth(),
        horizontalArrangement = androidx.compose.foundation.layout.Arrangement.spacedBy(12.dp)
    ) {
        androidx.compose.material3.Button(
            modifier = androidx.compose.ui.Modifier.weight(1f),
            onClick = { viewModel.sync() }
        ) {
            androidx.compose.material3.Icon(
                imageVector = androidx.compose.material.icons.Icons.Filled.Sync,
                contentDescription = "Sync"
            )
            androidx.compose.foundation.layout.Spacer(modifier = androidx.compose.ui.Modifier.padding(end = 8.dp))
            androidx.compose.material3.Text("Sync")
        }
        androidx.compose.material3.OutlinedButton(
            modifier = androidx.compose.ui.Modifier.weight(1f),
            onClick = { /* faucet */ }
        ) {
            androidx.compose.material3.Text("Faucet")
        }
        androidx.compose.material3.OutlinedButton(
            modifier = androidx.compose.ui.Modifier.weight(1f),
            onClick = { /* send */ }
        ) {
            androidx.compose.material3.Text("Send")
        }
    }
}

@Composable
fun StakingSection(viewModel: WalletViewModel) {
    val stakingState by viewModel.stakingState.observeAsState()
    
    androidx.compose.material3.Card(
        modifier = androidx.compose.ui.Modifier.fillMaxWidth()
    ) {
        androidx.compose.material3.Column(
            modifier = androidx.compose.ui.Modifier.padding(16.dp),
            verticalArrangement = androidx.compose.foundation.layout.Arrangement.spacedBy(12.dp)
        ) {
            androidx.compose.material3.Text("Staking", fontSize = 18.sp, fontWeight = FontWeight.Bold)
            
            stakingState?.let { s ->
                androidx.compose.material3.Row(
                    modifier = androidx.compose.ui.Modifier.fillMaxWidth(),
                    horizontalArrangement = androidx.compose.foundation.layout.Arrangement.SpaceBetween
                ) {
                    androidx.compose.material3.Text(
                        if (s.isValidator) "Validator: ACTIVE" else "Validator: INACTIVE",
                        color = if (s.isValidator) Color.Green else Color.Red
                    )
                    androidx.compose.material3.Text("Stake: ${Format.atomsToKvnc(s.myStake)} KVNC")
                }
            }
            
            androidx.compose.material3.Row(
                modifier = androidx.compose.ui.Modifier.fillMaxWidth(),
                horizontalArrangement = androidx.compose.foundation.layout.Arrangement.spacedBy(12.dp)
            ) {
                androidx.compose.material3.Button(
                    modifier = androidx.compose.ui.Modifier.weight(1f),
                    onClick = { viewModel.enableValidator("") }
                ) {
                    androidx.compose.material3.Text("Enable Validator")
                }
                androidx.compose.material3.OutlinedButton(
                    modifier = androidx.compose.ui.Modifier.weight(1f),
                    onClick = { viewModel.produceBlock() }
                ) {
                    androidx.compose.material3.Text("Produce Block")
                }
            }
        }
    }
}