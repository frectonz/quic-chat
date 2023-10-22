use anyhow::Result;
use quinn::{RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    content: String,
}

impl Message {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_owned(),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        Ok(rmp_serde::to_vec(self)?)
    }

    pub fn decode(slice: &[u8]) -> Result<Self> {
        Ok(rmp_serde::from_slice(slice)?)
    }
}

impl From<&str> for Message {
    fn from(content: &str) -> Self {
        Message::new(content)
    }
}

pub async fn send_msg(stream: &mut SendStream, msg: Message) -> Result<()> {
    stream.write_all(&msg.encode()?).await?;
    info!("sent msg: {}", msg.content);
    Ok(())
}

pub async fn recv_msg(stream: &mut RecvStream) -> Result<Message> {
    info!("waiting for data");

    let mut buf = [0u8; 1024];
    let read_data = stream.read(&mut buf).await?;
    info!("read data: {read_data:?}");

    let msg = Message::decode(&buf)?;
    info!("recieved msg: {}", msg.content);
    Ok(msg)
}
