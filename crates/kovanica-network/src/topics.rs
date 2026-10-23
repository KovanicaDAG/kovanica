//! Gossipsub base topic names shared by the whole crate.
//!
//! On the wire every topic is scoped to the chain:
//! `kovanica/<chain_id>-<genesis prefix>/<name>` ([`NetworkId::topic`]).

use crate::NetworkId;
use libp2p::gossipsub::IdentTopic;

/// Topic carrying full [`kovanica_types::block::StatementBlock`]s.
pub const BLOCKS: &str = "blocks";

/// Topic carrying broadcast [`kovanica_types::transaction::Transaction`]s.
pub const TRANSACTIONS: &str = "transactions";

/// Topic carrying votes for the consensus layer.
pub const VOTES: &str = "votes";

/// Topic carrying sync-range requests.
pub const SYNC: &str = "sync";

/// Topic carrying state sync requests (fast sync).
pub const STATE_SYNC: &str = "state_sync";

/// Topic carrying post-execution commit seals.
///
/// Consensus does not wait on this topic. Nodes that do not subscribe
/// still commit. The topic is not peer-scored.
pub const SEALS: &str = "seals";

/// Every topic subscribed to at startup, in subscription order.
pub const ALL: [&str; 6] = [BLOCKS, TRANSACTIONS, VOTES, SYNC, STATE_SYNC, SEALS];

/// Topic handle for a base topic name on `network`
/// (`kovanica/<tag>/<name>`, see [`NetworkId::topic`]).
pub fn ident(network: &NetworkId, name: &str) -> IdentTopic {
    IdentTopic::new(network.topic(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_topics_are_unique() {
        let mut seen = HashSet::new();
        for name in ALL {
            assert!(seen.insert(name), "duplicate topic name: {name}");
        }
        assert_eq!(ALL.len(), 6);
    }

    #[test]
    fn topic_handles_round_trip_to_their_names() {
        let network = NetworkId::default();
        for name in ALL {
            let hash = ident(&network, name).hash();
            assert_eq!(network.topic_kind(hash.as_str()), Some(name));
        }
    }
}
