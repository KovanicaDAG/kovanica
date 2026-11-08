import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { ExplorerBlock, ExplorerTransaction } from '../types'
import { formatAddress, formatAmount, formatRelativeTime, DEFAULT_RPC_URL, DEFAULT_INDEXER_HTTP_URL, DEFAULT_INDEXER_WS_URL } from '../types'
import { KovanicaIndexerClient, IndexerWebSocketClient, type IndexedEvent } from '@kovanica/api-client'

export function Explorer() {
  const [blocks, setBlocks] = useState<ExplorerBlock[]>([])
  const [latestHeight, setLatestHeight] = useState<number>(0)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [rpcUrl, setRpcUrl] = useState(DEFAULT_RPC_URL)
  const [indexerHttpUrl, setIndexerHttpUrl] = useState(DEFAULT_INDEXER_HTTP_URL)
  const [indexerWsUrl, setIndexerWsUrl] = useState(DEFAULT_INDEXER_WS_URL)
  const [indexerLogs, setIndexerLogs] = useState<IndexedEvent[]>([])
  const [indexerLive, setIndexerLive] = useState<IndexedEvent[]>([])
  const [indexerError, setIndexerError] = useState<string | null>(null)
  const [indexerAvailable, setIndexerAvailable] = useState<boolean | null>(null)
  const [dagTips, setDagTips] = useState<{ tips: string[]; count: number; max_round?: number } | null>(null)

  useEffect(() => {
    fetchData()
    const interval = setInterval(fetchData, 10000)
    return () => clearInterval(interval)
  }, [rpcUrl])

  useEffect(() => {
    if (!indexerHttpUrl) {
      setIndexerAvailable(false)
      return
    }
    let cancelled = false
    const load = async () => {
      try {
        const client = new KovanicaIndexerClient(indexerHttpUrl)
        await client.getStatus()
        const logs = await client.getLogs({ limit: 50 })
        if (cancelled) return
        setIndexerAvailable(true)
        setIndexerLogs(logs)
        setIndexerError(null)
      } catch (e) {
        if (cancelled) return
        setIndexerAvailable(false)
        setIndexerError(e instanceof Error ? e.message : 'Indexer unavailable')
      }
    }
    load()
    const interval = setInterval(load, 15000)
    return () => {
      cancelled = true
      clearInterval(interval)
    }
  }, [indexerHttpUrl])

  useEffect(() => {
    if (!indexerWsUrl) return
    const client = new IndexerWebSocketClient(indexerWsUrl)
    const offEvent = client.onEvent((ev) => {
      setIndexerLive((prev) => [ev, ...prev].slice(0, 50))
    })
    const offError = client.onError(() => {
      setIndexerError('Indexer WebSocket error')
    })
    client.connect()
    return () => {
      offEvent()
      offError()
      client.close()
    }
  }, [indexerWsUrl])

  const fetchData = async () => {
    try {
      setLoading(true)
      setError(null)
      const res = await fetch(`${rpcUrl}/rpc`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'kovanica_blockNumber',
          params: [],
          id: 1,
        }),
      })
      const data = await res.json()
      if (data.error) throw new Error(data.error.message)
      const height = parseInt(data.result, 16)
      setLatestHeight(height)

      // Query active DAG tips if node supports kovanica_getDagTips
      try {
        const tipsRes = await fetch(`${rpcUrl}/rpc`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'kovanica_getDagTips',
            params: [],
            id: 2,
          }),
        })
        const tipsData = await tipsRes.json()
        if (tipsData.result && Array.isArray(tipsData.result.tips)) {
          setDagTips(tipsData.result)
        } else {
          setDagTips(null)
        }
      } catch {
        setDagTips(null)
      }

      // Fetch last 10 blocks
      const blockPromises: Promise<ExplorerBlock | null>[] = []
      for (let i = 0; i < 10; i++) {
        const h = height - i
        if (h < 0) break
        blockPromises.push(fetchBlock(h))
      }
      const results = await Promise.all(blockPromises)
      setBlocks(results.filter((b): b is ExplorerBlock => b !== null) as ExplorerBlock[])
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch data')
    } finally {
      setLoading(false)
    }
  }

  const fetchBlock = async (height: number): Promise<ExplorerBlock | null> => {
    try {
      const res = await fetch(`${rpcUrl}/rpc`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'kovanica_getBlockByNumber',
          params: [height, true],
          id: 1,
        }),
      })
      const data = await res.json()
      if (data.error) throw new Error(data.error.message)
      if (!data.result) return null

      const block = data.result
      const txs: ExplorerTransaction[] = (block.transactions || []).map((tx: any) => ({
        hash: tx.hash,
        sender: tx.sender || '',
        nonce: parseInt(tx.nonce || '0', 16),
        fee: tx.fee || '0',
        kind: parseTxKind(tx),
        timestamp: Date.now() - (10 - (block.number ? parseInt(block.number, 16) : 0)) * 12000,
      }))

      return {
        hash: block.hash,
        round: block.number ? parseInt(block.number, 16) : (typeof block.round === 'number' ? block.round : 0),
        parent_hashes: block.parent_hashes || block.parents || [],
        transactions: txs,
        author: block.author || 0,
        timestamp: block.timestamp ? (typeof block.timestamp === 'number' ? block.timestamp : parseInt(block.timestamp, 16)) : Date.now(),
        nonce: block.nonce,
        pow_hash: block.pow_hash,
        target_bits: block.target_bits,
        blue_score: block.blue_score,
        blue_work: block.blue_work,
        selected_parent: block.selected_parent,
        mergeset_blues: block.mergeset_blues,
        mergeset_reds: block.mergeset_reds,
      }
    } catch {
      return null
    }
  }

  return (
    <div className="explorer">
      <div className="page-header">
        <h1 className="page-title">Block Explorer</h1>
        <div style={{ display: 'flex', gap: '16px', alignItems: 'center' }}>
          <div className="form-group" style={{ marginBottom: 0 }}>
            <label className="form-label">RPC Endpoint</label>
            <input
              type="text"
              value={rpcUrl}
              onChange={(e) => setRpcUrl(e.target.value)}
              placeholder="http://127.0.0.1:8545"
              style={{ width: '300px' }}
            />
          </div>
          <button className="btn-secondary" onClick={fetchData} disabled={loading}>
            {loading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
      </div>

      {error && (
        <div className="card" style={{ backgroundColor: 'rgba(248, 81, 73, 0.1)', borderColor: 'var(--color-danger)' }}>
          <strong style={{ color: 'var(--color-danger)' }}>Error:</strong> {error}
        </div>
      )}

      <div className="stat-grid">
        <div className="stat-card">
          <div className="stat-label">Block Height</div>
          <div className="stat-value monospace">{latestHeight.toLocaleString()}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Latest Block</div>
          <div className="stat-value monospace">{blocks[0] ? `#${blocks[0].round}` : '—'}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Active DAG Tips</div>
          <div className="stat-value monospace">{dagTips ? dagTips.tips.length : '—'}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Total TXs (last 10 blocks)</div>
          <div className="stat-value monospace">{blocks.reduce((sum, b) => sum + b.transactions.length, 0)}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">Avg TXs/Block</div>
          <div className="stat-value monospace">
            {blocks.length > 0
              ? (blocks.reduce((sum, b) => sum + b.transactions.length, 0) / blocks.length).toFixed(1)
              : '—'}
          </div>
        </div>
      </div>

      {dagTips && dagTips.tips.length > 0 && (
        <div className="card" style={{ marginBottom: '24px' }}>
          <h3 style={{ marginBottom: '12px' }}>Active DAG Tips ({dagTips.tips.length})</h3>
          <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
            {dagTips.tips.map((tip) => (
              <span
                key={tip}
                className="monospace"
                style={{
                  padding: '4px 10px',
                  borderRadius: '4px',
                  background: 'rgba(56, 139, 253, 0.15)',
                  border: '1px solid rgba(56, 139, 253, 0.4)',
                }}
              >
                <Link to={`/block/${tip}`} style={{ color: 'inherit' }}>
                  {formatAddress(tip)}
                </Link>
              </span>
            ))}
          </div>
        </div>
      )}

      <div className="tabs">
        <button className="tab active">Blocks</button>
        <button className="tab">Transactions</button>
        <button className="tab">Accounts</button>
        <button className="tab">Validators</button>
      </div>

      <div className="card">
        <div className="table-container">
          <table>
            <thead>
              <tr>
                <th>Round</th>
                <th>Hash</th>
                <th>GHOSTDAG</th>
                <th>Blue Score</th>
                <th>PoW Hash / Nonce</th>
                <th>Target Bits</th>
                <th>Author</th>
                <th>TXs</th>
                <th>Selected Parent</th>
                <th>Parents</th>
                <th>Time</th>
              </tr>
            </thead>
            <tbody>
              {loading && blocks.length === 0 ? (
                <tr><td colSpan={11} className="loading"><div className="spinner"></div>Loading blocks...</td></tr>
              ) : blocks.length === 0 ? (
                <tr><td colSpan={11} className="empty-state">No blocks found</td></tr>
              ) : (
                blocks.map(block => {
                  const isRed = Boolean(block.mergeset_reds && block.mergeset_reds.includes(block.hash))
                  return (
                    <tr key={block.hash}>
                      <td className="monospace">#{block.round}</td>
                      <td className="address"><Link to={`/block/${block.hash}`}>{formatAddress(block.hash)}</Link></td>
                      <td>
                        <span
                          style={{
                            padding: '2px 8px',
                            borderRadius: '4px',
                            fontWeight: 600,
                            fontSize: '0.85em',
                            background: isRed ? 'rgba(248, 81, 73, 0.2)' : 'rgba(63, 185, 80, 0.2)',
                            color: isRed ? '#f85149' : '#3fb950',
                            border: isRed ? '1px solid #f85149' : '1px solid #3fb950',
                          }}
                        >
                          {isRed ? 'RED' : 'BLUE'}
                        </span>
                      </td>
                      <td className="monospace">{block.blue_score !== undefined ? String(block.blue_score) : '—'}</td>
                      <td className="address monospace">
                        {block.pow_hash ? (
                          <span title={`Nonce: ${block.nonce ?? 0}`}>
                            {formatAddress(block.pow_hash)} (n:{block.nonce ?? 0})
                          </span>
                        ) : (
                          '—'
                        )}
                      </td>
                      <td className="monospace">
                        {block.target_bits !== undefined
                          ? (typeof block.target_bits === 'number'
                              ? `0x${block.target_bits.toString(16)}`
                              : String(block.target_bits))
                          : '—'}
                      </td>
                      <td className="monospace">{block.author}</td>
                      <td className="monospace">{block.transactions.length}</td>
                      <td className="address monospace">
                        {block.selected_parent ? (
                          <Link to={`/block/${block.selected_parent}`}>{formatAddress(block.selected_parent)}</Link>
                        ) : (
                          '—'
                        )}
                        {block.mergeset_blues ? (
                          <span style={{ fontSize: '0.8em', color: 'var(--color-muted)' }}>
                            {' '}(+{block.mergeset_blues.length}B/{block.mergeset_reds?.length ?? 0}R)
                          </span>
                        ) : null}
                      </td>
                      <td className="address monospace">{block.parent_hashes.slice(0, 2).map(formatAddress).join(', ') || '—'}</td>
                      <td>{formatRelativeTime(block.timestamp)}</td>
                    </tr>
                  )
                })
              )}
            </tbody>
          </table>
        </div>
      </div>

      <div className="card" style={{ marginTop: '24px' }}>
        <h3 style={{ marginBottom: '16px' }}>Recent Transactions</h3>
        <div className="table-container">
          <table>
            <thead>
              <tr>
                <th>Hash</th>
                <th>Sender</th>
                <th>Nonce</th>
                <th>Fee</th>
                <th>Kind</th>
              </tr>
            </thead>
            <tbody>
              {blocks.flatMap(b => b.transactions).slice(0, 20).map((tx, i) => (
                <tr key={tx.hash || i}>
                  <td className="address"><Link to={`/tx/${tx.hash}`}>{formatAddress(tx.hash)}</Link></td>
                  <td className="address">{formatAddress(tx.sender)}</td>
                  <td className="monospace">{tx.nonce}</td>
                  <td className="monospace">{formatAmount(tx.fee)} KUNA</td>
                  <td>{tx.kind.type}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
      <div className="card" style={{ marginTop: '24px' }}>
        <h3 style={{ marginBottom: '16px' }}>
          Indexer Events
          {indexerAvailable === false ? ' (indexer unavailable - node RPC fallback)' : ''}
        </h3>
        <div style={{ display: 'flex', gap: '16px', marginBottom: '12px', flexWrap: 'wrap' }}>
          <div className="form-group" style={{ marginBottom: 0 }}>
            <label className="form-label">Indexer HTTP</label>
            <input type="text" value={indexerHttpUrl} onChange={(e) => setIndexerHttpUrl(e.target.value)} style={{ width: '260px' }} />
          </div>
          <div className="form-group" style={{ marginBottom: 0 }}>
            <label className="form-label">Indexer WS</label>
            <input type="text" value={indexerWsUrl} onChange={(e) => setIndexerWsUrl(e.target.value)} style={{ width: '260px' }} />
          </div>
        </div>
        {indexerError && (
          <div className="card" style={{ backgroundColor: 'rgba(248, 81, 73, 0.1)', borderColor: 'var(--color-danger)', marginBottom: '12px' }}>
            <strong style={{ color: 'var(--color-danger)' }}>Indexer:</strong> {indexerError}
          </div>
        )}
        <div style={{ marginBottom: '8px' }}>
          <strong>Live events:</strong> {indexerLive.length}
        </div>
        <div className="table-container">
          <table>
            <thead>
              <tr>
                <th>Height</th>
                <th>Address</th>
                <th>Topic</th>
                <th>Data</th>
                <th>TX</th>
              </tr>
            </thead>
            <tbody>
              {indexerLogs.length === 0 && indexerLive.length === 0 ? (
                <tr><td colSpan={5} className="empty-state">No indexer events</td></tr>
              ) : (
                [...indexerLive, ...indexerLogs].slice(0, 50).map((ev, i) => (
                  <tr key={`${ev.id}-${i}`}>
                    <td className="monospace">{ev.height}</td>
                    <td className="address monospace">{formatAddress(bytesToHex(ev.address))}</td>
                    <td className="monospace">{bytesToText(ev.topic) || bytesToHex(ev.topic)}</td>
                    <td className="monospace">{ev.data.length} bytes</td>
                    <td className="address">{formatAddress(bytesToHex(ev.tx_hash))}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}


function bytesToHex(bytes: number[]): string {
  return bytes.map((b) => b.toString(16).padStart(2, '0')).join('')
}

function bytesToText(bytes: number[]): string {
  try {
    return new TextDecoder().decode(new Uint8Array(bytes))
  } catch {
    return ''
  }
}

function parseTxKind(tx: any): ExplorerTransaction['kind'] {
  if (tx.kind && typeof tx.kind === 'object') {
    const keys = Object.keys(tx.kind)
    if (keys.length === 1) {
      const type = keys[0]
      const value = tx.kind[type]
      return { type, ...value }
    }
  }
  return { type: 'Other', data: JSON.stringify(tx.kind) }
}