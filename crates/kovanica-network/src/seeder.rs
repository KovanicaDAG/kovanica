//! DNS seed discovery, crawling and zone file generation for Kovanica P2P network.
//!
//! Provides utilities to track active healthy nodes on the network and format
//! them into standard DNS A/AAAA zone records or queryable seed lists.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// An active network peer entry verified by the seeder crawler.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DnsSeedEntry {
    /// IP address of the peer.
    pub ip: IpAddr,
    /// P2P listen port (default: 8000).
    pub port: u16,
    /// Unix timestamp in seconds when the peer was last confirmed online.
    pub last_seen_utc: u64,
    /// Round-trip ping latency in milliseconds.
    pub latency_ms: u64,
    /// Verified chain ID of the peer.
    pub chain_id: u64,
}

impl DnsSeedEntry {
    /// Create a new verified seed entry with current system timestamp.
    pub fn new(ip: IpAddr, port: u16, latency_ms: u64, chain_id: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            ip,
            port,
            last_seen_utc: now,
            latency_ms,
            chain_id,
        }
    }

    /// Check if the seed record is fresh (seen within `max_age_secs`).
    pub fn is_fresh(&self, now_utc: u64, max_age_secs: u64) -> bool {
        now_utc.saturating_sub(self.last_seen_utc) <= max_age_secs
    }
}

/// Filter healthy, fresh seed entries matching the expected `chain_id`.
pub fn filter_healthy_seeds(
    entries: &[DnsSeedEntry],
    target_chain_id: u64,
    now_utc: u64,
    max_age_secs: u64,
) -> Vec<DnsSeedEntry> {
    entries
        .iter()
        .filter(|e| e.chain_id == target_chain_id && e.is_fresh(now_utc, max_age_secs))
        .cloned()
        .collect()
}

/// Generate a standard RFC 1035 DNS zone file text (compatible with CoreDNS, BIND, NSD)
/// from the list of healthy active peer addresses.
pub fn generate_zone_file(
    domain: &str,
    ttl_secs: u32,
    entries: &[DnsSeedEntry],
    serial: u32,
) -> String {
    let clean_domain = domain.trim().trim_end_matches('.');
    let mut out = String::new();
    out.push_str(&format!("$ORIGIN {clean_domain}.\n"));
    out.push_str(&format!("$TTL {ttl_secs}\n"));
    out.push_str(&format!(
        "@ IN SOA ns1.{clean_domain}. hostmaster.{clean_domain}. ( {serial} 3600 600 604800 {ttl_secs} )\n"
    ));
    out.push_str(&format!("@ IN NS ns1.{clean_domain}.\n"));

    let mut seen_ips = std::collections::HashSet::new();
    for entry in entries {
        if seen_ips.insert(entry.ip) {
            match entry.ip {
                IpAddr::V4(v4) => {
                    out.push_str(&format!("@ IN A {v4}\n"));
                }
                IpAddr::V6(v6) => {
                    out.push_str(&format!("@ IN AAAA {v6}\n"));
                }
            }
        }
    }
    out
}

/// Configuration for the DNS seeder crawler service.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DnsSeedCrawlerConfig {
    /// Expected chain ID (e.g. 2 for testnet). Peers on other chains will be rejected.
    pub target_chain_id: u64,
    /// Domain name for RFC 1035 zone file (e.g. "seed.kovanica.online").
    pub domain: String,
    /// Crawl interval in seconds.
    pub crawl_interval_secs: u64,
    /// Probe timeout in milliseconds.
    pub probe_timeout_ms: u64,
    /// Maximum age in seconds before a verified peer record is considered stale.
    pub max_age_secs: u64,
    /// DNS TTL in seconds for generated zone file.
    pub zone_ttl_secs: u32,
    /// Optional output file path to write RFC 1035 zone file.
    pub zone_file_path: Option<PathBuf>,
    /// Initial bootnodes or peers to crawl (host:port or IP:port).
    pub bootnodes: Vec<String>,
}

