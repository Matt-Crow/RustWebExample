use tiberius::{Client, Config, AuthMethod, error::Error};
use tokio_util::compat::{TokioAsyncWriteCompatExt, Compat};
use tokio::net::TcpStream;

pub async fn create_client(host: &str, port: u16, auth: AuthMethod) -> Result<Client<Compat<TcpStream>>, Error> {
    let mut mssql_config = Config::new();
    mssql_config.host(host);
    mssql_config.port(port);
    mssql_config.authentication(auth);

    let tcp = TcpStream::connect(mssql_config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let connection = Client::connect(mssql_config, tcp.compat_write()).await?;

    Ok(connection)
}