use std::{env, fmt::Display};
use tiberius::{Client, Config, AuthMethod, error::Error};
use tokio_util::compat::{TokioAsyncWriteCompatExt, Compat};
use tokio::net::TcpStream;

#[derive(Debug)]
pub enum DatabaseError {
    Tiberius(Error)
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tiberius(inner) => write!(f, "Tiberius error: {}", inner)
        }
    }
}

pub async fn create_client_from_env() -> Result<Client<Compat<TcpStream>>, DatabaseError> {
    let config = create_config_from_env()?;
    let client = create_client(config).await.map_err(DatabaseError::Tiberius)?;
    Ok(client)
}

pub fn create_config_from_env() -> Result<Config, DatabaseError> {
    let mut config = Config::new();
    config.database("RustDB");
    config.trust_cert();

    config.authentication(AuthMethod::sql_server(
        env::var("TIBERIUS_USERNAME").expect("TIBERIUS_USERNAME environment variable should be set"),
        env::var("TIBERIUS_PASSWORD").expect("TIBERIUS_PASSWORD environment variable should be set")
    ));

    Ok(config)
}

pub async fn create_client(
    mssql_config: Config
) -> Result<Client<Compat<TcpStream>>, Error> {
    let tcp = TcpStream::connect(mssql_config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    let connection = Client::connect(mssql_config, tcp.compat_write()).await?;
    Ok(connection)
}