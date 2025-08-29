use std::{fs, path::Path};

use itertools::Itertools;

fn load_file(path: &Path) -> serde_json::Value {
    serde_json::from_str(
        &fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("error reading {path:?}: {e}", path = &path)),
    )
    .unwrap_or_else(|e| panic!("invalid json in {path:?}: {e}"))
}
pub fn load(path: &Path) -> serde_json::Value {
    let files = match path
        .read_dir()
        .and_then(|files| files.collect::<Result<Vec<_>, _>>())
    {
        Ok(files) => files,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotADirectory => return load_file(path),
            _ => {
                panic!("error loading {path:?}: {e}");
            }
        },
    };
    let mut book_json: Option<serde_json::Value> = None;
    let mut entries = vec![];
    for file in files {
        let path = file.path();
        let json = load(&path);
        if file.file_name() == "book.json" {
            book_json = Some(json)
        } else {
            let json_index: Option<u64> = json.get("index").map(|val| {
                val.as_number()
                    .and_then(|num| num.as_u64())
                    .unwrap_or_else(|| panic!("invalid index {val:?} in {path:?}"))
            });
            let index = if let Some(filename) = file.file_name().to_str()
                && let Some((first_word, _)) = filename.split_once(' ')
                && let Ok(filename_index) = first_word.parse::<u64>()
            {
                if let Some(json_index) = json_index {
                    if filename_index != json_index {
                        eprintln!(
                            "WARN: {path:?} index mismatch; file contains index {json_index}, using index {filename_index} from filename"
                        )
                    }
                } else {
                    eprintln!(
                        "WARN: {path:?} missing index, assuming index {filename_index} from filename"
                    )
                }
                filename_index
            } else {
                json_index.unwrap_or_else(|| {
                    panic!("error: {path:?} missing index, can't load into book")
                })
            };

            entries.push((path, index, json));
        }
    }
    let book_json_path = path.join("book.json");
    let Some(mut book_json) = book_json else {
        panic!("{book_json_path:?} not found");
    };
    entries.sort_by_key(|(_path, index, _json)| *index);
    for (l, r) in entries.iter().tuple_windows() {
        if l.1 == r.1 {
            panic!(
                "error: {l:?} and {r:?} have same index {index}",
                l = l.0,
                r = r.0,
                index = l.1
            );
        }
    }
    let blueprints = book_json
        .get_mut("blueprint_book")
        .unwrap_or_else(|| panic!("error: {book_json_path:?} not a blueprint book??"))
        .get_mut("blueprints")
        .unwrap_or_else(|| panic!("error: {book_json_path:?} missing empty blueprints array"))
        .as_array_mut()
        .unwrap_or_else(|| {
            panic!("error: {book_json_path:?} blueprints wrong type (expected array)")
        });
    if !blueprints.is_empty() {
        panic!("error: {book_json_path:?} blueprints array not empty")
    }
    *blueprints = entries
        .into_iter()
        .map(|(_path, _index, json)| json)
        .collect();
    book_json
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use crate::test_util::read_dir_unwrap;

    use super::*;

    // See also save tests which check round-tripping.

    #[test]
    #[should_panic = "have same index 1"]
    fn test_load_duplicate() {
        let bp = "0eNrlUdtqg0AQ/ZUwz2uIJkYU8pIPCH0vRVadliV7sXsJDeK/d1SiLYS00Mc+ztk5lz3TQSUDtlZoX1bGnKHoFsRB8fxlHN5EbfQEO/GmuRwwzRVCAVWQ50hoh9ajhZ6B0A1+QBH3LwxQe+EFTtRxuJY6qIo2i5jdl2DQGkcsowcXUkrybJ0yuBIly9cpWTTCYj1txAmDVyGJOJnc3Gfx2rQt2shYJOX3wCVlIDjo2ihFCow2VMst94ZCwQGG3MFhOat6G3AAhUc1pJ2LYSB5hVQGHJ9WJ7JbxYRdiDRGS/dJvsvzNNum212WLNVsetb9+wNYPgJ/KP/novf3mNHY93K7EzqPzeo4oZy+dcHyJvAbl/iRy3e9zQO9/hMTWy6D";
        let dir = tempfile::tempdir().unwrap();
        let json = crate::blueprint::blueprint_to_json(bp);
        let json = serde_json::Value::from_str(&json).expect("should contain valid json");
        crate::save::save(json.clone(), Some(dir.path()));
        let files = read_dir_unwrap(dir.path());
        assert_eq!(files, &["Untitled"]);
        let subfiles = read_dir_unwrap(&dir.path().join("Untitled"));
        assert_eq!(subfiles, ["0 BP Name 1.json", "1 Nested Book", "book.json"]);

        std::fs::rename(
            dir.path().join("Untitled").join("0 BP Name 1.json"),
            dir.path().join("Untitled").join("1 BP Name 1.json"),
        )
        .unwrap();

        let loaded_json = load(&dir.path().join("Untitled"));
        assert_eq!(loaded_json, json)
    }
}
