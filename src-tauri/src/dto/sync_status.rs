use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusDto {
    pub is_syncing: bool,
    pub last_sync_at: Option<String>,
    pub current_match_id: Option<String>,
    pub error: Option<String>,
}
