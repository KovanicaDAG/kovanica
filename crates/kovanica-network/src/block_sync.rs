//! Block sync request-response protocol (`/kovanica/block-sync/1.1.0`).
//!
//! Uses libp2p's request-response behaviour to fetch blocks by hash or by
//! (author, round) pair, plus two bounded catch-up messages:
//!
//! * `Status` asks a peer for its current DAG tip round. The answer is only a
//!   *trigger* to start fetching; it is never trusted for validation.
//! * `RoundRange` asks for the block hashes of a bounded window of rounds so
//!   a node that is behind can fetch the missing range (each hash is then
//!   re-fetched via `ByHash` and validated as usual).
//!
//! All variants are appended after the original ones, so the encoding of the
//! pre-existing variants is unchanged.
//!
//! # Compatibility
//!
//! `Status` and `RoundRange` are new, so the protocol version was bumped from
//! `1.0.0` to `1.1.0`. Both names are advertised (see
//! [`block_sync_protocols`]), most-preferred first: a peer that speaks `1.1.0`
//! negotiates it and can use the catch-up messages, while a peer that only
//! understands `1.0.0` negotiates the legacy name. An old peer that receives a
//! `1.1.0`-only message over the `1.0.0` name cannot decode it and the request
//! fails at the transport layer; the requester's `retry_peer` budget makes
//! that fail *bounded* (three sends, then give up) and it is never counted
//! against the peer (see `NetworkService::handle_request_response_event`).

use futures::{AsyncReadExt, AsyncWriteExt};
use kovanica_types::{block::StatementBlock, hash::Hash, AuthorityIndex, Round};
use libp2p::request_response::{OutboundRequestId, ProtocolSupport, ResponseChannel};
use serde::{Deserialize, Serialize};

/// Protocol name for block sync request-response.
///
/// `1.1.0` adds the `Status`/`RoundRange` catch-up messages; see the module
/// docs for how it is negotiated alongside [`BLOCK_SYNC_PROTOCOL_LEGACY`].
pub const BLOCK_SYNC_PROTOCOL: &str = "/kovanica/block-sync/1.1.0";

/// Legacy protocol name (`1.0.0`), kept so peers that predate the catch-up
/// messages can still be dialed for `ByHash`/`ByAuthorRound` sync.
pub const BLOCK_SYNC_PROTOCOL_LEGACY: &str = "/kovanica/block-sync/1.0.0";

/// Maximum response size (1 MiB): one block, same cap as a gossip frame.
pub const MAX_RESPONSE_SIZE: usize = 1024 * 1024;

/// Maximum request size. Requests are a hash or an (author, round) pair, so
/// anything larger than this is garbage.
pub const MAX_REQUEST_SIZE: usize = 1024;

/// Read the rest of a request-response stream, failing once more than
/// `limit` bytes arrive instead of buffering an arbitrarily large message
/// from a hostile peer.
pub(crate) async fn read_bounded<T>(io: &mut T, limit: usize) -> std::io::Result<Vec<u8>>
where
    T: AsyncReadExt + Unpin + Send,
{
    let mut buf = Vec::new();
    // Read one byte past the limit so "exactly at the limit" is accepted and
    // "over the limit" is detected without reading the whole stream.
    io.take(limit as u64 + 1).read_to_end(&mut buf).await?;
    if buf.len() > limit {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("message exceeds {limit} byte limit"),
        ));
    }
    Ok(buf)
}

/// Block sync request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockSyncRequest {
    /// Request by block hash.
    ByHash(Hash),
    /// Request by author and round.
    ByAuthorRound {
        author: AuthorityIndex,
        round: Round,
    },
    /// Ask for the peer's current DAG tip round (proactive catch-up trigger).
    Status,
    /// Request the block hashes of rounds `from..=to` (inclusive), bounded by
    /// [`BlockSyncRequest::MAX_RANGE_ROUNDS`] rounds per request.
    RoundRange {
        /// First round of the window (inclusive).
        from: Round,
        /// Last round of the window (inclusive).
        to: Round,
    },
}

impl BlockSyncRequest {
    /// Maximum number of rounds covered by one [`BlockSyncRequest::RoundRange`]
    /// window. Bounds the storage reads a single request can cost the peer.
    pub const MAX_RANGE_ROUNDS: Round = 32;
}

/// Block sync response — includes missing blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockSyncResponse {
    /// The requested block.
    Block(StatementBlock),
    /// Block not found.
    NotFound,
    /// Request invalid.
    InvalidRequest,
    /// List of missing parent blocks required before the requested block.
    MissingBlocks(Vec<Hash>),
    /// Block hashes for rounds `..=up_to_round` in ascending round order,
    /// bounded by [`BlockSyncResponse::MAX_RANGE_HASHES`] hashes.
    ///
    /// `up_to_round` may be lower than the requested window end when the hash
    /// cap was reached; it is the round the requester continues from.
    RoundHashes {
        /// Highest round fully covered by `hashes`.
        up_to_round: Round,
        /// Block digests, grouped by ascending round.
        hashes: Vec<Hash>,
    },
    /// The responder's current DAG tip round (catch-up trigger only).
    Status {
        /// Advertised tip round; never trusted for validation.
        tip_round: Round,
    },
}

