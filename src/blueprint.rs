pub(crate) mod count_entities;
use std::io::Read;

use base64::{Engine, prelude::BASE64_STANDARD};

pub fn blueprint_to_json(blueprint_str: &str) -> String {
    let (header, b64_body) = blueprint_str
        .trim()
        .split_at_checked(1)
        .unwrap_or_else(|| panic!("blueprint should start with '0'"));
    assert_eq!(header, "0", "blueprint should start with '0'");
    let compressed = BASE64_STANDARD
        .decode(b64_body)
        .unwrap_or_else(|e| panic!("bad base64: {e}"));
    let mut json: String;
    {
        let mut cursor = &compressed[..];
        let mut decode = flate2::read::ZlibDecoder::new(&mut cursor);
        json = String::new();
        decode.read_to_string(&mut json).expect("valid zlib data");
    }
    json
}
