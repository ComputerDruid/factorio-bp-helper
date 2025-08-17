use std::{
    fs::File,
    io::{BufWriter, Write},
    mem,
    path::{Path, PathBuf},
};

pub fn save(mut json: serde_json::Value, dir: Option<&Path>) {
    let mut name = if let Some(thing) = json.get("blueprint").or(json.get("blueprint_book"))
        && let Some(name) = thing.get("label")
        && let Some(name) = name.as_str()
    {
        name.to_owned()
    } else if let Some(blueprint) = json.get("blueprint")
        && let Some(icons) = blueprint.get("icons")
        && let Some(icons) = icons.as_array()
    {
        let mut name = String::new();
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
        if !name.is_empty() {
            name
        } else {
            "Untitled".to_owned()
        }
    } else {
        "Untitled".to_owned()
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
