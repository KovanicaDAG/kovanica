import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// 1 KVNC = 10^8 atoms (mirrors profile.rs ATOM).
const ATOM = 100_000_000

// DOM elements — node status
const networkBadge = document.getElementById('network-badge')
const produceBtn = document.getElementById('produce-btn')
const statusBtn = document.getElementById('status-btn')
const genesisEl = document.getElementById('genesis')
const tipEl = document.getElementById('tip')
const blocksEl = document.getElementById('blocks')
const mempoolEl = document.getElementById('mempool')
const peersEl = document.getElementById('peers')
const syncEl = document.getElementById('sync')
const eventsEl = document.getElementById('events')

// DOM elements — wallet
const mnemonicInput = document.getElementById('mnemonic-input')
const passphraseInput = document.getElementById('passphrase-input')
const createWalletBtn = document.getElementById('create-wallet-btn')
const unlockWalletBtn = document.getElementById('unlock-wallet-btn')
const lockWalletBtn = document.getElementById('lock-wallet-btn')
const walletFingerprint = document.getElementById('wallet-fingerprint')
const addressListEl = document.getElementById('address-list')
const addressesBtn = document.getElementById('addresses-btn')
const addrInput = document.getElementById('addr-input')
const balanceBtn = document.getElementById('balance-btn')
const historyBtn = document.getElementById('history-btn')
const walletBalance = document.getElementById('wallet-balance')
const walletHistory = document.getElementById('wallet-history')
const toInput = document.getElementById('to-input')
const amountInput = document.getElementById('amount-input')
const sendBtn = document.getElementById('send-btn')
const sendResult = document.getElementById('send-result')

let nodeReady = false
let walletUnlocked = false

function setStatus(text, cls = '') {
  networkBadge.textContent = text
  networkBadge.className = 'value ' + cls
}

function fmtHash(h) {
  if (!h || h === '—') return '—'
  return h.length > 16 ? h.slice(0, 8) + '…' + h.slice(-8) : h
}

function fmtKvnc(atoms) {
  return (atoms / ATOM).toLocaleString(undefined, { maximumFractionDigits: 8 }) + ' KVNC'
}

function updateStatus(s) {
  genesisEl.textContent = fmtHash(s.genesis)
  tipEl.textContent = fmtHash(s.tip)
  blocksEl.textContent = s.block_count.toLocaleString()
  mempoolEl.textContent = s.mempool_size.toLocaleString()
  peersEl.textContent = s.peers.toLocaleString()
  syncEl.textContent = s.sync_progress ? `${s.sync_progress[0].toLocaleString()}/${s.sync_progress[1].toLocaleString()}` : 'synced'
}

function addEvent(type, data) {
  const loading = eventsEl.querySelector('.loading')
  if (loading) loading.remove()

  const div = document.createElement('div')
  div.className = 'event'
  const time = new Date().toLocaleTimeString()
  div.innerHTML = `
    <span class="event-time">[${time}]</span>
    <span class="event-type">${type}</span>
    <span class="event-data">${JSON.stringify(data)}</span>
  `
  eventsEl.insertBefore(div, eventsEl.firstChild)

  while (eventsEl.children.length > 100) {
    eventsEl.removeChild(eventsEl.lastChild)
  }
}

function walletError(e) {
  addEvent('Error', { message: String(e) })
}

function setWalletUnlocked(unlocked) {
  walletUnlocked = unlocked
  sendBtn.disabled = !(unlocked && nodeReady)
  addressesBtn.disabled = !unlocked
}

function setAddressDefaults(addresses) {
  if (addresses && addresses.length) {
    if (!addrInput.value) addrInput.value = addresses[0]
    if (!toInput.value) toInput.value = addresses[0]
    addrInput.placeholder = addresses[0] === '—' ? 'kvnc…dag or hex' : addresses[0]
  }
}

function renderAddresses(addresses) {
  addressListEl.innerHTML = addresses.length
    ? addresses.map((a) => `<div class="list-item">${a}</div>`).join('')
    : '<div class="list-item">no wallet unlocked</div>'
}

function renderHistory(events) {
  walletHistory.innerHTML = events.length
    ? events.map((e) => {
        const dir = e.type || Object.keys(e)[0] || '?'
        const cls = dir === 'Received' ? 'dir-received' : dir === 'Sent' ? 'dir-sent' : ''
        const body = e.tx_id ? `${fmtHash(e.tx_id)} · ${fmtKvnc(e.amount)}` : JSON.stringify(e)
        return `<div class="list-item"><span class="${cls}">${dir}</span> ${body}</div>`
      }).join('')
    : '<div class="list-item">no history</div>'
}

