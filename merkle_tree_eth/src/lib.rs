use tiny_keccak::{Hasher, Keccak};

pub struct MerkleTree{
    pub layers: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    pub fn new(leaves: Vec<[u8; 32]>) -> Self {
        println!("leaves length: {}", leaves.len());
        let mut layers = vec![leaves.clone()];
        println!("layers length: {}", layers.len());
        while layers.last().unwrap().len() > 1 {
            let current = layers.last().unwrap();
            let mut next = vec![];
            for chunk in current.chunks(2) {
                let left = chunk[0];
                let right = if chunk.len() > 1 {
                    chunk[1]
                } else {
                    left
                };
                let mut hasher = Keccak::v256();
                hasher.update(&left);
                hasher.update(&right);
                let mut output = vec![0u8; 32];
                hasher.finalize(&mut output);
                next.push(output.try_into().expect("Hash output length must be 32 bytes"));
            }
            layers.push(next);
        }
        Self {
            layers,
        }
    }
    pub fn root(&self) -> [u8; 32]{
        self.layers.last().unwrap()[0]
    }

    pub fn proof(&self, idx: usize) -> web3::Result<Vec<([u8; 32], bool)>> {
        let mut proof = vec![];
        let mut index = idx;
        for layer in &self.layers[..self.layers.len()-1] {
            let sibling = if index % 2 == 0 {
                layer.get(index+1).cloned()
            } else {
                Some(layer[index-1])
            };
            if let Some(sib) = sibling {
                proof.push((sib, index % 2 == 0))
            }
            index /= 2;
        }
        Ok(proof)
    }
    
    pub fn verify_proof(root: [u8; 32], leaf: [u8; 32], proof: Vec<([u8; 32], bool)>) -> bool {
        let mut hash = leaf;
        for (sibling, is_left) in proof {
            let mut hasher = Keccak::v256();
            if is_left {
                hasher.update(&hash);
                hasher.update(&sibling);
            } else {
                hasher.update(&sibling);
                hasher.update(&hash);
            }
            hasher.finalize(&mut hash);
        }
        hash == root
    }
}