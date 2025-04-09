use hmac::{Hmac, Mac};
use rand::{Rng, distr::Alphanumeric, rng};
use sha2::Sha256;

pub fn generate_random_token() -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(32) // 32-character token
        .map(char::from)
        .collect()
}

pub fn capitalize_first(s: String) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

pub fn generate_hash(str: &str) -> String {
    let key = env!("HASH_KEY");
    let mut mac =
        Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("HMAC can take a key of any size");
    mac.update(str.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    hex::encode(code_bytes) // Convert to hex string
}

pub fn check_hash(str: &str, provided_hash: &str) -> bool {
    let expected_hash = generate_hash(str);
    expected_hash == provided_hash
}