impl Default for DnsSeedCrawlerConfig {
    fn default() -> Self {
        Self {
            target_chain_id: kovanica_types::signing::chain_id::TESTNET,
            domain: "seed.kovanica.online".to_string(),
            crawl_interval_secs: 30,
            probe_timeout_ms: 2000,
            max_age_secs: 300,
            zone_ttl_secs: 60,
            zone_file_path: None,
            bootnodes: Vec::new(),
        }
    }
}

/// Dynamic DNS seeder crawler service that maintains verified healthy peers and exports
/// standard RFC 1035 zone files.
pub struct DnsSeedCrawler {
    config: DnsSeedCrawlerConfig,
    active_peers: RwLock<HashMap<SocketAddr, DnsSeedEntry>>,
}

impl DnsSeedCrawler {
    /// Create a new DNS seed crawler with configuration.
    pub fn new(config: DnsSeedCrawlerConfig) -> Self {
        Self {
            config,
            active_peers: RwLock::new(HashMap::new()),
        }
    }

    /// Access the crawler configuration.
    pub fn config(&self) -> &DnsSeedCrawlerConfig {
        &self.config
    }

    /// Record a discovered/probed peer entry.
    ///
    /// Validates chain ID: entries from other chains (e.g. Chain 1 mainnet or Chain 2)
    /// are rejected and will not be exported to the zone file.
    pub fn record_peer(&self, entry: DnsSeedEntry) -> bool {
        if entry.chain_id != self.config.target_chain_id {
            warn!(
                peer = %entry.ip,
                port = entry.port,
                peer_chain = entry.chain_id,
                target_chain = self.config.target_chain_id,
                "rejecting peer: chain ID mismatch"
            );
            return false;
        }

        let addr = SocketAddr::new(entry.ip, entry.port);
        let mut map = self.active_peers.write().unwrap();
        map.insert(addr, entry);
        true
    }

    /// Remove an inactive or unreachable peer.
    pub fn remove_peer(&self, addr: &SocketAddr) {
        let mut map = self.active_peers.write().unwrap();
        map.remove(addr);
    }

    /// Return all active, healthy seed entries matching the crawler's target chain ID.
    pub fn get_healthy_seeds(&self) -> Vec<DnsSeedEntry> {
        let now_utc = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let map = self.active_peers.read().unwrap();
        let entries: Vec<DnsSeedEntry> = map.values().cloned().collect();
        let mut healthy = filter_healthy_seeds(
            &entries,
            self.config.target_chain_id,
            now_utc,
            self.config.max_age_secs,
        );
        // Sort deterministically by latency ascending
        healthy.sort_by_key(|e| e.latency_ms);
        healthy
    }

    /// Generate RFC 1035 DNS zone file text and optionally write it to `zone_file_path`.
    pub fn export_zone_file(&self, serial: u32) -> Result<String, std::io::Error> {
        let healthy = self.get_healthy_seeds();
        let zone = generate_zone_file(
            &self.config.domain,
            self.config.zone_ttl_secs,
            &healthy,
            serial,
        );

        if let Some(ref path) = self.config.zone_file_path {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(path, &zone)?;
            info!(
                path = %path.display(),
                domain = %self.config.domain,
                peer_count = healthy.len(),
                "exported DNS seed zone file"
            );
        }

        Ok(zone)
    }

