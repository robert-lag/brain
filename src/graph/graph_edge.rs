use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

impl GraphEdge {
    pub fn from(id: &str, source: &str, target: &str) -> Self {
        GraphEdge {
            id: id.to_string(),
            source: source.to_string(),
            target: target.to_string(),
        }
    }
}