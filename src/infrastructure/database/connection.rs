use std::env;
use tiberius::{Client, Config, AuthMethod, error::Error, EncryptionLevel};
use tokio_util::compat::{TokioAsyncWriteCompatExt, Compat};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

pub fn create_config_from_env() -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = Config::new();
    config.database("RustDB"); // make sure user mapping is set up
    config.trust_cert();
    config.encryption(EncryptionLevel::NotSupported); // TLS is not enabled

    // might be able to remove these
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

// MSSQL is not set up for TCP, but uses a named pipe instead. If unable to
// connect, try some of these:
// * make sure username & password are set
// * make sure user mapping is set up for RustDB
// * make sure the pipe name hasn't changed (see SSMS logs)
pub async fn create_client(
    mssql_config: Config
) -> Result<Client<Compat<NamedPipeClient>>, Error> {
    let pipe_name = r"\\.\pipe\LOCALDB#AC3DE11D\tsql\query"; // todo env var
    let pipe = ClientOptions::new().open(pipe_name)?;
    let connection = Client::connect(mssql_config, pipe.compat_write()).await?;

    Ok(connection)
}