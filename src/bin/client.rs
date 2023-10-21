use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use quinn::{ClientConfig, Endpoint};
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
    let client_addr = "127.0.0.1:0".parse()?;

    let endpoint = make_client_endpoint(client_addr)?;

    info!("connecting to server...");
    let connection = endpoint.connect(server_addr, "localhost")?.await?;
    info!("connected: addr={}", connection.remote_address());

    let (_, mut recv_stream) = connection.accept_bi().await?;
    let data = recv_stream.read_to_end(1024).await?;
    let msg = Message::decode(&data)?;

    info!("got msg = {msg:?}");

    endpoint.wait_idle().await;

    Ok(())
}

struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

fn configure_client() -> ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    ClientConfig::new(Arc::new(crypto))
}

pub fn make_client_endpoint(bind_addr: SocketAddr) -> Result<Endpoint> {
    let client_cfg = configure_client();
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_cfg);
    Ok(endpoint)
}
