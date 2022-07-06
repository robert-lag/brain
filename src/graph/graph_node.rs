use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphNode {
    pub id: String
}

impl GraphNode {
    pub fn from(id: &str) -> Self {
        GraphNode {
            id: id.to_string()
        }
    }
}