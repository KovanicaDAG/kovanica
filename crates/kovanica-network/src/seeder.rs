//! DNS seed discovery, crawling and zone file generation for Kovanica P2P network.
//!
//! Provides utilities to track active healthy nodes on the network and format
//! them into standard DNS A/AAAA zone records or queryable seed lists.

use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn dns_seed_entry_freshness_and_filtering() {
        let now = 1_700_000_000u64;
        let fresh_peer = DnsSeedEntry {
            ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)),
            port: 8000,
            last_seen_utc: now - 30, // 30s ago
            latency_ms: 15,
            chain_id: 5,
        };
        let stale_peer = DnsSeedEntry {
            ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2)),
            port: 8000,
            last_seen_utc: now - 7200, // 2h ago
            latency_ms: 25,
            chain_id: 5,
        };
        let other_chain_peer = DnsSeedEntry {
            ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 3)),
            port: 8000,
            last_seen_utc: now - 10,
            latency_ms: 10,
            chain_id: 1, // Mainnet instead of 5
        };

        let list = vec![fresh_peer.clone(), stale_peer, other_chain_peer];
        let healthy = filter_healthy_seeds(&list, 5, now, 300);

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
                chain_id: 5,
            },
            DnsSeedEntry {
                ip: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
                port: 8000,
                last_seen_utc: 1_700_000_000,
                latency_ms: 18,
                chain_id: 5,
            },
            // Duplicate IP check
            DnsSeedEntry {
                ip: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 10)),
                port: 8000,
                last_seen_utc: 1_700_000_000,
                latency_ms: 12,
                chain_id: 5,
            },
        ];

        let zone = generate_zone_file("seed.kovanica.io", 60, &entries, 2026111001);
        assert!(zone.contains("$ORIGIN seed.kovanica.io."));
        assert!(zone.contains("$TTL 60"));
        assert!(zone.contains("@ IN A 198.51.100.10"));
        assert!(zone.contains("@ IN AAAA 2001:db8::1"));
        // Deduplicated: appears exactly once
        assert_eq!(zone.matches("198.51.100.10").count(), 1);
    }
}
