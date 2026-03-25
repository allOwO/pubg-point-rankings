use rusqlite::Connection;

use crate::error::AppError;

use super::schema::{DEFAULT_DATA_SQL, INITIAL_SCHEMA_SQL, SCHEMA_VERSION};

fn current_version(connection: &Connection) -> Result<i64, AppError> {
    let result = connection.query_row(
        "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
        [],
        |row| row.get::<_, i64>(0),
    );

    match result {
        Ok(version) => Ok(version),
        Err(rusqlite::Error::SqliteFailure(_, _)) | Err(rusqlite::Error::QueryReturnedNoRows) => {
            Ok(0)
        }
        Err(error) => Err(error.into()),
    }
}

fn set_version(connection: &Connection, version: i64) -> Result<(), AppError> {
    connection.execute(
        "INSERT OR REPLACE INTO schema_version (version, applied_at) VALUES (?1, CURRENT_TIMESTAMP)",
        [version],
    )?;

    Ok(())
}

pub fn bootstrap_database(connection: &Connection) -> Result<(), AppError> {
    if current_version(connection)? == 0 {
        connection.execute_batch(INITIAL_SCHEMA_SQL)?;
        set_version(connection, SCHEMA_VERSION)?;
    }

    connection.execute_batch(DEFAULT_DATA_SQL)?;

    Ok(())
}
