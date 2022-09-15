use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Message {
    nonce: i32,
    command: String,
    #[serde(rename = "dataType", default)]
    data_type: String,
    #[serde(default)]
    data: String,
}

impl Message {
    #[allow(dead_code)]
    pub fn new(nonce: i32, command: &str, data_type: &str, data: &str) -> Self {
        Self {
            nonce,
            command: command.to_owned(),
            data_type: data_type.to_owned(),
            data: data.to_owned(),
        }
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
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
