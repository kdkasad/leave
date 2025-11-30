//
// Copyright (C) 2025 Kian Kasad <kian@kasad.com>
//
// This file is part of Leave.
//
// Leave is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// Leave is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// Leave. If not, see <https://www.gnu.org/licenses/>.
//

use std::{collections::HashSet, path::Path};

use eyre::WrapErr as _;
use serde_json::Value as JsonValue;
use tempfile::{TempDir, tempdir};

/// Helper for creating directory structures from JSON objects.
///
/// # JSON format
///
/// `TestTree` takes a JSON object as input. This object represents the contents
/// of a new temporary directory which will be created. Each field of the object
/// is a file/directory to be created. If the value is `null`, the field
/// represents a file that will be created. If the value is an
/// object, it represents a directory which will be treated recursively. If the
/// value is a string, the field represents a symbolic link and the value is the
/// link target.
pub struct TestTree(TempDir);

type JsonObject = serde_json::Map<String, JsonValue>;

impl TestTree {
    /// Creates a new
    ///
    /// # Panics
    ///
    /// Panics on any underlying error.
    pub fn new(tree: JsonValue) -> TestTree {
        let dir = tempdir().expect("Can't create temporary directory");
        let obj = tree.as_object().expect("Argument must be a JSON object");
        populate_from_object(dir.path(), obj);
        TestTree(dir)
    }

    /// Tests whether the directory is empty
    pub fn is_empty(&self) -> bool {
        self.0
            .path()
            .read_dir()
            .expect("Can't read directory contents")
            .next()
            .is_none()
    }

    /// Returns the path of the temporary directory.
    pub fn path(&self) -> &Path {
        self.0.path()
    }

    /// Returns a set of the names of the directory's contents. Does not descend into directories.
    pub fn contents(&self) -> HashSet<String> {
        self.0
            .path()
            .read_dir()
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect()
    }
}

fn populate_from_object(dir: &Path, obj: &JsonObject) {
    for (key, value) in obj {
        let path = dir.join(key);
        match value {
            JsonValue::String(dest) => std::os::unix::fs::symlink(dest, &path)
                .wrap_err_with(|| format!("Can't link {} -> {}", path.display(), dest))
                .unwrap(),
            JsonValue::Null => std::fs::write(&path, "")
                .wrap_err_with(|| format!("Can't write to {}", path.display()))
                .unwrap(),
            JsonValue::Object(inner) => {
                std::fs::create_dir(&path)
                    .wrap_err_with(|| format!("Can't create directory {}", path.display()))
                    .unwrap();
                populate_from_object(&path, inner);
            }
            _ => panic!("Field value must be a string or an object"),
        }
    }
}
