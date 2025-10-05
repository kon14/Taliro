use super::*;

#[test]
fn test_hash_creation_and_display() {
    let bytes = [1u8; 32];
    let hash = Hash::new(bytes);

    assert_eq!(hash.as_bytes(), &bytes);
    assert_eq!(hash.as_ref(), &bytes);

    // Test display formatting
    let display_str = format!("{}", hash);
    assert_eq!(display_str.len(), 64); // 32 bytes * 2 hex chars
    assert!(display_str.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_hash_from_hex_string() {
    let hex_str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let hash = Hash::try_from(hex_str).expect("Valid hex string should parse");

    let expected_bytes = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd,
        0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
        0xcd, 0xef,
    ];

    assert_eq!(hash.as_bytes(), &expected_bytes);

    // Test that display matches original string
    assert_eq!(format!("{}", hash), hex_str);
}

#[test]
fn test_hash_from_invalid_hex_string() {
    // Too short
    let result = Hash::try_from("abc123");
    assert!(result.is_err());

    // Too long
    let result =
        Hash::try_from("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef00");
    assert!(result.is_err());

    // Invalid characters
    let result = Hash::try_from("xyz3456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    assert!(result.is_err());
}

#[test]
fn test_hash_equality_and_hashing() {
    let hash1 = Hash::new([1u8; 32]);
    let hash2 = Hash::new([1u8; 32]);
    let hash3 = Hash::new([2u8; 32]);

    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);

    // Test that equal hashes have equal std::hash values
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash as StdHash, Hasher};

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    hash1.hash(&mut hasher1);
    hash2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_hash_debug_format() {
    let hash = Hash::new([0x42u8; 32]);
    let debug_str = format!("{:?}", hash);

    assert!(debug_str.starts_with("Hash("));
    assert!(debug_str.ends_with(")"));
    assert!(debug_str.contains("4242424242424242")); // Part of the hex representation
}
