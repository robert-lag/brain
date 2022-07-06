use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String
}

impl GraphNode {
    pub fn from(id: &str, label: &str) -> Self {
        GraphNode {
            id: id.to_string(),
            label: label.to_string()
        }
    }
}