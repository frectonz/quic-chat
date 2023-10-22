use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use quinn::{Endpoint, ServerConfig};
use tracing::info;

use quic_chat::{recv_msg, send_msg};

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )?;

    let server_addr = "127.0.0.1:5000".parse()?;
    let endpoint = make_server_endpoint(server_addr)?;

    let mut messages = Vec::new();

    info!("waiting for connection...");
    while let Some(conn) = endpoint.accept().await {
        info!("connection accepted: addr={}", conn.remote_address());
        let conn = conn.await?;

        info!("opening bidirectional stream");
        let (mut send_stream, mut recv_stream) = conn.open_bi().await?;

        send_msg(&mut send_stream, "Hello client".into()).await?;
        let msg = recv_msg(&mut recv_stream).await?;

        if msg.content == "GET" {
            let data = rmp_serde::to_vec(&messages)?;
            send_stream.write_all(&data).await?;
            info!("sent a list of messages: {}", messages.len());
        } else {
            messages.push(msg);
        }

        send_msg(&mut send_stream, "message received".into()).await?;

        send_stream.finish().await?;
    }

    Ok(())
}

pub fn make_server_endpoint(bind_addr: SocketAddr) -> Result<Endpoint> {
    let server_config = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok(endpoint)
}

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