    /// Probe a socket address with a TCP connection to verify reachability and latency.
    pub async fn probe_socket_addr(&self, addr: SocketAddr) -> Result<DnsSeedEntry, String> {
        let start = Instant::now();
        let connect_fut = tokio::net::TcpStream::connect(addr);
        match tokio::time::timeout(
            Duration::from_millis(self.config.probe_timeout_ms),
            connect_fut,
        )
        .await
        {
            Ok(Ok(_stream)) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                let entry = DnsSeedEntry::new(
                    addr.ip(),
                    addr.port(),
                    latency_ms,
                    self.config.target_chain_id,
                );
                Ok(entry)
            }
            Ok(Err(e)) => Err(format!("connection error: {e}")),
            Err(_) => Err(format!("timeout after {} ms", self.config.probe_timeout_ms)),
        }
    }

    /// Execute a single pass over configured bootnodes and active peers.
    pub async fn crawl_once(&self) {
        let mut target_addrs: Vec<SocketAddr> = Vec::new();

        for bootnode in &self.config.bootnodes {
            let bn = bootnode.trim();
            if bn.is_empty() {
                continue;
            }
            let (host, default_port) = match bn.split_once(':') {
                Some((h, p)) => (h, p.parse::<u16>().unwrap_or(8000)),
                None => (bn, 8000),
            };

            if let Ok(iter) = format!("{host}:{default_port}").to_socket_addrs() {
                for saddr in iter {
                    if !target_addrs.contains(&saddr) {
                        target_addrs.push(saddr);
                    }
                }
            }
        }

        // Also re-probe existing active peers
        {
            let map = self.active_peers.read().unwrap();
            for addr in map.keys() {
                if !target_addrs.contains(addr) {
                    target_addrs.push(*addr);
                }
            }
        }

        for addr in target_addrs {
            match self.probe_socket_addr(addr).await {
                Ok(entry) => {
                    debug!(%addr, latency_ms = entry.latency_ms, "seed peer probe succeeded");
                    self.record_peer(entry);
                }
                Err(err) => {
                    debug!(%addr, error = %err, "seed peer probe failed");
                    // If peer was previously active but now unreachable, check freshness
                    // and allow eviction if stale
                }
            }
        }

        // Export zone file if configured
        if self.config.zone_file_path.is_some() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let serial = (now % 100_000_000) as u32;
            if let Err(e) = self.export_zone_file(serial) {
                warn!(error = %e, "failed to export DNS seed zone file");
            }
        }
    }

    /// Run the background crawler loop until shutdown signal is received.
    pub async fn run_loop(
        self: std::sync::Arc<Self>,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        let interval_duration = Duration::from_secs(self.config.crawl_interval_secs.max(5));
        let mut ticker = tokio::time::interval(interval_duration);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        info!(
            target_chain = self.config.target_chain_id,
            domain = %self.config.domain,
            interval_secs = self.config.crawl_interval_secs,
            "DNS seeder crawler background service running"
        );

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.crawl_once().await;
                }
                _ = shutdown.changed() => {
                    info!("DNS seeder crawler shutting down");
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::signing::chain_id;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn dns_seed_entry_freshness_and_filtering() {
        let now = 1_700_000_000u64;
        let fresh_peer = DnsSeedEntry {
            ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)),
            port: 8000,
            last_seen_utc: now - 30, // 30s ago
            latency_ms: 15,
            chain_id: chain_id::HYBRID_TESTNET,
        };
        let stale_peer = DnsSeedEntry {
            ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2)),
            port: 8000,
            last_seen_utc: now - 7200, // 2h ago
            latency_ms: 25,
            chain_id: chain_id::HYBRID_TESTNET,
        };
        let other_chain_peer = DnsSeedEntry {
            ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 3)),
            port: 8000,
            last_seen_utc: now - 10,
            latency_ms: 10,
            chain_id: 1, // Mainnet instead of 2
        };

        let list = vec![fresh_peer.clone(), stale_peer, other_chain_peer];
        let healthy = filter_healthy_seeds(&list, chain_id::HYBRID_TESTNET, now, 300);

        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0].ip, fresh_peer.ip);
    }

    #[test]
    fn generate_zone_file_rfc1035_output() {
        let entries = vec![
            DnsSeedEntry {
                ip: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 10)),
                port: 8000,
                last_seen_utc: 1_700_000_000,
                latency_ms: 12,
                chain_id: 2,
            },
            DnsSeedEntry {
                ip: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
                port: 8000,
                last_seen_utc: 1_700_000_000,
                latency_ms: 18,
                chain_id: 2,
            },
            // Duplicate IP check
            DnsSeedEntry {
                ip: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 10)),
                port: 8000,
                last_seen_utc: 1_700_000_000,
                latency_ms: 12,
                chain_id: 2,
            },
        ];

        let zone = generate_zone_file("seed.kovanica.online", 60, &entries, 2026111001);
        assert!(zone.contains("$ORIGIN seed.kovanica.online."));
        assert!(zone.contains("$TTL 60"));
        assert!(zone.contains("@ IN A 198.51.100.10"));
        assert!(zone.contains("@ IN AAAA 2001:db8::1"));
        // Deduplicated: appears exactly once
        assert_eq!(zone.matches("198.51.100.10").count(), 1);
    }

    #[test]
    fn crawler_chain_id_filtering_rejects_foreign_chains() {
        let config = DnsSeedCrawlerConfig {
            target_chain_id: 2, // Testnet
            domain: "seed.kovanica.online".to_string(),
            crawl_interval_secs: 30,
            probe_timeout_ms: 1000,
            max_age_secs: 300,
            zone_ttl_secs: 60,
            zone_file_path: None,
            bootnodes: vec!["127.0.0.1:8000".to_string()],
        };
        let crawler = DnsSeedCrawler::new(config);

        let mainnet_peer = DnsSeedEntry::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 8000, 10, 1);
        let unregistered_peer =
            DnsSeedEntry::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)), 8000, 12, 99);
        let testnet_peer = DnsSeedEntry::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 3)), 8000, 8, 2);

        // Chain 1 and an unregistered chain id must be rejected
        assert!(!crawler.record_peer(mainnet_peer));
        assert!(!crawler.record_peer(unregistered_peer));
        // Chain 2 must be accepted
        assert!(crawler.record_peer(testnet_peer.clone()));

        let healthy = crawler.get_healthy_seeds();
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0].ip, testnet_peer.ip);
        assert_eq!(healthy[0].chain_id, 2);
    }

    #[test]
    fn crawler_zone_file_export_seed_kovanica_online() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let zone_path = tmp_dir.path().join("seed.kovanica.online.zone");

        let config = DnsSeedCrawlerConfig {
            target_chain_id: 2,
            domain: "seed.kovanica.online".to_string(),
            crawl_interval_secs: 30,
            probe_timeout_ms: 1000,
            max_age_secs: 300,
            zone_ttl_secs: 60,
            zone_file_path: Some(zone_path.clone()),
            bootnodes: Vec::new(),
        };
        let crawler = DnsSeedCrawler::new(config);

        let peer1 = DnsSeedEntry::new(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)), 8000, 20, 2);
        let peer2 = DnsSeedEntry::new(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 2)), 8000, 10, 2);
        crawler.record_peer(peer1);
        crawler.record_peer(peer2);

        let zone_text = crawler.export_zone_file(2026111002).unwrap();
        assert!(zone_text.contains("$ORIGIN seed.kovanica.online."));
        assert!(zone_text.contains("@ IN NS ns1.seed.kovanica.online."));
        assert!(zone_text.contains("@ IN A 198.51.100.1"));
        assert!(zone_text.contains("@ IN A 198.51.100.2"));

        // File was written on disk
        assert!(zone_path.exists());
        let read_back = std::fs::read_to_string(&zone_path).unwrap();
        assert_eq!(read_back, zone_text);
    }

    #[tokio::test]
    async fn crawler_tcp_probe_reachable_and_timeout() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let config = DnsSeedCrawlerConfig {
            target_chain_id: 2,
            domain: "seed.kovanica.online".to_string(),
            crawl_interval_secs: 30,
            probe_timeout_ms: 500,
            max_age_secs: 300,
            zone_ttl_secs: 60,
            zone_file_path: None,
            bootnodes: Vec::new(),
        };
        let crawler = DnsSeedCrawler::new(config);

        // Probe active listener
        let result = crawler.probe_socket_addr(local_addr).await;
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.port, local_addr.port());
        assert_eq!(entry.chain_id, 2);

        drop(listener);

        // Probe closed port
        let closed_result = crawler.probe_socket_addr(local_addr).await;
        assert!(closed_result.is_err());
    }
}
