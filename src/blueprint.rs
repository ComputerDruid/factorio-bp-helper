pub(crate) mod count_entities;
pub(crate) mod upgrade_quality;

use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    io::Read,
    mem,
};

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

pub(crate) enum BlueprintType<T: Borrow<serde_json::Value>> {
    Blueprint(T),
    BlueprintBook(T),
    UpgradePlanner(T),
    DeconstructionPlanner(T),
}

impl BlueprintType<&serde_json::Value> {
    #[allow(
        dead_code,
        reason = "used in tests but not logically a test-only function"
    )]
    pub fn new(json: &serde_json::Value) -> BlueprintType<&serde_json::Value> {
        let json = match json {
            serde_json::Value::Object(json) => json,
            _ => panic!("blueprint entry should be an object, got {json:?}"),
        };

        if json.contains_key("blueprint") {
            BlueprintType::Blueprint(&json["blueprint"])
        } else if json.contains_key("blueprint_book") {
            BlueprintType::BlueprintBook(&json["blueprint_book"])
        } else if json.contains_key("upgrade_planner") {
            BlueprintType::UpgradePlanner(&json["upgrade_planner"])
        } else if json.contains_key("deconstruction_planner") {
            BlueprintType::DeconstructionPlanner(&json["deconstruction_planner"])
        } else {
            panic!("blueprint has unknown type: {:?}", json.keys());
        }
    }
}

impl BlueprintType<&mut serde_json::Value> {
    pub fn new(json: &mut serde_json::Value) -> BlueprintType<&mut serde_json::Value> {
        let json = match json {
            serde_json::Value::Object(json) => json,
            _ => panic!("blueprint entry should be an object, got {json:?}"),
        };

        if json.contains_key("blueprint") {
            BlueprintType::Blueprint(&mut json["blueprint"])
        } else if json.contains_key("blueprint_book") {
            BlueprintType::BlueprintBook(&mut json["blueprint_book"])
        } else if json.contains_key("upgrade_planner") {
            BlueprintType::UpgradePlanner(&mut json["upgrade_planner"])
        } else if json.contains_key("deconstruction_planner") {
            BlueprintType::DeconstructionPlanner(&mut json["deconstruction_planner"])
        } else {
            panic!("blueprint has unknown type: {:?}", json.keys());
        }
    }

    #[inline]
    pub(crate) fn as_ref(&self) -> BlueprintType<&serde_json::Value> {
        self.as_ref_inner()
    }

    fn any_mut(&mut self) -> &mut serde_json::Value {
        match self {
            BlueprintType::Blueprint(value) => value,
            BlueprintType::BlueprintBook(value) => value,
            BlueprintType::UpgradePlanner(value) => value,
            BlueprintType::DeconstructionPlanner(value) => value,
        }
    }

    pub(crate) fn set_tag_in_description(&mut self, tag: &str, value: &str) {
        let description = self
            .any_mut()
            .as_object_mut()
            .expect("should be a json object")
            .entry("description")
            .or_insert_with(|| json!(""));
        let serde_json::Value::String(description) = description else {
            panic!("expected description to be a string")
        };
        *description = set_tag_in_string(mem::take(description), tag, value);
    }
}

impl<T: Borrow<serde_json::Value>> BlueprintType<T> {
    pub(crate) fn any(&self) -> &serde_json::Value {
        match self {
            BlueprintType::Blueprint(value) => value,
            BlueprintType::BlueprintBook(value) => value,
            BlueprintType::UpgradePlanner(value) => value,
            BlueprintType::DeconstructionPlanner(value) => value,
        }
        .borrow()
    }

    fn as_ref_inner(&self) -> BlueprintType<&serde_json::Value> {
        match self {
            BlueprintType::Blueprint(value) => BlueprintType::Blueprint(value.borrow()),
            BlueprintType::BlueprintBook(value) => BlueprintType::BlueprintBook(value.borrow()),
            BlueprintType::UpgradePlanner(value) => BlueprintType::UpgradePlanner(value.borrow()),
            BlueprintType::DeconstructionPlanner(value) => {
                BlueprintType::DeconstructionPlanner(value.borrow())
            }
        }
    }

    pub(crate) fn label(&self) -> Option<&str> {
        self.any().get("label")?.as_str()
    }
}

fn set_tag_in_string(description: String, tag: &str, value: &str) -> String {
    let start_offset = if description.starts_with(&format!("{tag}:")) {
        0
    } else if let Some(index) = description.find(&format!("\n{tag}:")) {
        index + "\n".len()
    } else {
        // couldn't find tag, add it to a new line at the end
        let mut new_description = description;
        if !new_description.is_empty() {
            new_description.push('\n');
        }
        new_description.push_str(&format!("{tag}: {value}"));
        return new_description;
    };
    let old_description = description;
    let prefix = &old_description[..start_offset];
    let suffix = if let Some(end_index) = &old_description[start_offset..].find("\n") {
        &old_description[start_offset + end_index..]
    } else {
        ""
    };
    format!("{prefix}{tag}: {value}{suffix}")
}

pub fn make_constant_combinator_json(signals: Vec<((String, Quality), i64)>) -> serde_json::Value {
    const COMBINATOR: &str = "0eNqNkNEOgjAMRf+lz8MIEYH9ijFmw6pNRkdGMRKyf3fDF5+Mj23uPbe3K1g34xiIBfQK1HueQJ9WmOjOxuWdLCOCBhIcQAGbIU9ZJ4al6P1giY34AFEB8RVfoMt4VoAsJIQf3DYsF54HiyEJfoIUjH5KXs85P/GK+ljvagUL6OpY7uqUdKWA/UdyUJkiwbuLxYd5UkIk342cYPirjQTD0+iDFBad5CK9n/NHyu9KMbfajPrrawqeKWU7pGrLQ9NVTVt1zb7tYnwDHB11ag==";
    let json = blueprint_to_json(COMBINATOR);
    let mut value = serde_json::from_str::<serde_json::Value>(&json)
        .expect("constant combinator should be a valid blueprint");
    let new_filters = signals
        .into_iter()
        .enumerate()
        .map(|(idx, ((item_name, quality), count))| {
            let mut signal = json!({
                "type": "item",
                "name": item_name,
            });
            if let Some(quality) = quality.0 {
                signal
                    .as_object_mut()
                    .unwrap()
                    .insert("quality".to_owned(), json!(quality));
            }

            let filter = json!({
                "index": idx + 1,
                "count": count,
                "signal": signal,
            });
            filter
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quality(pub Option<String>);

impl Debug for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Quality {
    pub fn fmt_suffix(&self) -> impl Display {
        struct FromFn<T>(T);
        impl<T: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result> Display for FromFn<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                (self.0)(f)
            }
        }
        // someday, we can replace this with `std::fmt::from_fn`
        FromFn(|f: &mut std::fmt::Formatter<'_>| {
            if let Some(quality) = &self.0 {
                write!(f, " ({quality})")
            } else {
                Ok(())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::blueprint::set_tag_in_string;

    #[test]
    fn test_set_tag() {
        assert_eq!(
            set_tag_in_string("foo bar".into(), "t", "val"),
            "foo bar\nt: val"
        );

        // Empty string considered to be empty
        assert_eq!(set_tag_in_string("".into(), "t", "val"), "t: val");

        // Existing newlines are considered intentional
        assert_eq!(set_tag_in_string("\n".into(), "t", "val"), "\n\nt: val");
    }
}
