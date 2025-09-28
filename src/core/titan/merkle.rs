use sha2::{Digest, Sha256};

pub fn merkle_root(leaves: &[[u8;32]]) -> [u8;32] {
    if leaves.is_empty() { return [0u8;32]; }
    let mut layer: Vec<[u8;32]> = leaves.to_vec();
    while layer.len() > 1 {
        if layer.len() % 2 == 1 {
            // duplicate last for odd count (Bitcoin-style)
            layer.push(*layer.last().unwrap());
        }
        let mut next = Vec::with_capacity(layer.len()/2);
        for pair in layer.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(&pair[0]);
            hasher.update(&pair[1]);
            let h = hasher.finalize();
            let mut out = [0u8;32];
            out.copy_from_slice(&h);
            next.push(out);
        }
        layer = next;
    }
    layer[0]
}

/// Verify a leaf against a Merkle root with its authentication path.
/// `index` is the leaf index at the bottom.
/// `path` holds sibling hashes from leaf level up (left/right inferred by index bit).
pub fn merkle_verify(leaf: [u8;32], index: usize, path: &[[u8;32]], root: [u8;32]) -> bool {
    let mut acc = leaf;
    let mut idx = index;
    for sib in path {
        let mut hasher = Sha256::new();
        if idx & 1 == 0 {
            hasher.update(&acc);
            hasher.update(sib);
        } else {
            hasher.update(sib);
            hasher.update(&acc);
        }
        let h = hasher.finalize();
        acc.copy_from_slice(&h);
        idx >>= 1;
    }
    acc == root
}
