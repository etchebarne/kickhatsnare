use std::{fs, path::Path};

use rusqlite::Connection;

use crate::CoreError;

const DATABASE_FILE_NAME: &str = "kickhatsnare.sqlite3";
const MIGRATIONS: &[&str] = &[r"
    CREATE TABLE pinned_folders (
        id INTEGER PRIMARY KEY,
        path TEXT NOT NULL UNIQUE,
        position INTEGER NOT NULL UNIQUE
    );
"];

#[derive(Debug)]
pub(crate) struct Database {
    connection: Connection,
}

impl Database {
    pub(crate) fn open(data_directory: impl AsRef<Path>) -> Result<Self, CoreError> {
        let data_directory = data_directory.as_ref();
        fs::create_dir_all(data_directory).map_err(|error| {
            CoreError::new(format!(
                "failed to create application data directory {}: {error}",
                data_directory.display()
            ))
        })?;
        let connection = Connection::open(data_directory.join(DATABASE_FILE_NAME))
            .map_err(database_error("open application database"))?;
        Self::initialize(connection)
    }

    pub(crate) fn in_memory() -> Result<Self, CoreError> {
        let connection = Connection::open_in_memory()
            .map_err(database_error("open in-memory application database"))?;
        Self::initialize(connection)
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.connection
    }

    fn initialize(mut connection: Connection) -> Result<Self, CoreError> {
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(database_error("enable database foreign keys"))?;
        migrate(&mut connection)?;
        Ok(Self { connection })
    }
}

fn migrate(connection: &mut Connection) -> Result<(), CoreError> {
    let current_version = connection
        .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
        .map_err(database_error("read application database version"))?;
    let latest_version = u32::try_from(MIGRATIONS.len())
        .map_err(|error| CoreError::new(format!("invalid database migration count: {error}")))?;

    if current_version > latest_version {
        return Err(CoreError::new(format!(
            "application database version {current_version} is newer than supported version {latest_version}"
        )));
    }

    for (index, migration) in MIGRATIONS.iter().enumerate().skip(current_version as usize) {
        let next_version = u32::try_from(index + 1)
            .map_err(|error| CoreError::new(format!("invalid database version: {error}")))?;
        let transaction = connection
            .transaction()
            .map_err(database_error("start application database migration"))?;
        transaction
            .execute_batch(migration)
            .map_err(database_error("apply application database migration"))?;
        transaction
            .pragma_update(None, "user_version", next_version)
            .map_err(database_error("update application database version"))?;
        transaction
            .commit()
            .map_err(database_error("commit application database migration"))?;
    }

    Ok(())
}

pub(crate) fn database_error(operation: &'static str) -> impl FnOnce(rusqlite::Error) -> CoreError {
    move |error| CoreError::new(format!("failed to {operation}: {error}"))
}
