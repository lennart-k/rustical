const SYNC_NAMESPACE: &str = "github.com/lennart-k/rustical/ns/";

pub fn format_synctoken(synctoken: i64) -> String {
    format!("{SYNC_NAMESPACE}{synctoken}")
}

pub fn parse_synctoken(synctoken: &str) -> Option<i64> {
    if !synctoken.starts_with(SYNC_NAMESPACE) {
        return None;
    }
    let (_, synctoken) = synctoken.split_at(SYNC_NAMESPACE.len());
    synctoken.parse::<i64>().ok()
}
