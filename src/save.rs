use std::{
    fs::File,
    io::{BufWriter, Write},
    mem,
    path::{Path, PathBuf},
};

fn format_tag(
    typ: &str,
    name: Option<&serde_json::Value>,
    quality: Option<&serde_json::Value>,
) -> Option<String> {
    let name = name?.as_str()?;
    let quality_suffix = if let Some(quality) = quality
        && let Some(quality) = quality.as_str()
        && quality != "normal"
    {
        format!(",quality={quality}")
    } else {
        String::new()
    };
    Some(format!("[{typ}={name}{quality_suffix}]"))
}

fn format_entity_tag(entity: &serde_json::Value) -> Option<String> {
    format_tag("entity", entity.get("name"), entity.get("quality"))
}

fn format_icon_tag(icon: &serde_json::Value) -> Option<String> {
    format_tag("icon", icon.get("name"), icon.get("quality"))
}

fn format_tile_tag(tile: &serde_json::Value) -> Option<String> {
    format_tag("tile", tile.get("name"), tile.get("quality"))
}

fn unique_strings(iter: impl Iterator<Item = String>) -> impl Iterator<Item = String> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    iter.flat_map(move |s| {
        if seen.contains(&s) {
            None
        } else {
            seen.insert(s.clone());
            Some(s)
        }
    })
}

fn compute_name(json: &serde_json::Value) -> String {
    let mut name = String::new();
    if let Some(thing) = json
        .get("blueprint")
        .or(json.get("blueprint_book"))
        .or(json.get("upgrade_planner"))
        .or(json.get("deconstruction_planner"))
    {
        if let Some(label) = thing.get("label")
            && let Some(label) = label.as_str()
        {
            name = label.to_owned();
        } else if let Some(icons) = thing.get("icons").or(thing
            .get("settings")
            .and_then(|settings| settings.get("icons")))
            && let Some(icons) = icons.as_array()
        {
            for icon in icons {
                if let Some(signal) = icon.get("signal")
                    && let Some(signal_name) = format_icon_tag(signal)
                {
                    if !name.is_empty() {
                        name.push(' ');
                    }
                    name.push_str(&signal_name);
                }
            }
        } else if let Some(blueprint) = json.get("deconstruction_planner")
            && let Some(settings) = blueprint.get("settings")
        {
            let entities = settings.get("entity_filters");
            let entities = entities
                .iter()
                .flat_map(|e| e.as_array())
                .flatten()
                .flat_map(format_entity_tag);

            let tiles = settings.get("tile_filters");
            let tiles = tiles
                .iter()
                .flat_map(|e| e.as_array())
                .flatten()
                .flat_map(format_tile_tag);

            let mut iter = entities.chain(tiles).fuse();

            for name_part in (&mut iter).take(4) {
                if !name.is_empty() {
                    name.push(' ');
                }
                name.push_str(&name_part);
            }
            if iter.next().is_some() {
                name.push_str("…");
            }
        } else if let Some(blueprint) = json.get("upgrade_planner")
            && let Some(settings) = blueprint.get("settings")
            && let Some(mappers) = settings.get("mappers")
            && let Some(mappers) = mappers.as_array()
        {
            let mut iter = unique_strings(
                mappers
                    .iter()
                    .flat_map(|mapper| mapper.get("to"))
                    .flat_map(|to| {
                        let typ = to.get("type");
                        if let Some(typ) = typ
                            && let Some(typ) = typ.as_str()
                            && let Some(name_part) =
                                format_tag(typ, to.get("name"), to.get("quality"))
                        {
                            Some(name_part)
                        } else if let Some(name_part) = to.get("name")
                            && let Some(name_part) = name_part.as_str()
                        {
                            Some(name_part.to_owned())
                        } else {
                            None
                        }
                    }),
            )
            .fuse();
            for name_part in (&mut iter).take(4) {
                if !name.is_empty() {
                    name.push(' ');
                }
                name.push_str(&name_part);
            }
            if iter.next().is_some() {
                name.push_str("…");
            }
            if !name.is_empty() {
                name = format!("Upgrade {name}");
            }
        }
    }

    if name.is_empty() {
        name = "Untitled".to_owned()
    };
    name
}