async function refreshStatus() {
  try {
    const status = await invoke('get_status')
    updateStatus(status)
  } catch (e) {
    console.error('status error:', e)
    addEvent('Error', { message: 'Failed to get status' })
  }
}

async function produceBlock() {
  produceBtn.disabled = true
  produceBtn.textContent = 'Producing...'
  try {
    const result = await invoke('produce_block')
    addEvent('BlockProduced', result)
    await refreshStatus()
  } catch (e) {
    console.error('produce error:', e)
    addEvent('Error', { message: String(e) })
  } finally {
    produceBtn.disabled = false
    produceBtn.textContent = 'Produce Block'
  }
}

async function createWallet() {
  try {
    const [mnemonic, fingerprint] = await invoke('create_wallet', {
      passphrase: passphraseInput.value || null,
    })
    mnemonicInput.value = mnemonic
    walletFingerprint.textContent = 'fingerprint ' + fingerprint
    walletFingerprint.className = 'value sm ok'
    setWalletUnlocked(true)
    await refreshAddresses()
    addEvent('WalletCreated', { fingerprint })
  } catch (e) {
    walletError(e)
  }
}

async function unlockWallet() {
  try {
    const fingerprint = await invoke('unlock_wallet', {
      mnemonic: mnemonicInput.value.trim(),
      passphrase: passphraseInput.value || null,
    })
    walletFingerprint.textContent = 'fingerprint ' + fingerprint
    walletFingerprint.className = 'value sm ok'
    setWalletUnlocked(true)
    await refreshAddresses()
    addEvent('WalletUnlocked', { fingerprint })
  } catch (e) {
    walletError(e)
  }
}

async function lockWallet() {
  try {
    await invoke('lock_wallet')
    walletUnlocked = false
    walletFingerprint.textContent = 'no unlocked wallet'
    walletFingerprint.className = 'value sm'
    setWalletUnlocked(false)
    renderAddresses([])
    addEvent('WalletLocked', {})
  } catch (e) {
    walletError(e)
  }
}

async function refreshAddresses() {
  try {
    const addresses = await invoke('get_addresses', { count: 5 })
    renderAddresses(addresses)
    setAddressDefaults(addresses)
  } catch (e) {
    walletError(e)
  }
}

async function loadBalance() {
  if (!addrInput.value.trim()) return
  try {
    const balance = await invoke('get_balance', { address: addrInput.value.trim() })
    walletBalance.textContent = fmtKvnc(balance)
    walletBalance.className = 'value sm ok'
  } catch (e) {
    walletError(e)
  }
}

async function loadHistory() {
  if (!addrInput.value.trim()) return
  try {
    const events = await invoke('get_history', {
      address: addrInput.value.trim(),
      maxBlocks: 0,
    })
    renderHistory(events)
  } catch (e) {
    walletError(e)
  }
}

async function send() {
  const to = toInput.value.trim()
  const amount = Number(amountInput.value)
  if (!to || isNaN(amount) || amount <= 0) {
    walletError('Recipient and a positive amount are required')
    return
  }
  const atoms = Math.floor(amount * ATOM)
  sendBtn.disabled = true
  sendBtn.textContent = 'Sending...'
  try {
    const txId = await invoke('send_from_wallet', { toAddress: to, amount: atoms })
    sendResult.textContent = 'tx ' + txId
    sendResult.className = 'value sm ok'
    addEvent('Sent', { txId })
    amountInput.value = ''
  } catch (e) {
    sendResult.textContent = String(e)
    sendResult.className = 'value sm err'
    walletError(e)
  } finally {
    setWalletUnlocked(true)
    sendBtn.textContent = 'Send'
  }
}

async function init() {
  setStatus('Starting node…', 'warn')

  await listen('node-event', async (event) => {
    const { type, data } = event.payload
    addEvent(type, data)

    if (type === 'Booted') {
      setStatus(data.genesis.slice(0, 12) + '…', 'ok')
      networkBadge.title = data.genesis
      nodeReady = true
      produceBtn.disabled = false
      statusBtn.disabled = false
      setWalletUnlocked(walletUnlocked)
      await refreshStatus()
    } else if (type === 'TipChanged' || type === 'BlockProduced' || type === 'BlockReceived') {
      await refreshStatus()
    }
  })

  setTimeout(refreshStatus, 500)
}

produceBtn.addEventListener('click', produceBlock)
statusBtn.addEventListener('click', refreshStatus)
createWalletBtn.addEventListener('click', createWallet)
unlockWalletBtn.addEventListener('click', unlockWallet)
lockWalletBtn.addEventListener('click', lockWallet)
addressesBtn.addEventListener('click', refreshAddresses)
balanceBtn.addEventListener('click', loadBalance)
historyBtn.addEventListener('click', loadHistory)
sendBtn.addEventListener('click', send)

init()