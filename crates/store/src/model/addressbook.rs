use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Addressbook {
    pub id: String,
    pub principal: String,
    pub displayname: Option<String>,
    pub description: Option<String>,
    pub deleted_at: Option<NaiveDateTime>,
    pub synctoken: i64,
}

impl Addressbook {
    pub fn format_synctoken(&self) -> String {
        format_synctoken(self.synctoken)
    }
}

// TODO: make nicer
const SYNC_NAMESPACE: &str = "github.com/lennart-k/rustical/ns/";

pub fn format_synctoken(synctoken: i64) -> String {
    format!("{}{}", SYNC_NAMESPACE, synctoken)
}

pub fn parse_synctoken(synctoken: &str) -> Option<i64> {
    if !synctoken.starts_with(SYNC_NAMESPACE) {
        return None;
    }
    let (_, synctoken) = synctoken.split_at(SYNC_NAMESPACE.len());
    synctoken.parse::<i64>().ok()
}
