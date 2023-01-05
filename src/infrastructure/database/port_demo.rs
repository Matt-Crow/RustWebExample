// showcases how I figured out which port MSSQL is *NOT* running on
use tokio::net::TcpListener;

pub fn set_up_tcp_listener_on(port: u16) {
    tokio::spawn(async move {
        let address = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&address).await;
        println!("Set up TCP listener on {}", address);

        match listener {
            Ok(listening) => {
                println!("Result: {:#?}", listening);
                match listening.accept().await {
                    Ok(_) => println!("OOPS! Looks like this thread is listening on port {port}! Resetting connection..."),
                    Err(e) => panic!("{}", e)
                }
            },
            Err(e) => panic!("{}", e)
        }        
    });
}