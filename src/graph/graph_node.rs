use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub weight: usize,
}

impl GraphNode {
    pub fn from(id: &str, label: &str, weight: usize) -> Self {
        GraphNode {
            id: id.to_string(),
            label: label.to_string(),
            weight,
        }
    }
}
