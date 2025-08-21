use std::{
    fs::File,
    io::{BufWriter, Write},
    mem,
    path::{Path, PathBuf},
};

pub fn save(mut json: serde_json::Value, dir: Option<&Path>) {
    let mut name = String::new();
    if let Some(thing) = json
        .get("blueprint")
        .or(json.get("blueprint_book"))
        .or(json.get("upgrade_planner"))
        .or(json.get("deconstruction_planner"))
        && let Some(label) = thing.get("label")
        && let Some(label) = label.as_str()
    {
        name = label.to_owned();
    } else if let Some(blueprint) = json.get("blueprint")
        && let Some(icons) = blueprint.get("icons")
        && let Some(icons) = icons.as_array()
    {
        for icon in icons {
            if let Some(signal) = icon.get("signal")
                && let Some(signal_name) = signal.get("name")
                && let Some(signal_name) = signal_name.as_str()
            {
                if !name.is_empty() {
                    name.push(' ');
                }
                name.push_str(signal_name);
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
            .flat_map(|e| e.get("name"))
            .flat_map(|e| e.as_str())
            .map(|entity_name| format!("[entity={entity_name}]"));

        let tiles = settings.get("tile_filters");
        let tiles = tiles
            .iter()
            .flat_map(|e| e.as_array())
            .flatten()
            .flat_map(|e| e.get("name"))
            .flat_map(|e| e.as_str())
            .map(|tile_name| format!("[tile={tile_name}]"));

        let mut iter = entities.chain(tiles).fuse();

        for name_part in (&mut iter).take(4) {
            if !name.is_empty() {
                name.push(' ');
            }
            name.push_str(&name_part);
        }
        if iter.next().is_some() {
            name.push_str(" …");
        }
    } else if let Some(blueprint) = json.get("upgrade_planner")
        && let Some(settings) = blueprint.get("settings")
        && let Some(mappers) = settings.get("mappers")
        && let Some(mappers) = mappers.as_array()
    {
        let mut iter = mappers.iter().fuse();
        for mapper in (&mut iter).take(4) {
            if let Some(to) = mapper.get("to")
                && let Some(name_part) = to.get("name")
                && let Some(name_part) = name_part.as_str()
            {
                if !name.is_empty() {
                    name.push(' ');
                }
                if let Some(typ) = to.get("type")
                    && let Some(typ) = typ.as_str()
                    && typ == "entity"
                {
                    name.push_str(&format!("[entity={name_part}]"));
                } else {
                    name.push_str(&name_part);
                }
            }
        }
        if !name.is_empty() {
            name = format!("Upgrade {name}");
        }
    }

    if name.is_empty() {
        name = "Untitled".to_owned()
    };

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
        assert_eq!(files, &["selector-combinator.json"]);
        let written_json =
            std::fs::read_to_string(dir.path().join("selector-combinator.json")).unwrap();
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
        assert_eq!(subsubfiles, ["6 bulk-inserter.json", "book.json"]);

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
}
