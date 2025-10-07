use super::*;
use libp2p::{Multiaddr, PeerId};

// ============================================================================
// validate_address()
// ============================================================================

#[test]
fn test_validate_address_valid_ipv4() {
    let validator = Libp2pNetworkEntityValidator;
    let valid_address =
        "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp";

    let result = validator.validate_address(valid_address.to_string());
    assert!(result.is_ok());

    let address = result.unwrap();

    assert_eq!(address.get_address_str(), valid_address);

    let multiaddr = address.get_address_str().parse::<Multiaddr>();
    assert!(multiaddr.is_ok(), "Address should parse as valid Multiaddr");

    let peer_id = address.get_peer_id();
    let libp2p_peer_id = peer_id.as_str().parse::<PeerId>();
    assert!(
        libp2p_peer_id.is_ok(),
        "Peer ID should parse as valid PeerId"
    );
    assert_eq!(
        libp2p_peer_id.unwrap().to_string(),
        "12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp"
    );
}

#[test]
fn test_validate_address_valid_ipv6() {
    let validator = Libp2pNetworkEntityValidator;
    let valid_address =
        "/ip6/::1/tcp/4001/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp";

    let result = validator.validate_address(valid_address.to_string());
    assert!(result.is_ok());

    let address = result.unwrap();

    let multiaddr = address.get_address_str().parse::<Multiaddr>();
    assert!(
        multiaddr.is_ok(),
        "IPv6 address should parse as valid Multiaddr"
    );

    let peer_id = address.get_peer_id();
    let libp2p_peer_id = peer_id.as_str().parse::<PeerId>();
    assert!(
        libp2p_peer_id.is_ok(),
        "Peer ID should parse as valid PeerId"
    );
    assert_eq!(
        libp2p_peer_id.unwrap().to_string(),
        "12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp"
    );
}

#[test]
fn test_validate_address_valid_dns() {
    let validator = Libp2pNetworkEntityValidator;
    let valid_dns_address =
        "/dns4/example.com/tcp/4001/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp";

    let result = validator.validate_address(valid_dns_address.to_string());
    assert!(result.is_ok());

    let address = result.unwrap();

    let multiaddr = address.get_address_str().parse::<Multiaddr>();
    assert!(
        multiaddr.is_ok(),
        "DNS address should parse as valid Multiaddr"
    );
}

#[test]
fn test_validate_address_missing_peer_id() {
    let validator = Libp2pNetworkEntityValidator;
    let address_without_peer = "/ip4/127.0.0.1/tcp/4001";

    let result = validator.validate_address(address_without_peer.to_string());
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("doesn't contain a valid P2P peer ID")
    );
}

#[test]
fn test_validate_address_invalid_format() {
    let validator = Libp2pNetworkEntityValidator;
    let invalid_address = "not-a-valid-multiaddr";

    let result = validator.validate_address(invalid_address.to_string());
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid multiaddr"));
}

#[test]
fn test_validate_address_empty_string() {
    let validator = Libp2pNetworkEntityValidator;

    let result = validator.validate_address(String::new());
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn test_validate_address_with_whitespace() {
    let validator = Libp2pNetworkEntityValidator;
    let address_with_whitespace =
        " /ip4/127.0.0.1/tcp/4001/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp ";

    let result = validator.validate_address(address_with_whitespace.to_string());
    assert!(result.is_err());
}

#[test]
fn test_validate_address_matches_constructed_multiaddr() {
    let validator = Libp2pNetworkEntityValidator;

    let peer_id = PeerId::random();
    let multiaddr: Multiaddr = format!("/ip4/192.168.1.100/tcp/9000/p2p/{}", peer_id)
        .parse()
        .unwrap();

    let result = validator.validate_address(multiaddr.to_string());
    assert!(result.is_ok());

    let network_address = result.unwrap();

    let parsed_multiaddr: Multiaddr = network_address.get_address_str().parse().unwrap();
    assert_eq!(
        parsed_multiaddr, multiaddr,
        "Parsed multiaddr should match original"
    );

    let network_peer_id = network_address.get_peer_id();
    let parsed_peer_id: PeerId = network_peer_id.as_str().parse().unwrap();
    assert_eq!(
        parsed_peer_id, peer_id,
        "Parsed peer ID should match original"
    );
}

// ============================================================================
// validate_peer_id()
// ============================================================================

#[test]
fn test_validate_peer_id_valid() {
    let validator = Libp2pNetworkEntityValidator;
    let valid_peer_id = "12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp";

    let result = validator.validate_peer_id(valid_peer_id.to_string());
    assert!(result.is_ok());

    let peer_id = result.unwrap();

    assert_eq!(peer_id.as_str(), valid_peer_id);

    let libp2p_peer_id = peer_id.as_str().parse::<PeerId>();
    assert!(
        libp2p_peer_id.is_ok(),
        "Peer ID should parse as valid PeerId"
    );

    assert_eq!(libp2p_peer_id.unwrap().to_string(), valid_peer_id);
}

#[test]
fn test_validate_peer_id_invalid() {
    let validator = Libp2pNetworkEntityValidator;
    let invalid_peer_id = "invalid-peer-id";

    let result = validator.validate_peer_id(invalid_peer_id.to_string());
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid peer_id"));
}

#[test]
fn test_validate_peer_id_empty_string() {
    let validator = Libp2pNetworkEntityValidator;

    let result = validator.validate_peer_id(String::new());
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid peer_id"));
}

#[test]
fn test_validate_peer_id_with_whitespace() {
    let validator = Libp2pNetworkEntityValidator;
    let peer_id_with_whitespace = " 12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp ";

    let result = validator.validate_peer_id(peer_id_with_whitespace.to_string());
    assert!(result.is_err());
}

#[test]
fn test_validate_peer_id_matches_constructed_peer_id() {
    let validator = Libp2pNetworkEntityValidator;

    let original_peer_id = PeerId::random();

    let result = validator.validate_peer_id(original_peer_id.to_string());
    assert!(result.is_ok());

    let network_peer_id = result.unwrap();

    let parsed_peer_id: PeerId = network_peer_id.as_str().parse().unwrap();
    assert_eq!(
        parsed_peer_id, original_peer_id,
        "Parsed peer ID should match original"
    );

    assert_eq!(
        network_peer_id.as_bytes(),
        original_peer_id.to_bytes().as_slice(),
        "Peer ID bytes should match"
    );
}
