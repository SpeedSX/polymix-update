use tiberius::{SqlBrowser, error::Error};
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub async fn connect(connection_string: &str) -> Result<Client<Compat<TcpStream>>, Error> {
    let config = Config::from_ado_string(&connection_string)?;

    let tcp = TcpStream::connect_named(&config).await?;
    tcp.set_nodelay(true)?;

    Client::connect(config.clone(), tcp.compat_write()).await
}