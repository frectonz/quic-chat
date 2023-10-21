use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use quinn::{Endpoint, ServerConfig};
use tracing::info;

use quic_chat::Message;

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )?;

    let server_addr = "127.0.0.1:5000".parse()?;
    let endpoint = make_server_endpoint(server_addr)?;

    info!("waiting for connection...");
    while let Some(conn) = endpoint.accept().await {
        info!("connection accepted: addr={}", conn.remote_address());
        let conn = conn.await?;

        let (mut send_stream, _) = conn.open_bi().await?;
        let msg = Message::new("Hello, World!").encode()?;
        send_stream.write_all(&msg).await?;
        info!("sent msg: {msg:?}");
        send_stream.finish().await?;
    }

    Ok(())
}

pub fn make_server_endpoint(bind_addr: SocketAddr) -> Result<Endpoint> {
    let server_config = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok(endpoint)
}

/// Returns default server configuration along with its certificate.
pub fn configure_server() -> Result<ServerConfig> {
    let crt = std::fs::read("cert/cert.der")?;
    let key = std::fs::read("cert/key.der")?;

    let priv_key = rustls::PrivateKey(key);
    let cert_chain = vec![rustls::Certificate(crt)];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
    if let Some(transport_config) = Arc::get_mut(&mut server_config.transport) {
        transport_config.max_concurrent_uni_streams(0_u8.into());
    }

    Ok(server_config)
}
