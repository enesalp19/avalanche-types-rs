use std::io::{Error, ErrorKind, Result};

use crate::ids;
use crate::message::{self, Compressor, Outbound};

/// Message sent from the VM, in response to "app_request".
#[derive(
    std::clone::Clone,
    std::cmp::Eq,
    std::cmp::Ord,
    std::cmp::PartialEq,
    std::cmp::PartialOrd,
    std::fmt::Debug,
    std::hash::Hash,
)]
pub struct Message {
    pub chain_id: ids::Id,
    pub request_id: u32,
    pub app_bytes: Vec<u8>,
}

impl Message {
    pub fn create(
        chain_id: ids::Id,
        request_id: u32,
        app_bytes: Vec<u8>,
    ) -> impl Outbound + Compressor {
        Self {
            chain_id,
            request_id,
            app_bytes,
        }
    }
}

/// ref. https://doc.rust-lang.org/std/string/trait.ToString.html
/// ref. https://doc.rust-lang.org/std/fmt/trait.Display.html
/// Use "Self.to_string()" to directly invoke this
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "msg app_response")
    }
}

impl Outbound for Message {
    fn serialize_with_header(&self) -> Result<bytes::Bytes> {
        let type_id = message::TYPES
            .get("app_response")
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "unknown type name"))?;

        let packer = message::default_packer_with_header();
        packer.pack_byte(*type_id)?;
        packer.pack_bool(false)?; // compressible
        packer.pack_bytes(self.chain_id.as_ref())?;
        packer.pack_u32(self.request_id)?;
        packer.pack_bytes_with_header(self.app_bytes.as_ref())?;

        Ok(packer.take_bytes())
    }
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- message::app_response::test_message --exact --show-output
#[test]
fn test_message() {
    let msg = Message::create(ids::Id::empty(), 7, vec![0x01, 0x02, 0x03, 0x04]);
    let data_with_header = msg.serialize_with_header().unwrap();
    // for c in &data_with_header {
    //     print!("{:#02x},", *c);
    // }

    let expected_data: &[u8] = &[
        0x00, 0x00, 0x00, 0x2e, // message length
        0x15, // type_id
        0x00, // compressible
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, // chain_id
        0x00, 0x00, 0x00, 0x07, // request_id
        0x00, 0x00, 0x00, 0x04, // length of app_bytes
        0x01, 0x02, 0x03, 0x04, // app_bytes
    ];
    assert!(cmp_manager::eq_vectors(&expected_data, &data_with_header));
}