// opening and closing database connections as they are needed can be costly
// therefore, it is sometimes better to keep a pool of many open connections,
// and have clients use a connection from the pool, then put it back in

use std::env;

use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::{Config, AuthMethod};

#[derive(Debug)]
pub enum DatabaseError {
    BB8(bb8_tiberius::Error)
}

/// creates a DbPool of connections to the database
/// this will return an error if any environment variables are not set or any
/// errors arise while connecting
pub async fn make_db_pool() -> Result<Pool<ConnectionManager>, DatabaseError> {
    let config = create_config_from_env();
    let manager = ConnectionManager::new(config);
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .map_err(DatabaseError::BB8)?;
    Ok(pool)
}

pub fn create_config_from_env() -> Config {
    let mut config = Config::new();
    config.database("RustDB");
    config.trust_cert();

    config.authentication(AuthMethod::sql_server(
        env::var("TIBERIUS_USERNAME").expect("TIBERIUS_USERNAME environment variable should be set"),
        env::var("TIBERIUS_PASSWORD").expect("TIBERIUS_PASSWORD environment variable should be set")
    ));

    config
}