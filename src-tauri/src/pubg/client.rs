use std::io::Read;

use serde::Deserialize;

use crate::error::AppError;

const PUBG_BASE_URL: &str = "https://api.pubg.com/shards";

#[derive(Debug, Clone)]
pub struct PubgClient {
    api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgPlayer {
    pub id: String,
    pub attributes: PubgPlayerAttributes,
    pub relationships: Option<PubgPlayerRelationships>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgPlayerAttributes {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgPlayerRelationships {
    pub matches: Option<PubgRelationshipList>,
}

impl PubgPlayerRelationships {
    pub fn recent_match_ids(&self, limit: usize) -> Vec<String> {
        self.matches
            .as_ref()
            .map(|matches| {
                matches
                    .data
                    .iter()
                    .filter(|entry| entry.resource_type == "match")
                    .take(limit)
                    .map(|entry| entry.id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgRelationshipList {
    pub data: Vec<PubgResourceRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgResourceRef {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgMatch {
    pub id: String,
    pub attributes: PubgMatchAttributes,
    pub relationships: PubgMatchRelationships,
    #[serde(default)]
    pub included: Vec<PubgIncludedEntity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgMatchAttributes {
    #[serde(rename = "gameMode")]
    pub game_mode: Option<String>,
    #[serde(rename = "mapName")]
    pub map_name: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub duration: Option<i64>,
    #[serde(rename = "matchType")]
    pub match_type: Option<String>,
    #[serde(rename = "isCustomMatch")]
    pub is_custom_match: Option<bool>,
    #[serde(rename = "shardId")]
    pub shard_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgMatchRelationships {
    pub assets: PubgRelationshipList,
    pub rosters: Option<PubgRelationshipList>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgIncludedEntity {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub attributes: serde_json::Value,
    pub relationships: Option<PubgIncludedRelationships>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgIncludedRelationships {
    pub participants: Option<PubgRelationshipList>,
}

#[derive(Debug, Clone)]
pub struct PubgMatchParticipantStats {
    pub pubg_account_id: String,
    pub pubg_player_name: String,
    pub team_id: Option<i64>,
    pub placement: Option<i64>,
    pub damage: f64,
    pub kills: i64,
    pub assists: i64,
    pub revives: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct PubgPlayersResponse {
    data: Vec<PubgPlayer>,
}

#[derive(Debug, Clone, Deserialize)]
struct PubgPlayerResponse {
    data: PubgPlayer,
}

#[derive(Debug, Clone, Deserialize)]
struct PubgMatchResponse {
    data: PubgMatch,
    #[serde(default)]
    included: Vec<PubgIncludedEntity>,
}

#[derive(Debug, Clone, Deserialize)]
struct PubgRosterAttributes {
    stats: Option<PubgRosterStats>,
}

#[derive(Debug, Clone, Deserialize)]
struct PubgRosterStats {
    #[serde(rename = "rank")]
    rank: Option<i64>,
    #[serde(rename = "teamId")]
    team_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
struct PubgParticipantAttributes {
    stats: Option<PubgParticipantStats>,
}

#[derive(Debug, Clone, Deserialize)]
struct PubgParticipantStats {
    #[serde(rename = "playerId")]
    player_id: Option<String>,
    #[serde(rename = "name")]
    name: Option<String>,
    #[serde(rename = "damageDealt")]
    damage_dealt: Option<f64>,
    #[serde(rename = "kills")]
    kills: Option<i64>,
    #[serde(rename = "assists")]
    assists: Option<i64>,
    #[serde(rename = "revives")]
    revives: Option<i64>,
}

impl PubgClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn get_player_by_name(
        &self,
        player_name: &str,
        platform: &str,
    ) -> Result<Option<PubgPlayer>, AppError> {
        let url = format!(
            "{}/{}/players?filter[playerNames]={}",
            PUBG_BASE_URL,
            platform,
            urlencoding::encode(player_name)
        );

        let response: PubgPlayersResponse = self.get_json(&url)?;
        Ok(response.data.into_iter().next())
    }

    pub fn get_player_raw_by_name(
        &self,
        player_name: &str,
        platform: &str,
    ) -> Result<serde_json::Value, AppError> {
        let url = format!(
            "{}/{}/players?filter[playerNames]={}",
            PUBG_BASE_URL,
            platform,
            urlencoding::encode(player_name)
        );

        self.get_json(&url)
    }

    pub fn get_player_by_id(
        &self,
        player_id: &str,
        platform: &str,
    ) -> Result<Option<PubgPlayer>, AppError> {
        let url = format!(
            "{}/{}/players/{}",
            PUBG_BASE_URL,
            platform,
            urlencoding::encode(player_id)
        );

        let response: PubgPlayerResponse = self.get_json(&url)?;
        Ok(Some(response.data))
    }

    pub fn get_recent_matches(
        &self,
        player_id: &str,
        platform: &str,
        limit: usize,
    ) -> Result<Vec<String>, AppError> {
        let player = self
            .get_player_by_id(player_id, platform)?
            .ok_or_else(|| AppError::Message("player not found".to_string()))?;

        let Some(relationships) = player.relationships else {
            return Ok(Vec::new());
        };
        let Some(matches) = relationships.matches else {
            return Ok(Vec::new());
        };

        Ok(matches
            .data
            .into_iter()
            .filter(|entry| entry.resource_type == "match")
            .take(limit)
            .map(|entry| entry.id)
            .collect())
    }

    pub fn get_recent_matches_for_player_name(
        &self,
        player_name: &str,
        platform: &str,
        limit: usize,
    ) -> Result<Vec<String>, AppError> {
        let Some(player) = self.get_player_by_name(player_name, platform)? else {
            return Ok(Vec::new());
        };

        Ok(player
            .relationships
            .as_ref()
            .map(|relationships| relationships.recent_match_ids(limit))
            .unwrap_or_default())
    }

    pub fn get_match(&self, match_id: &str, platform: &str) -> Result<Option<PubgMatch>, AppError> {
        let url = format!(
            "{}/{}/matches/{}",
            PUBG_BASE_URL,
            platform,
            urlencoding::encode(match_id)
        );

        let mut response: PubgMatchResponse = self.get_json(&url)?;
        response.data.included = response.included;
        Ok(Some(response.data))
    }

    pub fn get_match_raw(
        &self,
        match_id: &str,
        platform: &str,
    ) -> Result<serde_json::Value, AppError> {
        let url = format!(
            "{}/{}/matches/{}",
            PUBG_BASE_URL,
            platform,
            urlencoding::encode(match_id)
        );

        self.get_json(&url)
    }

    pub fn get_telemetry_url(&self, match_data: &PubgMatch) -> Option<String> {
        let asset_id = match_data.relationships.assets.data.first()?.id.clone();

        match_data.included.iter().find_map(|included| {
            if included.entity_type != "asset" || included.id != asset_id {
                return None;
            }

            included
                .attributes
                .get("URL")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned)
        })
    }

    pub fn extract_match_participants(
        &self,
        match_data: &PubgMatch,
    ) -> Vec<PubgMatchParticipantStats> {
        let mut roster_lookup: std::collections::HashMap<String, (Option<i64>, Option<i64>)> =
            std::collections::HashMap::new();

        for entity in &match_data.included {
            if entity.entity_type != "roster" {
                continue;
            }

            let roster_attributes =
                serde_json::from_value::<PubgRosterAttributes>(entity.attributes.clone()).ok();
            let team_id = roster_attributes
                .as_ref()
                .and_then(|value| value.stats.as_ref())
                .and_then(|value| value.team_id);
            let placement = roster_attributes
                .as_ref()
                .and_then(|value| value.stats.as_ref())
                .and_then(|value| value.rank);

            let participant_refs = entity
                .relationships
                .as_ref()
                .and_then(|value| value.participants.as_ref())
                .map(|value| value.data.clone())
                .unwrap_or_default();

            for participant_ref in participant_refs {
                roster_lookup.insert(participant_ref.id, (team_id, placement));
            }
        }

        let mut participants = Vec::new();
        for entity in &match_data.included {
            if entity.entity_type != "participant" {
                continue;
            }

            let Ok(participant_attributes) =
                serde_json::from_value::<PubgParticipantAttributes>(entity.attributes.clone())
            else {
                continue;
            };
            let Some(stats) = participant_attributes.stats else {
                continue;
            };

            let Some(player_id) = stats.player_id.map(|value| value.trim().to_string()) else {
                continue;
            };
            if player_id.is_empty() {
                continue;
            }

            let player_name = stats
                .name
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| player_id.clone());
            let (team_id, placement) = roster_lookup
                .get(&entity.id)
                .copied()
                .unwrap_or((None, None));

            participants.push(PubgMatchParticipantStats {
                pubg_account_id: player_id,
                pubg_player_name: player_name,
                team_id,
                placement,
                damage: stats.damage_dealt.unwrap_or(0.0),
                kills: stats.kills.unwrap_or(0),
                assists: stats.assists.unwrap_or(0),
                revives: stats.revives.unwrap_or(0),
            });
        }

        participants
    }

    pub fn get_telemetry(&self, telemetry_url: &str) -> Result<String, AppError> {
        let response = ureq::get(telemetry_url).call().map_err(map_ureq_error)?;
        read_response_string(response.into_reader(), "telemetry body")
    }

    fn get_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, AppError> {
        let response = ureq::get(url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Accept", "application/vnd.api+json")
            .call()
            .map_err(map_ureq_error)?;

        serde_json::from_reader(response.into_reader()).map_err(|error| {
            AppError::Message(format!("failed to parse PUBG API response: {error}"))
        })
    }
}

fn map_ureq_error(error: ureq::Error) -> AppError {
    match error {
        ureq::Error::Status(status, response) => {
            let detail = read_response_string(response.into_reader(), "response body")
                .unwrap_or_else(|_| "failed to read response body".to_string());
            AppError::Message(format!("PUBG API request failed ({status}): {detail}"))
        }
        ureq::Error::Transport(transport) => {
            AppError::Message(format!("PUBG API transport error: {transport}"))
        }
    }
}

fn read_response_string(mut reader: impl Read, label: &str) -> Result<String, AppError> {
    let mut body = String::new();
    reader
        .read_to_string(&mut body)
        .map_err(|error| AppError::Message(format!("failed to read {label}: {error}")))?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::Value;

    use super::PubgClient;
    use crate::{db::connection::open_database, repository::accounts::AccountsRepository};

    #[test]
    #[ignore = "diagnostic live PUBG API probe using active local account"]
    fn inspect_live_pubg_payloads_for_avatar_related_fields() {
        let (connection, db_path) = open_database().expect("database opens");
        let account = AccountsRepository::new(&connection)
            .require_active()
            .expect("active account required");
        let mut report = String::new();

        let client = PubgClient::new(account.pubg_api_key.clone());
        let player_name = account.self_player_name;
        let platform = account.self_platform;

        let header = format!(
            "[avatar_probe] db_path={} player_name={} platform={}\n",
            db_path.display(),
            player_name,
            platform
        );
        report.push_str(&header);
        eprintln!("{header}");

        let player_raw = client
            .get_player_raw_by_name(&player_name, &platform)
            .expect("player raw payload fetches");
        let player_attribute_keys =
            collect_child_object_keys(player_raw.pointer("/data/0/attributes"));
        let player_relationship_keys =
            collect_child_object_keys(player_raw.pointer("/data/0/relationships"));
        let top_level_player_links = player_raw.pointer("/links");
        let player_links = player_raw.pointer("/data/0/links");
        let player_matches_links = player_raw.pointer("/data/0/relationships/matches/links");
        let player_assets_links = player_raw.pointer("/data/0/relationships/assets/links");
        let player_avatar_paths = collect_matching_paths(&player_raw, &[]);
        report.push_str(&format!(
            "[avatar_probe] player attribute keys={player_attribute_keys:?}\n"
        ));
        report.push_str(&format!(
            "[avatar_probe] player relationship keys={player_relationship_keys:?}\n"
        ));
        report.push_str(&format!(
            "[avatar_probe] top-level links={top_level_player_links:?}\n"
        ));
        report.push_str(&format!("[avatar_probe] player links={player_links:?}\n"));
        report.push_str(&format!(
            "[avatar_probe] player matches links={player_matches_links:?}\n"
        ));
        report.push_str(&format!(
            "[avatar_probe] player assets links={player_assets_links:?}\n"
        ));
        report.push_str(&format!(
            "[avatar_probe] player avatar-like paths={player_avatar_paths:?}\n"
        ));
        eprintln!("[avatar_probe] player attribute keys={player_attribute_keys:?}");
        eprintln!("[avatar_probe] player relationship keys={player_relationship_keys:?}");
        eprintln!("[avatar_probe] top-level links={top_level_player_links:?}");
        eprintln!("[avatar_probe] player links={player_links:?}");
        eprintln!("[avatar_probe] player matches links={player_matches_links:?}");
        eprintln!("[avatar_probe] player assets links={player_assets_links:?}");
        eprintln!("[avatar_probe] player avatar-like paths={player_avatar_paths:?}");

        let player = client
            .get_player_by_name(&player_name, &platform)
            .expect("player lookup succeeds")
            .expect("player exists");
        let recent_match_ids = client
            .get_recent_matches(&player.id, &platform, 1)
            .expect("recent matches fetch");

        if let Some(match_id) = recent_match_ids.first() {
            let match_raw = client
                .get_match_raw(match_id, &platform)
                .expect("match raw payload fetches");
            let match_attribute_keys =
                collect_child_object_keys(match_raw.pointer("/data/attributes"));
            let match_included_types = collect_included_types(match_raw.pointer("/included"));
            let match_top_level_links = match_raw.pointer("/links");
            let match_links = match_raw.pointer("/data/links");
            let match_assets_links = match_raw.pointer("/data/relationships/assets/links");
            let match_avatar_paths = collect_matching_paths(&match_raw, &[]);
            report.push_str(&format!(
                "[avatar_probe] probing recent_match_id={match_id}\n"
            ));
            report.push_str(&format!(
                "[avatar_probe] match attribute keys={match_attribute_keys:?}\n"
            ));
            report.push_str(&format!(
                "[avatar_probe] match included entity types={match_included_types:?}\n"
            ));
            report.push_str(&format!(
                "[avatar_probe] match top-level links={match_top_level_links:?}\n"
            ));
            report.push_str(&format!("[avatar_probe] match links={match_links:?}\n"));
            report.push_str(&format!(
                "[avatar_probe] match assets links={match_assets_links:?}\n"
            ));
            report.push_str(&format!(
                "[avatar_probe] match avatar-like paths={match_avatar_paths:?}\n"
            ));
            eprintln!("[avatar_probe] probing recent_match_id={match_id}");
            eprintln!("[avatar_probe] match attribute keys={match_attribute_keys:?}");
            eprintln!("[avatar_probe] match included entity types={match_included_types:?}");
            eprintln!("[avatar_probe] match top-level links={match_top_level_links:?}");
            eprintln!("[avatar_probe] match links={match_links:?}");
            eprintln!("[avatar_probe] match assets links={match_assets_links:?}");
            eprintln!("[avatar_probe] match avatar-like paths={match_avatar_paths:?}");

            if let Some(match_data) = client
                .get_match(match_id, &platform)
                .expect("match fetch succeeds")
            {
                if let Some(telemetry_url) = client.get_telemetry_url(&match_data) {
                    let telemetry = client
                        .get_telemetry(&telemetry_url)
                        .expect("telemetry fetch succeeds");
                    let telemetry_line = format!(
                        "[avatar_probe] telemetry contains avatar={} image={} icon={} profile={} bytes={}\n",
                        telemetry.contains("avatar"),
                        telemetry.contains("image"),
                        telemetry.contains("icon"),
                        telemetry.contains("profile"),
                        telemetry.len()
                    );
                    report.push_str(&telemetry_line);
                    eprintln!("{telemetry_line}");
                }
            }
        }

        std::fs::write("/tmp/pubg-avatar-probe-result.txt", report).expect("probe report writes");
    }

    fn collect_child_object_keys(value: Option<&Value>) -> Vec<String> {
        let Some(Value::Object(object)) = value else {
            return Vec::new();
        };

        let mut keys = object.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    }

    fn collect_included_types(value: Option<&Value>) -> Vec<String> {
        let Some(Value::Array(items)) = value else {
            return Vec::new();
        };

        let mut types = BTreeSet::new();
        for item in items {
            if let Some(entity_type) = item.get("type").and_then(Value::as_str) {
                types.insert(entity_type.to_string());
            }
        }

        types.into_iter().collect()
    }

    fn collect_matching_paths(value: &Value, path: &[String]) -> Vec<String> {
        let mut matches = Vec::new();

        match value {
            Value::Object(object) => {
                for (key, child) in object {
                    let mut next_path = path.to_vec();
                    next_path.push(key.clone());

                    let lowercase_key = key.to_ascii_lowercase();
                    if lowercase_key.contains("avatar")
                        || lowercase_key.contains("image")
                        || lowercase_key.contains("icon")
                        || lowercase_key.contains("profile")
                    {
                        matches.push(next_path.join("."));
                    }

                    matches.extend(collect_matching_paths(child, &next_path));
                }
            }
            Value::Array(items) => {
                for (index, child) in items.iter().enumerate() {
                    let mut next_path = path.to_vec();
                    next_path.push(index.to_string());
                    matches.extend(collect_matching_paths(child, &next_path));
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }

        matches
    }
}
