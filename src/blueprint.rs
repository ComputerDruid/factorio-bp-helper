pub(crate) mod count_entities;
use std::io::Read;

use base64::{Engine, prelude::BASE64_STANDARD};
use flate2::Compression;
use serde_json::json;

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

pub fn json_to_blueprint(value: serde_json::Value) -> String {
    let mut result = "0".to_owned();
    let json = value.to_string();
    let compressed: Vec<u8> = {
        let mut buf = Vec::new();
        flate2::read::ZlibEncoder::new(json.as_bytes(), Compression::fast())
            .read_to_end(&mut buf)
            .unwrap();
        buf
    };
    BASE64_STANDARD.encode_string(&compressed, &mut result);
    result
}

pub fn make_constant_combinator_json(signals: Vec<(String, i64)>) -> serde_json::Value {
    const COMBINATOR: &str = "0eNqNkNEOgjAMRf+lz8MIEYH9ijFmw6pNRkdGMRKyf3fDF5+Mj23uPbe3K1g34xiIBfQK1HueQJ9WmOjOxuWdLCOCBhIcQAGbIU9ZJ4al6P1giY34AFEB8RVfoMt4VoAsJIQf3DYsF54HiyEJfoIUjH5KXs85P/GK+ljvagUL6OpY7uqUdKWA/UdyUJkiwbuLxYd5UkIk342cYPirjQTD0+iDFBad5CK9n/NHyu9KMbfajPrrawqeKWU7pGrLQ9NVTVt1zb7tYnwDHB11ag==";
    let json = blueprint_to_json(COMBINATOR);
    let mut value = serde_json::from_str::<serde_json::Value>(&json)
        .expect("constant combinator should be a valid blueprint");
    let new_filters = signals
        .iter()
        .enumerate()
        .map(|(idx, (item_name, count))| {
            json!({
                "index": idx + 1,
                "count": count,
                "signal": {
                    "type": "item",
                    "name": item_name,
                }
            })
        })
        .collect::<Vec<_>>();
    let entities = value
        .get_mut("blueprint")
        .unwrap()
        .get_mut("entities")
        .unwrap()
        .as_array_mut()
        .unwrap();
    let &mut [ref mut const_combinator] = &mut entities[..] else {
        panic!("should only be one entity in constant combinator blueprint");
    };
    let filters = const_combinator
        .get_mut("control_behavior")
        .unwrap()
        .get_mut("filters")
        .unwrap()
        .as_array_mut()
        .unwrap();
    *filters = new_filters;
    value
}
