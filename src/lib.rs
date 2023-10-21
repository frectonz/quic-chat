use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<'a> {
    content: &'a str,
}

impl<'a> Message<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        Ok(rmp_serde::to_vec(self)?)
    }

    pub fn decode(slice: &'a [u8]) -> Result<Self> {
        Ok(rmp_serde::from_slice(slice)?)
    }
}
