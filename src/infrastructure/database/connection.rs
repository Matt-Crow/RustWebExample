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
        env::var("TIBERIUS_USERNAME").expect("Don't forget to set the TIBERIUS_USERNAME environment variable"),
        env::var("TIBERIUS_PASSWORD").expect("Don't forget to set the TIBERIUS_PASSWORD environment variable")
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