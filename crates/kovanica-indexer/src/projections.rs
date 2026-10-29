//! Optional projections for token/NFT events.

pub const TOKEN_TOPICS: &[&str] = &["token_created", "transfer", "approval", "mint", "burn"];

pub fn is_token_topic(topic: &str) -> bool {
    TOKEN_TOPICS.contains(&topic)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_topics_recognized() {
        assert!(is_token_topic("transfer"));
        assert!(!is_token_topic("htlc_created"));
    }
}
