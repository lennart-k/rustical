use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Calendar {
    pub id: String,
    pub name: Option<String>,
    pub owner: String,
    pub order: i64,
    pub description: Option<String>,
    pub color: Option<String>,
    pub timezone: Option<String>,
}
