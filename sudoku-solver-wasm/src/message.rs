use serde::*;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Message {
    nonce: i32,
    command: String,
    #[serde(rename = "dataType")]
    data_type: String,
    data: String,
}

impl Message {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn nonce(&self) -> i32 {
        self.nonce
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn data_type(&self) -> &str {
        &self.data_type
    }

    pub fn data(&self) -> &str {
        &self.data
    }
}
