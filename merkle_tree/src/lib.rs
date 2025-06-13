use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq)]
pub struct MerkleTree {
    leaves: Vec<Vec<u8>>,
    root: Option<Vec<u8>>,
    tree: Vec<Vec<Vec<u8>>>,
}

#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub proof: Vec<(Vec<u8>, bool)>, // (hash, is_right)
    pub leaf_index: usize,
}

impl MerkleTree {
    pub fn new(data: Vec<Vec<u8>>) -> Self {
        if data.is_empty() {
            return Self {
                leaves: vec![],
                root: None,
                tree: vec![],
            };
        }

        let mut tree = MerkleTree {
            leaves: data.clone(),
            root: None,
            tree: vec![],
        };
        tree.build_tree();
        tree
    }

    fn hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    fn combine_hashes(left: &[u8], right: &[u8]) -> Vec<u8> {
        let mut combined = Vec::new();
        combined.extend_from_slice(left);
        combined.extend_from_slice(right);
        Self::hash(&combined)
    }

    fn build_tree(&mut self) {
        if self.leaves.is_empty() {
            return;
        }
        
        // Hash all leaves
        let mut current_level: Vec<Vec<u8>> = self.leaves
            .iter()
            .map(|leaf| Self::hash(leaf))
            .collect();
        
        self.tree.push(current_level.clone());
        
        // Build tree bottom-up
        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i + 1]
                } else {
                    left // Duplicate if odd number of nodes
                };

                next_level.push(Self::combine_hashes(left, right));
            }

            self.tree.push(next_level.clone());
            current_level = next_level;
        }

        self.root = current_level.into_iter().next();
    }

    pub fn root(&self) -> Option<&Vec<u8>> {
        self.root.as_ref()
    }

    pub fn get_proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() || self.tree.is_empty() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_index = index;

        // Traverse from leaf to root
        for level in 0..self.tree.len() - 1 {
            let is_right = current_index % 2 == 1;
            let sibling_index = if is_right {
                current_index - 1
            } else {
                current_index + 1
            };

            if sibling_index < self.tree[level].len() {
                proof.push((self.tree[level][sibling_index].clone(), !is_right));
            }

            current_index /= 2;
        }

        Some(MerkleProof {
            proof,
            leaf_index: index,
        })
    }

    pub fn verify_proof(&self, proof: &MerkleProof, leaf_data: &[u8]) -> bool {
        if proof.leaf_index >= self.leaves.len() {
            return false;
        }

        let mut current_hash = Self::hash(leaf_data);

        for (sibling_hash, is_right) in &proof.proof {
            current_hash = if *is_right {
                Self::combine_hashes(&current_hash, sibling_hash)
            } else {
                Self::combine_hashes(sibling_hash, &current_hash)
            };
        }

        Some(&current_hash) == self.root.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new(vec![]);
        assert!(tree.root().is_none());
    }

    #[test]
    fn test_single_leaf() {
        let data = vec![b"hello".to_vec()];
        let tree = MerkleTree::new(data);
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_multiple_leaves() {
        let data = vec![
            b"hello".to_vec(),
            b"world".to_vec(),
            b"foo".to_vec(),
            b"bar".to_vec(),
        ];
        let tree = MerkleTree::new(data);
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_proof_generation_and_verification() {
        let data = vec![
            b"hello".to_vec(),
            b"world".to_vec(),
            b"foo".to_vec(),
            b"bar".to_vec(),
        ];
        let tree = MerkleTree::new(data.clone());

        for (i, leaf) in data.iter().enumerate() {
            let proof = tree.get_proof(i).unwrap();
            assert!(tree.verify_proof(&proof, leaf));
        }
    }

    #[test]
    fn test_invalid_proof() {
        let data = vec![
            b"hello".to_vec(),
            b"world".to_vec(),
        ];
        let tree = MerkleTree::new(data);

        let proof = tree.get_proof(0).unwrap();
        assert!(!tree.verify_proof(&proof, b"invalid"));
    }
}