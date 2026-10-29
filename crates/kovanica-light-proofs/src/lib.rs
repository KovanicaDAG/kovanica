use blake3::Hasher;

#[derive(Clone, Debug)]
pub struct InclusionProof {
    pub leaf: Vec<u8>,
    pub siblings: Vec<Vec<u8>>,
}

pub fn hash_leaf(data: &[u8]) -> [u8; 32] {
    let mut h = Hasher::new();
    h.update(data);
    *h.finalize().as_bytes()
}

pub fn verify(proof: &InclusionProof, root: &[u8; 32]) -> bool {
    let mut cur = hash_leaf(&proof.leaf);
    for sib in &proof.siblings {
        let mut h = Hasher::new();
        if cur <= *sib.as_slice().try_into().unwrap_or(&[0u8;32]) {
            h.update(&cur);
            h.update(sib);
        } else {
            h.update(sib);
            h.update(&cur);
        }
        cur = *h.finalize().as_bytes();
    }
    &cur == root
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn honest() {
        let leaf = b"abc";
        let lh = hash_leaf(leaf);
        let sibling = [9u8;32];
        let mut h = Hasher::new();
        if lh <= sibling {
            h.update(&lh);
            h.update(&sibling);
        } else {
            h.update(&sibling);
            h.update(&lh);
        }
        let root = *h.finalize().as_bytes();
        let proof = InclusionProof { leaf: leaf.to_vec(), siblings: vec![sibling.to_vec()] };
        assert!(verify(&proof, &root));
    }
    #[test]
    fn tampered() {
        let leaf = b"abc";
        let proof = InclusionProof { leaf: leaf.to_vec(), siblings: vec![vec![9u8;32]] };
        assert!(!verify(&proof, &[0u8;32]));
    }
}