impl BlockSyncResponse {
    /// Maximum number of hashes in one [`BlockSyncResponse::RoundHashes`]
    /// response (bounds the response size well under the 1 MiB frame cap).
    pub const MAX_RANGE_HASHES: usize = 512;
}

impl std::fmt::Display for BlockSyncResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockSyncResponse::Block(_) => write!(f, "Block"),
            BlockSyncResponse::NotFound => write!(f, "NotFound"),
            BlockSyncResponse::InvalidRequest => write!(f, "InvalidRequest"),
            BlockSyncResponse::MissingBlocks(_) => write!(f, "MissingBlocks"),
            BlockSyncResponse::RoundHashes { up_to_round, .. } => {
                write!(f, "RoundHashes(up_to={up_to_round})")
            }
            BlockSyncResponse::Status { tip_round } => write!(f, "Status(tip={tip_round})"),
        }
    }
}

/// Codec for the block sync protocol.
#[derive(Debug, Clone, Default)]
pub struct BlockSyncCodec;

impl libp2p::request_response::Codec for BlockSyncCodec {
    type Protocol = libp2p::StreamProtocol;
    type Request = BlockSyncRequest;
    type Response = BlockSyncResponse;

    async fn read_request<T>(
        &mut self,
        _: &libp2p::StreamProtocol,
        io: &mut T,
    ) -> Result<Self::Request, std::io::Error>
    where
        T: AsyncReadExt + Unpin + Send,
    {
        let buf = read_bounded(io, MAX_REQUEST_SIZE).await?;
        bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _: &libp2p::StreamProtocol,
        io: &mut T,
    ) -> Result<Self::Response, std::io::Error>
    where
        T: AsyncReadExt + Unpin + Send,
    {
        let buf = read_bounded(io, MAX_RESPONSE_SIZE).await?;
        bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _: &libp2p::StreamProtocol,
        io: &mut T,
        req: Self::Request,
    ) -> Result<(), std::io::Error>
    where
        T: AsyncWriteExt + Unpin + Send,
    {
        let data = bincode::serialize(&req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await
    }

    async fn write_response<T>(
        &mut self,
        _: &libp2p::StreamProtocol,
        io: &mut T,
        resp: Self::Response,
    ) -> Result<(), std::io::Error>
    where
        T: AsyncWriteExt + Unpin + Send,
    {
        let data = bincode::serialize(&resp)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await
    }
}

/// Protocol config.
pub fn block_sync_protocol() -> libp2p::request_response::Config {
    libp2p::request_response::Config::default()
        .with_request_timeout(std::time::Duration::from_secs(30))
        .with_max_concurrent_streams(100)
}

/// Protocols advertised for block sync, most-preferred first.
///
/// `1.1.0` (catch-up aware) is offered first, so two current nodes always
/// negotiate it; a peer that only understands `1.0.0` falls back to the legacy
/// name instead of being dropped. Both names share [`BlockSyncCodec`].
pub fn block_sync_protocols() -> [(libp2p::StreamProtocol, ProtocolSupport); 2] {
    [
        (
            libp2p::StreamProtocol::new(BLOCK_SYNC_PROTOCOL),
            ProtocolSupport::Full,
        ),
        (
            libp2p::StreamProtocol::new(BLOCK_SYNC_PROTOCOL_LEGACY),
            ProtocolSupport::Full,
        ),
    ]
}

/// Request event.
#[derive(Debug)]
pub struct BlockSyncRequestEvent {
    pub peer: libp2p::PeerId,
    pub request: BlockSyncRequest,
    pub channel: ResponseChannel<BlockSyncResponse>,
}

/// Response event.
#[derive(Debug, Clone)]
pub struct BlockSyncResponseEvent {
    pub peer: libp2p::PeerId,
    pub request_id: OutboundRequestId,
    pub response: BlockSyncResponse,
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::hash::Hash;

    #[test]
    fn bounded_read_accepts_up_to_the_limit_and_rejects_beyond() {
        futures::executor::block_on(async {
            let mut exact = futures::io::Cursor::new(vec![1u8; 16]);
            assert_eq!(
                read_bounded(&mut exact, 16).await.expect("at limit").len(),
                16
            );
            let mut over = futures::io::Cursor::new(vec![1u8; 17]);
            let err = read_bounded(&mut over, 16).await.expect_err("over limit");
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        });
    }

    #[test]
    fn oversized_block_sync_response_is_rejected_by_the_codec() {
        use libp2p::request_response::Codec;
        futures::executor::block_on(async {
            let mut codec = BlockSyncCodec;
            let protocol = libp2p::StreamProtocol::new(BLOCK_SYNC_PROTOCOL);
            let mut io = futures::io::Cursor::new(vec![0u8; MAX_RESPONSE_SIZE + 1]);
            assert!(codec.read_response(&protocol, &mut io).await.is_err());
            let mut io = futures::io::Cursor::new(vec![0u8; MAX_REQUEST_SIZE + 1]);
            assert!(codec.read_request(&protocol, &mut io).await.is_err());
        });
    }

