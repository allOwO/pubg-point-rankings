use serde::Deserialize;

use crate::{engine::calculator::PlayerStats, error::AppError};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "_T")]
pub enum TelemetryEvent {
    #[serde(rename = "LogMatchStart")]
    MatchStart {
        characters: Vec<MatchCharacterStart>,
    },
    #[serde(rename = "LogPlayerTakeDamage")]
    PlayerTakeDamage {
        attacker: Option<PlayerIdentity>,
        victim: Option<PlayerIdentity>,
        damage: f64,
    },
    #[serde(rename = "LogPlayerKillV2")]
    PlayerKillV2 { killer: Option<PlayerIdentity> },
    #[serde(rename = "LogPlayerRevive")]
    PlayerRevive { reviver: Option<PlayerIdentity> },
    #[serde(rename = "LogMatchEnd")]
    MatchEnd { characters: Vec<MatchCharacterEnd> },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerIdentity {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchCharacterStart {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub name: String,
    #[serde(rename = "teamId")]
    pub team_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchCharacterEnd {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "ranking")]
    pub ranking: Option<i64>,
}

#[derive(Debug, Clone)]
struct PlayerStatsAccumulator {
    account_id: String,
    name: String,
    team_id: Option<i64>,
    damage: f64,
    kills: i64,
    revives: i64,
    placement: Option<i64>,
}

pub fn parse_telemetry(json_data: &str) -> Result<Vec<TelemetryEvent>, AppError> {
    serde_json::from_str::<Vec<TelemetryEvent>>(json_data)
        .map_err(|error| AppError::Message(format!("failed to parse telemetry data: {error}")))
}

pub fn aggregate_player_stats(events: &[TelemetryEvent]) -> Vec<PlayerStats> {
    let mut stats_map: std::collections::HashMap<String, PlayerStatsAccumulator> =
        std::collections::HashMap::new();

    for event in events {
        if let TelemetryEvent::MatchStart { characters } = event {
            for character in characters {
                stats_map
                    .entry(character.account_id.clone())
                    .or_insert(PlayerStatsAccumulator {
                        account_id: character.account_id.clone(),
                        name: character.name.clone(),
                        team_id: Some(character.team_id),
                        damage: 0.0,
                        kills: 0,
                        revives: 0,
                        placement: None,
                    });
            }
        }
    }

    for event in events {
        match event {
            TelemetryEvent::PlayerTakeDamage {
                attacker,
                victim,
                damage,
            } => {
                if let Some(attacker_identity) = attacker {
                    let should_count = victim.as_ref().is_some_and(|victim_identity| {
                        victim_identity.account_id != attacker_identity.account_id
                    });

                    if should_count {
                        let attacker_stats = stats_map
                            .entry(attacker_identity.account_id.clone())
                            .or_insert(PlayerStatsAccumulator {
                                account_id: attacker_identity.account_id.clone(),
                                name: attacker_identity.name.clone(),
                                team_id: None,
                                damage: 0.0,
                                kills: 0,
                                revives: 0,
                                placement: None,
                            });
                        attacker_stats.damage += *damage;
                    }
                }
            }
            TelemetryEvent::PlayerKillV2 { killer } => {
                if let Some(killer_identity) = killer {
                    let killer_stats = stats_map
                        .entry(killer_identity.account_id.clone())
                        .or_insert(PlayerStatsAccumulator {
                            account_id: killer_identity.account_id.clone(),
                            name: killer_identity.name.clone(),
                            team_id: None,
                            damage: 0.0,
                            kills: 0,
                            revives: 0,
                            placement: None,
                        });
                    killer_stats.kills += 1;
                }
            }
            TelemetryEvent::PlayerRevive { reviver } => {
                if let Some(reviver_identity) = reviver {
                    let reviver_stats = stats_map
                        .entry(reviver_identity.account_id.clone())
                        .or_insert(PlayerStatsAccumulator {
                            account_id: reviver_identity.account_id.clone(),
                            name: reviver_identity.name.clone(),
                            team_id: None,
                            damage: 0.0,
                            kills: 0,
                            revives: 0,
                            placement: None,
                        });
                    reviver_stats.revives += 1;
                }
            }
            TelemetryEvent::MatchEnd { characters } => {
                for character in characters {
                    if let Some(stats) = stats_map.get_mut(&character.account_id) {
                        stats.placement = character.ranking;
                    }
                }
            }
            TelemetryEvent::MatchStart { .. } | TelemetryEvent::Other => {}
        }
    }

    stats_map
        .into_values()
        .map(|stats| PlayerStats {
            pubg_account_id: stats.account_id,
            pubg_player_name: stats.name,
            damage: (stats.damage * 10.0).round() / 10.0,
            kills: stats.kills,
            revives: stats.revives,
            team_id: stats.team_id,
            placement: stats.placement,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{aggregate_player_stats, parse_telemetry};

    #[test]
    fn parses_and_aggregates_basic_stats() {
        let json = r#"[
          {"_T":"LogMatchStart","characters":[{"accountId":"a1","name":"self","teamId":10},{"accountId":"a2","name":"mate","teamId":10}]},
          {"_T":"LogPlayerTakeDamage","attacker":{"accountId":"a1","name":"self"},"victim":{"accountId":"a2","name":"mate"},"damage":99.4},
          {"_T":"LogPlayerKillV2","killer":{"accountId":"a1","name":"self"}},
          {"_T":"LogPlayerRevive","reviver":{"accountId":"a2","name":"mate"}},
          {"_T":"LogMatchEnd","characters":[{"accountId":"a1","ranking":1},{"accountId":"a2","ranking":1}]}
        ]"#;

        let events = parse_telemetry(json).expect("telemetry parses");
        let stats = aggregate_player_stats(&events);

        let self_stats = stats
            .iter()
            .find(|entry| entry.pubg_account_id == "a1")
            .expect("self stats present");
        assert_eq!(self_stats.kills, 1);
        assert_eq!(self_stats.damage, 99.4);
        assert_eq!(self_stats.placement, Some(1));
    }
}
