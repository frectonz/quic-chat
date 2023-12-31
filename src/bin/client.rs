use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use clap::{Parser, Subcommand};
use quic_chat::{client_addr, server_addr, ClientToServer, ServerToClient};
use quinn::{ClientConfig, Endpoint};
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Post { msg: String },
    GetAll,
    GetLen,
    Clear,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )?;

    let cli = Cli::parse();

    let endpoint = make_client_endpoint(client_addr())?;

    info!("connecting to server...");
    let connection = endpoint.connect(server_addr(), "localhost")?.await?;
    info!("connected: addr={}", connection.remote_address());

    info!("accepting bidirectional stream");
    let (mut send_stream, mut recv_stream) = connection.accept_bi().await?;
    let hello_msg = ServerToClient::recv(&mut recv_stream).await?;

    assert!(matches!(hello_msg, ServerToClient::Hello));

    match cli.command {
        Commands::GetAll => {
            ClientToServer::GetAll.send(&mut send_stream).await?;
            send_stream.finish().await?;
            let get_all_msg = ServerToClient::recv(&mut recv_stream).await?;
            assert!(matches!(get_all_msg, ServerToClient::Messages(_)));
            dbg!(get_all_msg);
        }
        Commands::GetLen => {
            ClientToServer::GetLen.send(&mut send_stream).await?;
            send_stream.finish().await?;
            let get_len_msg = ServerToClient::recv(&mut recv_stream).await?;
            assert!(matches!(get_len_msg, ServerToClient::MessagesLen(_)));
            dbg!(get_len_msg);
        }
        Commands::Post { msg } => {
            ClientToServer::Post { content: msg }
                .send(&mut send_stream)
                .await?;
            send_stream.finish().await?;
            let ok_msg = ServerToClient::recv(&mut recv_stream).await?;
            assert!(matches!(ok_msg, ServerToClient::OK));
        }
        Commands::Clear => {
            ClientToServer::Clear.send(&mut send_stream).await?;
            send_stream.finish().await?;
            let ok_msg = ServerToClient::recv(&mut recv_stream).await?;
            assert!(matches!(ok_msg, ServerToClient::OK));
        }
    }

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
