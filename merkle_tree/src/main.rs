use merkle_tree::MerkleTree;

fn main() {
    // Example usage
    let data = vec![
        b"hello".to_vec(),
        b"world".to_vec(),
        b"foo".to_vec(),
        b"bar".to_vec(),
    ];
    
    // Create merkle tree
    let tree = MerkleTree::new(data.clone());

    // Get root
    if let Some(root) = tree.root() {
        println!("Root hash: {}", hex::encode(root));
    }

    // Get proof for first leaf
    if let Some(proof) = tree.get_proof(3) {
        println!("Generated proof for leaf 3");

        // Verify proof
        let is_valid = tree.verify_proof(&proof, &data[3]);
        println!("Proof valid: {}", is_valid);
    }
}
