use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use quic_chat::{ClientToServer, ServerToClient};
use quinn::{Endpoint, ServerConfig};
use tokio::sync::Mutex;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )?;

    let server_addr = "127.0.0.1:5000".parse()?;
    let endpoint = make_server_endpoint(server_addr)?;

    let messages = Arc::new(Mutex::new(Vec::new()));

    info!("waiting for connection...");
    while let Some(conn) = endpoint.accept().await {
        let messages = messages.clone();
        tokio::spawn(async move { handle_connection(conn, messages).await });
    }

    Ok(())
}

async fn handle_connection(
    conn: quinn::Connecting,
    messages: Arc<Mutex<Vec<String>>>,
) -> Result<()> {
    info!("connection accepted: addr={}", conn.remote_address());
    let conn = conn.await?;

    info!("opening bidirectional stream");
    let (mut send_stream, mut recv_stream) = conn.open_bi().await?;

    ServerToClient::Hello.send(&mut send_stream).await?;
    let client_msg = ClientToServer::recv(&mut recv_stream).await?;

    match client_msg {
        ClientToServer::GetAll => {
            let messages = messages.lock().await;
            ServerToClient::Messages(messages.clone())
                .send(&mut send_stream)
                .await?;
        }
        ClientToServer::GetLen => {
            let messages = messages.lock().await;
            ServerToClient::MessagesLen(messages.len())
                .send(&mut send_stream)
                .await?;
        }
        ClientToServer::Post { content } => {
            info!("stored message: {content}");
            let mut messages = messages.lock().await;
            messages.push(content);
            ServerToClient::OK.send(&mut send_stream).await?;
        }
        ClientToServer::Clear => {
            let mut messages = messages.lock().await;
            messages.clear();
            ServerToClient::OK.send(&mut send_stream).await?;
        }
    }

    send_stream.finish().await?;
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
