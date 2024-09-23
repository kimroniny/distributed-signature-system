use bls_signatures::{aggregate, verify_messages, PrivateKey, PublicKey, Serialize, Signature};

// Function to create a BLS signature
fn create_signature(message: &[u8]) -> (Signature, PublicKey) {
    // Generate a new key pair
    let keypair = PrivateKey::generate(&mut rand_core::OsRng);
    let public_key = keypair.public_key(); // Get the public key from the keypair
    let message_with_pk = format!("{}:{}", hex::encode(message), hex::encode(public_key.as_bytes()));
    // Sign the message
    let signature = keypair.sign(message_with_pk.as_bytes());
    
    (signature, public_key) // Return both the signature and the public key
}

// Example usage
fn example_usage(num_nodes: usize) {
    let message = b"Hello, BLS signatures!";
    
    // Create a signature
    let (signatures, public_keys): (Vec<Signature>, Vec<PublicKey>) = (0..num_nodes).map(|_| create_signature(message)).collect::<Vec<(Signature, PublicKey)>>().into_iter().unzip();

    // Aggregate the signatures
    let aggregated_signature = aggregate(&signatures).unwrap();

    let messages_with_pks: Vec<String> = (0..num_nodes).map(|i| format!("{}:{}", hex::encode(message), hex::encode(public_keys[i].as_bytes()))).collect();

    let is_valid = verify_messages(
        &aggregated_signature, 
        &messages_with_pks.iter().map(|s| s.as_bytes()).collect::<Vec<&[u8]>>(), 
        &public_keys
    );
    
    println!("Signature valid: {}", is_valid);
}

// Call the example usage function
fn main() {
    example_usage(10);
}
