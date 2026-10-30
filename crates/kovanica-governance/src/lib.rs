use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Active,
    Passed,
    Rejected,
}

#[derive(Clone, Debug)]
pub struct Proposal {
    pub description: String,
    pub payload_hash: [u8; 32],
    pub voting_end: u64,
    pub yes: u64,
    pub no: u64,
    pub status: Status,
}

#[derive(Default)]
pub struct Governance {
    pub proposals: BTreeMap<u64, Proposal>,
    next_id: u64,
}

impl Governance {
    pub fn propose(&mut self, description: String, payload_hash: [u8; 32], voting_end: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.proposals.insert(
            id,
            Proposal {
                description,
                payload_hash,
                voting_end,
                yes: 0,
                no: 0,
                status: Status::Active,
            },
        );
        id
    }
    pub fn vote(&mut self, id: u64, support: bool, now: u64) -> bool {
        if let Some(p) = self.proposals.get_mut(&id) {
            if p.status == Status::Active && now <= p.voting_end {
                if support {
                    p.yes += 1;
                } else {
                    p.no += 1;
                }
                return true;
            }
        }
        false
    }
    pub fn finalize(&mut self, id: u64, now: u64) -> Option<Status> {
        if let Some(p) = self.proposals.get_mut(&id) {
            if now >= p.voting_end && p.status == Status::Active {
                p.status = if p.yes > p.no {
                    Status::Passed
                } else {
                    Status::Rejected
                };
            }
            return Some(p.status.clone());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flow() {
        let mut g = Governance::default();
        let id = g.propose("param".into(), [0u8; 32], 10);
        assert!(g.vote(id, true, 5));
        assert_eq!(g.finalize(id, 10), Some(Status::Passed));
    }
}
