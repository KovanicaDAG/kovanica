use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Feed {
    pub description: String,
    pub decimals: u8,
    pub min_publishers: u8,
    pub price: u64,
    pub timestamp: u64,
    pub publishers: Vec<[u8;32]>,
}

#[derive(Default)]
pub struct Oracle {
    pub feeds: BTreeMap<u64, Feed>,
    next_id: u64,
}

impl Oracle {
    pub fn create_feed(&mut self, description: String, decimals: u8, min_publishers: u8) -> u64 {
        let id = self.next_id; self.next_id += 1;
        self.feeds.insert(id, Feed { description, decimals, min_publishers, price: 0, timestamp: 0, publishers: vec![] });
        id
    }
    pub fn set_publisher(&mut self, feed_id: u64, publisher: [u8;32]) {
        if let Some(f) = self.feeds.get_mut(&feed_id) { if !f.publishers.contains(&publisher) { f.publishers.push(publisher); } }
    }
    pub fn publish(&mut self, feed_id: u64, publisher: [u8;32], price: u64, timestamp: u64) -> bool {
        if let Some(f) = self.feeds.get_mut(&feed_id) {
            if f.publishers.contains(&publisher) {
                f.price = price; f.timestamp = timestamp; return true;
            }
        }
        false
    }
    pub fn get_price(&self, feed_id: u64) -> Option<(u64,u64)> {
        self.feeds.get(&feed_id).map(|f| (f.price, f.timestamp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn publish_read() {
        let mut o = Oracle::default();
        let id = o.create_feed("ETH/USD".into(), 8, 1);
        let publ = [1u8;32];
        o.set_publisher(id, publ);
        assert!(o.publish(id, publ, 12345, 100));
        assert_eq!(o.get_price(id), Some((12345,100)));
        assert!(!o.publish(id, [2u8;32], 1,1));
    }
}
