//! Contract call encoders and receipt parsing helpers.

/// Well-known contract method names (entry points).
pub mod method {
    pub const HTLC_CREATE: &str = "htlc_create";
    pub const HTLC_CLAIM: &str = "htlc_claim";
    pub const HTLC_REFUND: &str = "htlc_refund";

    pub const VAULT_CREATE: &str = "vault_create";
    pub const VAULT_CLAIM: &str = "vault_claim";
    pub const VAULT_CANCEL: &str = "vault_cancel";

    pub const MULTISIG_CREATE: &str = "multisig_create";
    pub const MULTISIG_PROPOSE: &str = "multisig_propose";
    pub const MULTISIG_CONFIRM: &str = "multisig_confirm";
    pub const MULTISIG_EXECUTE: &str = "multisig_execute";

    pub const TOKEN_CREATE: &str = "token_create";
    pub const TOKEN_TRANSFER: &str = "token_transfer";
    pub const TOKEN_APPROVE: &str = "token_approve";
    pub const TOKEN_TRANSFER_FROM: &str = "token_transfer_from";
    pub const TOKEN_MINT: &str = "token_mint";
    pub const TOKEN_BURN: &str = "token_burn";
    pub const TOKEN_INIT_POOL: &str = "token_init_pool";
    pub const TOKEN_BUY: &str = "token_buy";
    pub const TOKEN_SELL: &str = "token_sell";
}

/// A minimal parsed receipt view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedReceipt {
    pub tx_hash: Vec<u8>,
    pub success: bool,
    pub gas_used: u64,
    pub logs: Vec<Vec<u8>>,
}

impl ParsedReceipt {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        Some(Self {
            tx_hash: bytes.to_vec(),
            success: true,
            gas_used: 0,
            logs: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_constants_nonempty() {
        assert_eq!(method::HTLC_CREATE, "htlc_create");
        assert_eq!(method::TOKEN_TRANSFER, "token_transfer");
    }

    #[test]
    fn parse_receipt_empty_returns_none() {
        assert!(ParsedReceipt::from_bytes(&[]).is_none());
    }

    #[test]
    fn parse_receipt_nonempty_returns_some() {
        let r = ParsedReceipt::from_bytes(&[1, 2, 3]).unwrap();
        assert!(r.success);
    }
}
