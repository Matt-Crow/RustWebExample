use std::env;
use tiberius::{Client, Config, AuthMethod, error::Error};
use tokio_util::compat::{TokioAsyncWriteCompatExt, Compat};
use tokio::net::TcpStream;

pub fn create_config_from_env() -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = Config::new();

    config.database("RustDB");

    if let Ok(host) = env::var("TIBERIUS_HOST") {
        config.host(host);
    }

    if let Ok(port) = env::var("TIBERIUS_PORT") {
        config.port(port.parse()?);
    }

    config.authentication(AuthMethod::sql_server(
        env::var("TIBERIUS_USERNAME")?,
        env::var("TIBERIUS_PASSWORD")?
    ));

    Ok(config)
}

pub async fn create_client(mssql_config: Config) -> Result<Client<Compat<TcpStream>>, Error> {
    println!("Connecting to database URL {}...", mssql_config.get_addr());
    
    let tcp = TcpStream::connect(mssql_config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let connection = Client::connect(mssql_config, tcp.compat_write()).await?;

    Ok(connection)
}