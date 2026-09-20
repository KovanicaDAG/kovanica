import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// DOM elements
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

let nodeReady = false

function setStatus(text, cls = '') {
  networkBadge.textContent = text
  networkBadge.className = 'value ' + cls
}

function fmtHash(h) {
  if (!h || h === '—') return '—'
  return h.length > 16 ? h.slice(0, 8) + '…' + h.slice(-8) : h
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

  // Keep last 100 events
  while (eventsEl.children.length > 100) {
    eventsEl.removeChild(eventsEl.lastChild)
  }
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

async function init() {
  setStatus('Starting node…', 'warn')

  // Listen for node events from the Rust side
  await listen('node-event', async (event) => {
    const { type, data } = event.payload
    addEvent(type, data)

    // Update status on key events
    if (type === 'Booted') {
      setStatus(data.genesis.slice(0, 12) + '…', 'ok')
      networkBadge.title = data.genesis
      nodeReady = true
      produceBtn.disabled = false
      statusBtn.disabled = false
      await refreshStatus()
    } else if (type === 'TipChanged' || type === 'BlockProduced' || type === 'BlockReceived') {
      await refreshStatus()
    }
  })

  // Initial status fetch (will wait for boot)
  setTimeout(refreshStatus, 500)
}

produceBtn.addEventListener('click', produceBlock)
statusBtn.addEventListener('click', refreshStatus)

init()