use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Music {
    pub url: String,
    pub title: Option<String>,
    pub downloaded_at: Option<String>,
}
