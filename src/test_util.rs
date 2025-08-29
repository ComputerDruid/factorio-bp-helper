use std::{ffi::OsString, fs::read_dir, path::Path};

#[track_caller]
pub(crate) fn read_dir_unwrap(path: &Path) -> Vec<OsString> {
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
