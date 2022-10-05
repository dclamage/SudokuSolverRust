use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CanceledResponse {
    pub nonce: i32,
    #[serde(rename = "type")]
    pub response_type: String,
}

impl CanceledResponse {
    pub fn new(nonce: i32) -> Self {
        Self { nonce, response_type: "canceled".to_owned() }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InvalidResponse {
    pub nonce: i32,
    #[serde(rename = "type")]
    pub response_type: String,
    pub message: String,
}

impl InvalidResponse {
    pub fn new(nonce: i32, message: &str) -> Self {
        Self { nonce, response_type: "invalid".to_owned(), message: message.to_owned() }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DebugLogResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub message: String,
}

impl DebugLogResponse {
    pub fn new(message: &str) -> Self {
        Self { response_type: "debuglog".to_owned(), message: message.to_owned() }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TrueCandidatesResponse {
    pub nonce: i32,
    #[serde(rename = "type")]
    pub response_type: String,
    #[serde(rename = "solutionsPerCandidate")]
    pub solutions_per_candidate: Vec<i32>,
}

impl TrueCandidatesResponse {
    pub fn new(nonce: i32, solutions_per_candidate: &[i32]) -> Self {
        Self {
            nonce,
            response_type: "truecandidates".to_owned(),
            solutions_per_candidate: solutions_per_candidate.to_owned(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SolvedResponse {
    pub nonce: i32,
    #[serde(rename = "type")]
    pub response_type: String,
    pub solution: Vec<i32>,
}

impl SolvedResponse {
    pub fn new(nonce: i32, solution: &[i32]) -> Self {
        Self { nonce, response_type: "solved".to_owned(), solution: solution.to_owned() }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CountResponse {
    pub nonce: i32,
    #[serde(rename = "type")]
    pub response_type: String,
    pub count: u64,
    #[serde(rename = "inProgress")]
    pub in_progress: bool,
}

impl CountResponse {
    pub fn new(nonce: i32, count: u64, in_progress: bool) -> Self {
        Self { nonce, response_type: "count".to_owned(), count, in_progress }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LogicalCell {
    pub value: i32,
    pub candidates: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LogicalResponse {
    pub nonce: i32,
    #[serde(rename = "type")]
    pub response_type: String,
    pub cells: Vec<LogicalCell>,
    pub message: String,
    #[serde(rename = "isValid")]
    pub is_valid: bool,
}

impl LogicalResponse {
    pub fn new(nonce: i32, cells: &[LogicalCell], message: &str, is_valid: bool) -> Self {
        let mut message = message.to_owned();
        if !message.ends_with('\n') {
            message.push('\n');
        }
        Self { nonce, response_type: "logical".to_owned(), cells: cells.to_owned(), message, is_valid }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
