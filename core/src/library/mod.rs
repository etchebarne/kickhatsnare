use std::{
    fs,
    path::{Path, PathBuf},
};

use rusqlite::{OptionalExtension, params};

use crate::{
    CoreError,
    storage::{Database, database_error},
};

/// Owns global library locations and their persistent records.
#[derive(Debug)]
pub struct Library {
    database: Database,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibrarySnapshot {
    pub pinned_folders: Vec<PinnedFolder>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedFolder {
    pub id: i64,
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<String>,
    pub is_available: bool,
}

impl Library {
    /// Opens the persistent application database in `data_directory`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be initialized.
    pub fn open(data_directory: impl AsRef<Path>) -> Result<Self, CoreError> {
        Ok(Self {
            database: Database::open(data_directory)?,
        })
    }

    pub(crate) fn in_memory() -> Result<Self, CoreError> {
        Ok(Self {
            database: Database::in_memory()?,
        })
    }

    /// Returns every pinned folder in its configured order.
    ///
    /// # Errors
    ///
    /// Returns an error if records cannot be read or an available folder cannot be traversed.
    pub fn snapshot(&self) -> Result<LibrarySnapshot, CoreError> {
        let mut statement = self
            .database
            .connection()
            .prepare("SELECT id, path FROM pinned_folders ORDER BY position, id")
            .map_err(database_error("prepare pinned folder query"))?;
        let records = statement
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(database_error("query pinned folders"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database_error("read pinned folder record"))?;

        let pinned_folders = records
            .into_iter()
            .map(|(id, path)| pinned_folder(id, PathBuf::from(path)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(LibrarySnapshot { pinned_folders })
    }

    /// Pins an existing directory. Pinning the same canonical path again is idempotent.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not an existing directory or cannot be persisted.
    pub fn pin_folder(&mut self, path: impl AsRef<Path>) -> Result<LibrarySnapshot, CoreError> {
        let path = path.as_ref();
        let canonical_path = fs::canonicalize(path).map_err(|error| {
            CoreError::new(format!(
                "failed to resolve pinned folder {}: {error}",
                path.display()
            ))
        })?;
        if !canonical_path.is_dir() {
            return Err(CoreError::new(format!(
                "pinned folder must be a directory: {}",
                canonical_path.display()
            )));
        }
        let path_text = canonical_path.to_str().ok_or_else(|| {
            CoreError::new(format!(
                "pinned folder path must be valid UTF-8: {}",
                canonical_path.display()
            ))
        })?;

        let connection = self.database.connection();
        let existing = connection
            .query_row(
                "SELECT id FROM pinned_folders WHERE path = ?1",
                [path_text],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(database_error("check pinned folder"))?;
        if existing.is_none() {
            connection
                .execute(
                    "INSERT INTO pinned_folders (path, position) \
                     VALUES (?1, COALESCE((SELECT MAX(position) + 1 FROM pinned_folders), 0))",
                    [path_text],
                )
                .map_err(database_error("persist pinned folder"))?;
        }

        self.snapshot()
    }

    /// Removes a pinned folder by its stable database identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot remove the record.
    pub fn unpin_folder(&mut self, id: i64) -> Result<LibrarySnapshot, CoreError> {
        self.database
            .connection()
            .execute("DELETE FROM pinned_folders WHERE id = ?1", params![id])
            .map_err(database_error("remove pinned folder"))?;
        self.snapshot()
    }
}

fn pinned_folder(id: i64, path: PathBuf) -> Result<PinnedFolder, CoreError> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map_or_else(|| path.display().to_string(), str::to_owned);
    let is_available = path.is_dir();
    let files = if is_available {
        collect_library_files(&path)?
    } else {
        Vec::new()
    };

    Ok(PinnedFolder {
        id,
        name,
        path,
        files,
        is_available,
    })
}

fn collect_library_files(root_path: &Path) -> Result<Vec<String>, CoreError> {
    let mut files = Vec::new();
    collect_files(root_path, root_path, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(
    root_path: &Path,
    directory: &Path,
    files: &mut Vec<String>,
) -> Result<(), CoreError> {
    let entries = fs::read_dir(directory).map_err(|error| {
        CoreError::new(format!(
            "failed to read pinned folder {}: {error}",
            directory.display()
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            CoreError::new(format!(
                "failed to read an entry in pinned folder {}: {error}",
                directory.display()
            ))
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            CoreError::new(format!(
                "failed to inspect pinned folder entry {}: {error}",
                path.display()
            ))
        })?;
        let relative_path = path.strip_prefix(root_path).map_err(|error| {
            CoreError::new(format!(
                "failed to resolve pinned folder entry {}: {error}",
                path.display()
            ))
        })?;
        let relative_path = path_string(relative_path)?;

        if file_type.is_dir() {
            files.push(format!("{relative_path}/"));
            collect_files(root_path, &path, files)?;
        } else {
            files.push(relative_path);
        }
    }

    Ok(())
}

fn path_string(path: &Path) -> Result<String, CoreError> {
    let mut components = Vec::new();
    for component in path.components() {
        let component = component.as_os_str().to_str().ok_or_else(|| {
            CoreError::new(format!(
                "pinned folder contains a non-UTF-8 path: {}",
                path.display()
            ))
        })?;
        components.push(component);
    }
    Ok(components.join("/"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::Library;

    #[test]
    fn pinned_folders_persist_in_order() {
        let data = tempdir().expect("data directory should be created");
        let first = tempdir().expect("first folder should be created");
        let second = tempdir().expect("second folder should be created");
        fs::create_dir(first.path().join("Drums")).expect("nested folder should be created");
        fs::write(first.path().join("Drums/kick.wav"), []).expect("sample should be created");

        let mut library = Library::open(data.path()).expect("library should open");
        library
            .pin_folder(first.path())
            .expect("first folder should be pinned");
        library
            .pin_folder(second.path())
            .expect("second folder should be pinned");
        drop(library);

        let library = Library::open(data.path()).expect("library should reopen");
        let snapshot = library.snapshot().expect("snapshot should load");
        assert_eq!(snapshot.pinned_folders.len(), 2);
        assert_eq!(
            snapshot.pinned_folders[0].path,
            first.path().canonicalize().unwrap()
        );
        assert_eq!(
            snapshot.pinned_folders[0].files,
            ["Drums/", "Drums/kick.wav"]
        );
        assert_eq!(
            snapshot.pinned_folders[1].path,
            second.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn duplicate_pins_are_idempotent_and_unpin_persists() {
        let data = tempdir().expect("data directory should be created");
        let folder = tempdir().expect("folder should be created");
        let mut library = Library::open(data.path()).expect("library should open");

        let first = library
            .pin_folder(folder.path())
            .expect("folder should be pinned");
        let second = library
            .pin_folder(folder.path())
            .expect("duplicate pin should succeed");
        assert_eq!(first, second);

        library
            .unpin_folder(first.pinned_folders[0].id)
            .expect("folder should be unpinned");
        drop(library);
        let library = Library::open(data.path()).expect("library should reopen");
        assert!(library.snapshot().unwrap().pinned_folders.is_empty());
    }

    #[test]
    fn missing_folders_remain_pinned() {
        let data = tempdir().expect("data directory should be created");
        let folder_parent = tempdir().expect("folder parent should be created");
        let folder = folder_parent.path().join("Samples");
        fs::create_dir(&folder).expect("folder should be created");
        let mut library = Library::open(data.path()).expect("library should open");
        library
            .pin_folder(&folder)
            .expect("folder should be pinned");
        fs::remove_dir(&folder).expect("folder should be removed");

        let snapshot = library.snapshot().expect("snapshot should load");
        assert_eq!(snapshot.pinned_folders.len(), 1);
        assert!(!snapshot.pinned_folders[0].is_available);
        assert!(snapshot.pinned_folders[0].files.is_empty());
    }
}