    #[test]
    fn block_sync_request_by_hash_round_trip() {
        let hash = Hash::new("test-hash");
        let req = BlockSyncRequest::ByHash(hash);
        let encoded = bincode::serialize(&req).expect("serializes");
        let decoded: BlockSyncRequest = bincode::deserialize(&encoded).expect("deserializes");
        assert_eq!(req, decoded);
    }

    #[test]
    fn block_sync_request_by_author_round_round_trip() {
        let req = BlockSyncRequest::ByAuthorRound {
            author: 5,
            round: 42,
        };
        let encoded = bincode::serialize(&req).expect("serializes");
        let decoded: BlockSyncRequest = bincode::deserialize(&encoded).expect("deserializes");
        assert_eq!(req, decoded);
    }

    #[test]
    fn block_sync_response_round_trip() {
        let resp = BlockSyncResponse::NotFound;
        let encoded = bincode::serialize(&resp).expect("serializes");
        let decoded: BlockSyncResponse = bincode::deserialize(&encoded).expect("deserializes");
        assert_eq!(resp, decoded);
    }

    #[test]
    fn block_sync_response_missing_blocks_round_trip() {
        let resp = BlockSyncResponse::MissingBlocks(vec![Hash::new("a"), Hash::new("b")]);
        let encoded = bincode::serialize(&resp).expect("serializes");
        let decoded: BlockSyncResponse = bincode::deserialize(&encoded).expect("deserializes");
        assert_eq!(resp, decoded);
    }

    #[test]
    fn block_sync_status_round_trip() {
        let req = BlockSyncRequest::Status;
        let encoded = bincode::serialize(&req).expect("serializes");
        let decoded: BlockSyncRequest = bincode::deserialize(&encoded).expect("deserializes");
        assert_eq!(req, decoded);

        let resp = BlockSyncResponse::Status { tip_round: 7 };
        let encoded = bincode::serialize(&resp).expect("serializes");
        let decoded: BlockSyncResponse = bincode::deserialize(&encoded).expect("deserializes");
        assert_eq!(resp, decoded);
        assert_eq!(resp.to_string(), "Status(tip=7)");
    }

    #[test]
    fn block_sync_round_range_round_trip() {
        let req = BlockSyncRequest::RoundRange { from: 2, to: 9 };
        let encoded = bincode::serialize(&req).expect("serializes");
        let decoded: BlockSyncRequest = bincode::deserialize(&encoded).expect("deserializes");
        assert_eq!(req, decoded);

        let resp = BlockSyncResponse::RoundHashes {
            up_to_round: 9,
            hashes: vec![Hash::new("a"), Hash::new("b")],
        };
        let encoded = bincode::serialize(&resp).expect("serializes");
        let decoded: BlockSyncResponse = bincode::deserialize(&encoded).expect("deserializes");
        assert_eq!(resp, decoded);
        assert_eq!(resp.to_string(), "RoundHashes(up_to=9)");
    }

    /// Catch-up bounds must fit comfortably inside the framing caps: a range
    /// request is tiny and a range response stays far below the 1 MiB frame.
    #[test]
    fn catch_up_bounds_fit_the_frame_caps() {
        let worst_request = BlockSyncRequest::RoundRange {
            from: 0,
            to: BlockSyncRequest::MAX_RANGE_ROUNDS - 1,
        };
        let encoded = bincode::serialize(&worst_request).expect("serializes");
        assert!(encoded.len() <= MAX_REQUEST_SIZE);

        let worst_hashes = vec![Hash::zero(); BlockSyncResponse::MAX_RANGE_HASHES];
        let worst_response = BlockSyncResponse::RoundHashes {
            up_to_round: u64::MAX,
            hashes: worst_hashes,
        };
        let encoded = bincode::serialize(&worst_response).expect("serializes");
        assert!(encoded.len() <= MAX_RESPONSE_SIZE);
    }

    /// The catch-up protocol is offered first, and the legacy name stays
    /// available so old peers are not dropped (compat, audit MEDIUM-6).
    #[test]
    fn block_sync_advertises_new_and_legacy_protocols_in_preference_order() {
        assert_eq!(BLOCK_SYNC_PROTOCOL, "/kovanica/block-sync/1.1.0");
        assert_eq!(BLOCK_SYNC_PROTOCOL_LEGACY, "/kovanica/block-sync/1.0.0");
        let protocols = block_sync_protocols();
        assert_eq!(protocols[0].0.as_ref(), BLOCK_SYNC_PROTOCOL);
        assert_eq!(protocols[1].0.as_ref(), BLOCK_SYNC_PROTOCOL_LEGACY);
        assert!(protocols[0].1.inbound() && protocols[0].1.outbound());
        assert!(protocols[1].1.inbound() && protocols[1].1.outbound());
    }
}
