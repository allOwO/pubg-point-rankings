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
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgMatchRelationships {
    pub assets: PubgRelationshipList,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PubgIncludedEntity {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub attributes: serde_json::Value,
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

    pub fn get_telemetry(&self, telemetry_url: &str) -> Result<String, AppError> {
        let response = ureq::get(telemetry_url).call().map_err(map_ureq_error)?;
        response
            .into_string()
            .map_err(|error| AppError::Message(format!("failed to read telemetry body: {error}")))
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
            let detail = response
                .into_string()
                .unwrap_or_else(|_| "failed to read response body".to_string());
            AppError::Message(format!("PUBG API request failed ({status}): {detail}"))
        }
        ureq::Error::Transport(transport) => {
            AppError::Message(format!("PUBG API transport error: {transport}"))
        }
    }
}
