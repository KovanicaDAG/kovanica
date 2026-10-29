use std::sync::Arc;

pub trait BlockSource {
    fn fetch_blocks(&self, from: u64, to: u64) -> Result<Vec<Vec<u8>>, String>;
}

pub trait StateSnapshotSource {
    fn fetch_snapshot(&self) -> Result<Vec<u8>, String>;
}

pub struct SyncClient {
    pub block_source: Arc<dyn BlockSource>,
    pub snapshot_source: Arc<dyn StateSnapshotSource>,
}

impl SyncClient {
    pub fn catch_up<F>(&self, target_height: u64, mut progress: F) -> Result<(), String>
    where F: FnMut(u64) {
        let mut h = 0u64;
        while h < target_height {
            let batch = self.block_source.fetch_blocks(h + 1, target_height.min(h + 10))?;
            if batch.is_empty() { return Err("no blocks".into()); }
            h += batch.len() as u64;
            progress(h);
        }
        Ok(())
    }
    pub fn fetch_snapshot(&self) -> Result<Vec<u8>, String> {
        self.snapshot_source.fetch_snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBlocks { data: Vec<Vec<u8>> }
    impl BlockSource for MockBlocks {
        fn fetch_blocks(&self, from: u64, to: u64) -> Result<Vec<Vec<u8>>, String> {
            Ok(self.data.iter().skip((from-1) as usize).take((to-from+1) as usize).cloned().collect())
        }
    }
    struct MockSnap;
    impl StateSnapshotSource for MockSnap {
        fn fetch_snapshot(&self) -> Result<Vec<u8>, String> { Ok(vec![1,2,3]) }
    }

    #[test]
    fn catch_up_progress() {
        let src = Arc::new(MockBlocks { data: vec![vec![1],vec![2],vec![3]] });
        let snap = Arc::new(MockSnap);
        let client = SyncClient { block_source: src, snapshot_source: snap };
        let mut seen = Vec::new();
        client.catch_up(3, |h| seen.push(h)).unwrap();
        assert_eq!(seen, vec![3]);
        assert_eq!(client.fetch_snapshot().unwrap(), vec![1,2,3]);
    }
}
