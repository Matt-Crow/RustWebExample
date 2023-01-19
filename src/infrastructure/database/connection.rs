use std::{env, fmt::Display};
use tiberius::{Client, Config, AuthMethod, error::Error, EncryptionLevel};
use tokio_util::compat::{TokioAsyncWriteCompatExt, Compat};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

#[derive(Debug)]
pub enum DatabaseError {
    Tiberius(Error)
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tiberius(inner) => write!(f, "Tiberius error: {}", inner.to_string())
        }
    }
}

pub async fn create_client_from_env() -> Result<Client<Compat<NamedPipeClient>>, DatabaseError> {
    let config = create_config_from_env()?;
    let client = create_client(config).await.map_err(DatabaseError::Tiberius)?;
    Ok(client)
}

pub fn create_config_from_env() -> Result<Config, DatabaseError> {
    let mut config = Config::new();
    config.database("RustDB"); // make sure user mapping is set up
    config.trust_cert();
    config.encryption(EncryptionLevel::NotSupported); // TLS is not enabled

    config.authentication(AuthMethod::sql_server(
        env::var("TIBERIUS_USERNAME").expect("TIBERIUS_USERNAME environment variable should be set"),
        env::var("TIBERIUS_PASSWORD").expect("TIBERIUS_PASSWORD environment variable should be set")
    ));

    Ok(config)
}

// MSSQL is not set up for TCP, but uses a named pipe instead. If unable to
// connect, try some of these:
// * make sure username & password are set
// * make sure user mapping is set up for RustDB
// * make sure the pipe name hasn't changed (see SSMS logs)
pub async fn create_client(
    mssql_config: Config
) -> Result<Client<Compat<NamedPipeClient>>, Error> {
    let pipe_name = env::var("TIBERIUS_PIPE").expect("TIBERIUS_PIPE environment variable should be set");
    let pipe = ClientOptions::new().open(pipe_name)?;
    let connection = Client::connect(mssql_config, pipe.compat_write()).await?;

    Ok(connection)
}