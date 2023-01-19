
use bb8::Pool;
use bb8_tiberius::ConnectionManager;

use super::connection::create_config_from_env;

#[derive(Debug)]
pub enum DatabaseError {
    Other(String),
    BB8(bb8_tiberius::Error)
}

/// creates a DbPool of connections to the database
/// this will return an error if any environment variables are not set or any
/// errors arise while connecting
pub async fn make_db_pool() -> Result<Pool<ConnectionManager>, DatabaseError> {
    // might not work, as it looks like bb8-tiberius is only implemented for TCP
    // not NamedPipe
    // https://github.com/kardeiz/bb8-tiberius/blob/master/src/lib.rs
    let config = create_config_from_env().map_err(|env_err| DatabaseError::Other(env_err.to_string()))?;

    let manager = ConnectionManager::new(config);
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .map_err(DatabaseError::BB8)?;
    Ok(pool)
}