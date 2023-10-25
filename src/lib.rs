use anyhow::Result;
use quinn::{RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientToServer {
    GetAll,
    GetLen,
    Post { content: String },
    Clear,
}

impl ClientToServer {
    pub async fn send(&self, stream: &mut SendStream) -> Result<()> {
        stream.write_all(&self.encode()?).await?;
        info!("sent msg from client to server: {self:?}");
        Ok(())
    }

    pub async fn recv(stream: &mut RecvStream) -> Result<Self> {
        info!("waiting for data from client");

        let mut buf = [0u8; 1024];
        let read_data = stream.read(&mut buf).await?;
        info!("read data sent from client: {read_data:?}");

        let msg = ClientToServer::decode(&buf)?;
        info!("recieved msg from client: {msg:?}");

        Ok(msg)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerToClient {
    Hello,
    Messages(Vec<String>),
    MessagesLen(usize),
    OK,
}

impl ServerToClient {
    pub async fn send(&self, stream: &mut SendStream) -> Result<()> {
        stream.write_all(&self.encode()?).await?;
        info!("sent msg from server to client: {self:?}");
        Ok(())
    }

    pub async fn recv(stream: &mut RecvStream) -> Result<Self> {
        info!("waiting for data from server");

        let mut buf = [0u8; 1024];
        let read_data = stream.read(&mut buf).await?;
        info!("read data sent from server: {read_data:?}");

        let msg = ServerToClient::decode(&buf)?;
        info!("recieved msg from server: {msg:?}");

        Ok(msg)
    }
}

pub trait Msgpack
where
    Self: Sized + Serialize + for<'a> Deserialize<'a>,
{
    fn encode(&self) -> Result<Vec<u8>> {
        Ok(rmp_serde::to_vec(self)?)
    }

    fn decode(slice: &[u8]) -> Result<Self> {
        Ok(rmp_serde::from_slice(slice)?)
    }
}

impl Msgpack for ServerToClient {}
impl Msgpack for ClientToServer {}