pub fn save(mut json: serde_json::Value, dir: Option<&Path>) {
    let mut name = compute_name(&json);

    if let Some(index) = json.get_mut("index")
        && let Some(index) = index.as_number()
    {
        name = format!("{index} {name}");
    }

    // replace problematic characters like "/"
    name = sanitize_filename::sanitize_with_options(
        name,
        sanitize_filename::Options {
            windows: true,
            truncate: true,
            replacement: "_",
        },
    );

    let file_path: PathBuf = if let Some(blueprint_book) = json.get_mut("blueprint_book")
        && let Some(blueprints) = blueprint_book.get_mut("blueprints")
        && let Some(blueprints) = blueprints.as_array_mut()
    {
        let blueprints = mem::replace(blueprints, vec![]);
        let path = if let Some(dir) = dir {
            dir.join(&name)
        } else {
            name.clone().into()
        };
        std::fs::create_dir(&path)
            .unwrap_or_else(|e| panic!("error creating book directory {path:?}: {e}"));
        for blueprint in blueprints {
            save(blueprint, Some(&path));
        }
        path.join("book.json")
    } else {
        let file = format!("{name}.json");
        if let Some(dir) = dir {
            dir.join(&file)
        } else {
            file.clone().into()
        }
    };

    {
        let out_file = File::create_new(&file_path)
            .unwrap_or_else(|e| panic!("error creating file {file_path:?}: {e:?}"));
        let mut writer = BufWriter::new(out_file);
        serde_json::to_writer_pretty(&mut writer, &json).expect("error writing {file_path}");
        writer.flush().expect("error writing {file_path}");
    }
    println!("{file_path:?} saved.");
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, fs::read_dir, str::FromStr as _};

    #[track_caller]
    fn read_dir_unwrap(path: &Path) -> Vec<OsString> {
        let mut result = read_dir(path)
            .unwrap_or_else(|e| panic!("error reading directory {path:?}: {e}"))
            .map(|f| {
                f.unwrap_or_else(|e| panic!("error while reading directory {path:?}: {e}"))
                    .file_name()
            })
            .collect::<Vec<_>>();
        result.sort();
        result
    }

    use super::*;
    #[test]
    fn test_save_blueprint() {
        let bp = "0eNqtlN1ugzAMhd/F12Eaf+uK1CeZJhSCu0WChDmhWlXl3Wd+1G6l6wXbFcE6Pv4OSThB1fTYkTYeihNoZY2D4uUETr8Z2Qw1I1uEAhw2qLylSNm20kbyEoIAbWr8hCIOrwLQeO01Tgbjy7E0fVshsUDcMxLQWce91gwT2S+NHwUcoYiS+Pkh5zm1Ju4aBZkAxvRkm7LCd3nQbMBdtkOSk2KewbbTomwle3rqcQYuF/E0WRNZQu756GXD6FzETisIIYhFnGR1nM2KOMr2ZkgzPpfsynYsXtCT5MJN+nQ1fbKCfiaKPEnj9kgXyNLZnhSWzrNWfUv0O3q2Gj39A/peN/4H+Fxg+dX3HjzbTtKIU8Dudop8dYr4fzZgvhfnOGTb87ma7sn1Ft05dOGirtH5IQUPXrbI+iCNwjpSmlSv/c27xv8R7bHlyuXPJOCA5MYw+VOyzbbbfJPmabZJQvgCXQumXg==";
        let dir = tempfile::tempdir().unwrap();
        let json = crate::blueprint::blueprint_to_json(bp);
        let json = serde_json::Value::from_str(&json).expect("should contain valid json");
        save(json.clone(), Some(dir.path()));
        let files = read_dir_unwrap(dir.path());
        assert_eq!(files, &["[icon=selector-combinator].json"]);
        let written_json =
            std::fs::read_to_string(dir.path().join("[icon=selector-combinator].json")).unwrap();
        let written_json = serde_json::Value::from_str(&written_json).unwrap();
        assert_eq!(written_json, json);
    }

    #[test]
    fn test_save_book() {
        let bp = "0eNrlUdtqg0AQ/ZUwz2uIJkYU8pIPCH0vRVadliV7sXsJDeK/d1SiLYS00Mc+ztk5lz3TQSUDtlZoX1bGnKHoFsRB8fxlHN5EbfQEO/GmuRwwzRVCAVWQ50hoh9ajhZ6B0A1+QBH3LwxQe+EFTtRxuJY6qIo2i5jdl2DQGkcsowcXUkrybJ0yuBIly9cpWTTCYj1txAmDVyGJOJnc3Gfx2rQt2shYJOX3wCVlIDjo2ihFCow2VMst94ZCwQGG3MFhOat6G3AAhUc1pJ2LYSB5hVQGHJ9WJ7JbxYRdiDRGS/dJvsvzNNum212WLNVsetb9+wNYPgJ/KP/novf3mNHY93K7EzqPzeo4oZy+dcHyJvAbl/iRy3e9zQO9/hMTWy6D";
        let dir = tempfile::tempdir().unwrap();
        let json = crate::blueprint::blueprint_to_json(bp);
        let json = serde_json::Value::from_str(&json).expect("should contain valid json");
        save(json.clone(), Some(dir.path()));
        let files = read_dir_unwrap(dir.path());
        assert_eq!(files, &["Untitled"]);
        let subfiles = read_dir_unwrap(&dir.path().join("Untitled"));
        assert_eq!(subfiles, ["0 BP Name 1.json", "1 Nested Book", "book.json"]);
        let subsubfiles = read_dir_unwrap(&dir.path().join("Untitled/1 Nested Book"));
        assert_eq!(subsubfiles, ["6 [icon=bulk-inserter].json", "book.json"]);

        // TODO: when load is implemented, test that it can load everything back properly.
    }

    #[test]
    fn test_save_deconstruction_planner() {
        let bp = "0eNptj90KwjAMhd8l1xvofhwr+CQio65Rim0620yU0Xc3ol4MvEu+E87JWcDgGChxnEe2gYbJaSKMoBZIyGzpkt4zElt+DmfrGKOQwwKkPYKC0+yupaWEURQo4DZrJ6eiUIheO0Fj8JOOmoPYwl6AJYMPUJt8LICtwyGhw0++D0Zct1/+J0/+M4LdyibLwuhFXtcpf3UKuIuPEFDtruqbvm+7uq2brsr5Baj6WmY=";
        let dir = tempfile::tempdir().unwrap();
        let json = crate::blueprint::blueprint_to_json(bp);
        let json = serde_json::Value::from_str(&json).expect("should contain valid json");
        save(json.clone(), Some(dir.path()));
        let files = read_dir_unwrap(dir.path());
        assert_eq!(files, &["[entity=bulk-inserter] [tile=landfill].json"]);
        let written_json = std::fs::read_to_string(
            dir.path()
                .join("[entity=bulk-inserter] [tile=landfill].json"),
        )
        .unwrap();
        let written_json = serde_json::Value::from_str(&written_json).unwrap();
        assert_eq!(written_json, json);
    }

    #[test]
    fn test_save_upgrade_planner() {
        let bp = "0eNqtkdFqwzAMRf9FzwmUNmkbQ7+klKGuagjEsisroyX43ycz1sIGWx/6ZOkiXx10Z5hiL3iitzgiMwm4GRKpDtynUnuMkcTK/QxnCb5oeosEDoh10BtUwOhLr4KcYhCtjzSq6ZcJxzLhgIN4HE16Dz6ioAZbBDvIFWj4w/KMSesfvvZn4BNdwS1y9S/UZKPSS7D3xVi/nB9gzRNgKRqE2sFfBnR3fIBs86E0Sr6c4ivp+jvpCj4s2SEwuHa97JquazerdtVsljl/Ag1ntSg=";
        let dir = tempfile::tempdir().unwrap();
        let json = crate::blueprint::blueprint_to_json(bp);
        let json = serde_json::Value::from_str(&json).expect("should contain valid json");
        save(json.clone(), Some(dir.path()));
        let files = read_dir_unwrap(dir.path());
        const EXPECTED_NAME: &str = "Upgrade [entity=fast-transport-belt] [entity=fast-underground-belt] [entity=fast-splitter].json";
        assert_eq!(files, [EXPECTED_NAME]);
        let written_json = std::fs::read_to_string(dir.path().join(EXPECTED_NAME)).unwrap();
        let written_json = serde_json::Value::from_str(&written_json).unwrap();
        assert_eq!(written_json, json);
    }

    #[test]
    fn test_dedupe_names_upgrade_planner() {
        let bp = "0eNq1UMsKwjAQ/Jc9V5A+LA34JSIS2rUGmt2YbNVS8u8mYK968jaPZXaYFWY3ej3gxU2aCD2oFQKKGBpDxlY7hz7B0wpXzzZrsjgEBUhiZIECSNvMn8wD0q6/YZCk3mc9ZV8Bsbd6SlLP1mmvhdMbOEIsQPhLYBDE6ZOXbg0N+AK1j8XPKsbz/4vU8ZyJYGqyzbjbZizgkWYzTKCaQ9nVXde0VVPVbRnjGw8nfKM=";
        let json = crate::blueprint::blueprint_to_json(bp);
        let json = serde_json::Value::from_str(&json).expect("should contain valid json");
        assert_eq!(compute_name(&json), "Upgrade [entity=steel-chest]")
    }

    #[test]
    fn test_upgrade_remove_module() {
        let bp = "0eNp1js0KwjAQhN9lzy1IfywN+CQiEsxaAtlkTbZiKXl3E7BHbzPfDrOzw8pL1Abv7LT3GEHtkFDE+iVVTZoZY5HXHZ4xUGWyMYICK0jQgNdUXWJE01Iwq8NCX6t2VrZy8CGSdgU9ArGOWkJ5AhfIDUj4W4fEsv3q2uSC1Lz1Bj+gTvlWTc2rY3977G/gXfba4EGN524e5nmc+rEfpi7nL1TiT8I=";
        let json = crate::blueprint::blueprint_to_json(bp);
        let json = serde_json::Value::from_str(&json).expect("should contain valid json");
        assert_eq!(compute_name(&json), "Upgrade [item=empty-module-slot]")
    }

    #[test]
    fn test_upgrade_quality() {
        let bp = "0eNqVT1sKwjAQvMt+V5A+LA14EhHZtmsNdDcx3Yql5O4miAfwbx7M7M4Oq58CjnTzM4pQALPDQqpWpiVjRu8pJHjZ4R4cZ003T2CARK1uUIAgZ95bNzyQ+1RSwHPFObsGxAXGOUmDY48B1aUjcIZYgLr/61ZJPewk562M9AZzjNdMlDj73z2H354CXul/mwKmOZVd3XVNWzVV3ZYxfgDMH1WE";
        let json = crate::blueprint::blueprint_to_json(bp);
        let json = serde_json::Value::from_str(&json).expect("should contain valid json");
        assert_eq!(
            compute_name(&json),
            "Upgrade [entity=biochamber,quality=uncommon]"
        )
    }
}
