use std::io::{self, Error, ErrorKind};

use ethereum_types::H256;
use ring::digest::{digest, SHA256};
use ripemd::{Digest, Ripemd160};
use sha3::Keccak256;

/// Converts public key bytes to the short address bytes (20-byte).
/// "hashing.PubkeyBytesToAddress" and "ids.ToShortID"
/// ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/hashing#PubkeyBytesToAddress
pub fn hash_sha256_ripemd160<S>(pub_key_bytes: S) -> io::Result<Vec<u8>>
where
    S: AsRef<[u8]>,
{
    let digest_sha256 = digest(&SHA256, pub_key_bytes.as_ref());

    // "hashing.PubkeyBytesToAddress"
    // acquire hash digest in the form of GenericArray,
    // which in this case is equivalent to [u8; 20]
    // already in "type ShortID [20]byte" format
    let sha256_ripemd160 = Ripemd160::digest(&digest_sha256);

    // "ids.ToShortID" merely enforces "ripemd160" size!
    // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/ids#ToShortID
    if sha256_ripemd160.len() != 20 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "ripemd160 of sha256 must be 20-byte, got {}",
                sha256_ripemd160.len()
            ),
        ));
    }

    Ok(sha256_ripemd160.to_vec())
}

pub fn keccak256(data: impl AsRef<[u8]>) -> H256 {
    H256::from_slice(&Keccak256::digest(data.as_ref()))
}

/// ref. https://github.com/Ethereum/EIPs/blob/master/EIPS/eip-55.md
pub fn eth_checksum(addr: &str) -> String {
    let addr_lower_case = addr
        .trim_start_matches(super::private_key::HEX_ENCODE_PREFIX)
        .to_lowercase();
    let digest_h256 = keccak256(&addr_lower_case.as_bytes());

    // this also works...
    //
    // addr_lower_case
    //     .chars()
    //     .enumerate()
    //     .map(|(i, c)| {
    //         if matches!(c, 'a' | 'b' | 'c' | 'd' | 'e' | 'f')
    //             && (digest_h256[i >> 1] & if i % 2 == 0 { 128 } else { 8 } != 0)
    //         {
    //             c.to_ascii_uppercase()
    //         } else {
    //             c
    //         }
    //     })
    //     .collect::<String>()

    checksum_eip55(&addr_lower_case, &hex::encode(digest_h256))
}

/// ref. https://github.com/Ethereum/EIPs/blob/master/EIPS/eip-55.md
fn checksum_eip55(addr: &str, addr_hash: &str) -> String {
    let mut chksum = String::new();
    for (c, hash_char) in addr.chars().zip(addr_hash.chars()) {
        if hash_char.to_digit(16) >= Some(8) {
            chksum.extend(c.to_uppercase());
        } else {
            chksum.push(c);
        }
    }
    chksum
}

/// Converts "bech32::encode"d AVAX address to the short address bytes (20-byte) and HRP for network name.
pub fn avax_address_to_short_bytes(chain_alias: &str, addr: &str) -> io::Result<(String, Vec<u8>)> {
    let trimmed = if chain_alias.is_empty() {
        addr.trim().to_string()
    } else {
        // e.g., "P-custom12szthht8tnl455u4mz3ns3nvvkel8ezvw2n8cx".trim_start_matches("P-")
        let pfx = if chain_alias.ends_with("-") {
            chain_alias.to_string()
        } else {
            format!("{}-", chain_alias)
        };
        addr.trim_start_matches(&pfx).to_string()
    };

    let (hrp, data, _) = bech32::decode(&trimmed)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed bech32::decode '{}'", e)))?;

    let convert = bech32::convert_bits(&data, 5, 8, false).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed bech32::convert_bits '{}'", e),
        )
    })?;
    Ok((hrp, convert))
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- key::secp256k1::address::test_avax_address_to_short_bytes --exact --show-output
#[test]
fn test_avax_address_to_short_bytes() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let pk = crate::key::secp256k1::private_key::Key::generate().unwrap();
    let pubkey = pk.to_public_key();
    let short_addr = pubkey.to_short_bytes().unwrap();

    let x_avax_addr = pubkey.to_avax_address(1, "X").unwrap();
    let p_avax_addr = pubkey.to_avax_address(1, "P").unwrap();
    let c_avax_addr = pubkey.to_avax_address(1, "C").unwrap();
    log::info!("AVAX X address: {}", x_avax_addr);
    log::info!("AVAX P address: {}", p_avax_addr);
    log::info!("AVAX C address: {}", c_avax_addr);

    let (hrp, parsed_short_addr) = avax_address_to_short_bytes("X", &x_avax_addr).unwrap();
    assert_eq!(hrp, "avax");
    assert_eq!(parsed_short_addr, short_addr);

    let (hrp, parsed_short_addr) = avax_address_to_short_bytes("P", &p_avax_addr).unwrap();
    assert_eq!(hrp, "avax");
    assert_eq!(parsed_short_addr, short_addr);

    let (hrp, parsed_short_addr) = avax_address_to_short_bytes("C", &c_avax_addr).unwrap();
    assert_eq!(hrp, "avax");
    assert_eq!(parsed_short_addr, short_addr);
}