use serde::{Deserialize, Serialize};
use toasty::Model;

#[derive(Debug, Model)]
pub struct Family {
    #[key]
    #[auto]
    pub id: u64,

    pub name: String,
    pub summary: Option<String>,

    pub deleted_at: Option<jiff::Timestamp>,
}

#[derive(Debug, Serialize)]
pub struct FamilyResponse {
    pub id: u64,
    pub name: String,
    pub summary: Option<String>,
}

impl From<Family> for FamilyResponse {
    fn from(family: Family) -> Self {
        Self {
            id: family.id,
            name: family.name,
            summary: family.summary,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateFamily {
    pub name: String,
    pub summary: Option<String>,
}
