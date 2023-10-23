use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use clap::{Parser, Subcommand};
use quinn::{ClientConfig, Endpoint};
use tracing::info;

use quic_chat::{recv_msg, send_msg, Message};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Post { msg: String },
    Get,
}

impl From<Commands> for Message {
    fn from(val: Commands) -> Self {
        use Commands::*;
        match val {
            Get => Message::GetAll,
            Post { msg } => Message::Post { content: msg },
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )?;

    let cli = Cli::parse();

    let server_addr = "127.0.0.1:5000".parse()?;
    let client_addr = "127.0.0.1:0".parse()?;

    let endpoint = make_client_endpoint(client_addr)?;

    info!("connecting to server...");
    let connection = endpoint.connect(server_addr, "localhost")?.await?;
    info!("connected: addr={}", connection.remote_address());

    info!("accepting bidirectional stream");
    let (mut send_stream, mut recv_stream) = connection.accept_bi().await?;

    let msg: Message = cli.command.into();

    recv_msg(&mut recv_stream).await?;

    match &msg {
        Message::GetAll => {
            info!("waiting for data");

            let mut buf = [0u8; 1024];
            let read_data = recv_stream.read(&mut buf).await?;
            info!("read data: {read_data:?}");

            let messages: Vec<Message> = rmp_serde::from_slice(&buf)?;
            dbg!(messages);
        }
        Message::Post { content: _ } => {
            send_msg(&mut send_stream, msg).await?;
        }
    }

    recv_msg(&mut recv_stream).await?;
    send_stream.finish().await?;
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
