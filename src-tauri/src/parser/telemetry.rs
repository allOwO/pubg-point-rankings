use std::collections::HashMap;

use serde::Deserialize;

use crate::{engine::calculator::PlayerStats, error::AppError};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "_T")]
pub enum TelemetryEvent {
    #[serde(rename = "LogMatchStart")]
    MatchStart {
        #[serde(rename = "_D")]
        event_at: Option<String>,
        characters: Vec<MatchCharacterStart>,
    },
    #[serde(rename = "LogPlayerTakeDamage")]
    PlayerTakeDamage {
        #[serde(rename = "_D")]
        event_at: Option<String>,
        attacker: Option<PlayerIdentity>,
        victim: Option<PlayerIdentity>,
        damage: f64,
        #[serde(rename = "damageTypeCategory")]
        damage_type_category: Option<String>,
        #[serde(rename = "damageCauserName")]
        damage_causer_name: Option<String>,
    },
    #[serde(rename = "LogPlayerKillV2")]
    PlayerKillV2 {
        #[serde(rename = "_D")]
        event_at: Option<String>,
        killer: Option<PlayerIdentity>,
        victim: Option<PlayerIdentity>,
        finisher: Option<PlayerIdentity>,
        #[serde(rename = "dBNOMaker")]
        dbno_maker: Option<PlayerIdentity>,
        #[serde(rename = "assists_AccountId", default)]
        assists_account_ids: Vec<String>,
        #[serde(rename = "damageTypeCategory")]
        damage_type_category: Option<String>,
        #[serde(rename = "damageCauserName")]
        damage_causer_name: Option<String>,
        #[serde(rename = "killerDamageInfo")]
        killer_damage_info: Option<DamageInfo>,
        #[serde(rename = "finishDamageInfo")]
        finish_damage_info: Option<DamageInfo>,
    },
    #[serde(rename = "LogPlayerMakeGroggy")]
    PlayerMakeGroggy {
        #[serde(rename = "_D")]
        event_at: Option<String>,
        attacker: Option<PlayerIdentity>,
        victim: Option<PlayerIdentity>,
        #[serde(rename = "damageTypeCategory")]
        damage_type_category: Option<String>,
        #[serde(rename = "damageCauserName")]
        damage_causer_name: Option<String>,
    },
    #[serde(rename = "LogPlayerRevive")]
    PlayerRevive {
        #[serde(rename = "_D")]
        event_at: Option<String>,
        reviver: Option<PlayerIdentity>,
        victim: Option<PlayerIdentity>,
    },
    #[serde(rename = "LogMatchEnd")]
    MatchEnd {
        #[serde(rename = "_D")]
        event_at: Option<String>,
        characters: Vec<MatchCharacterEnd>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DamageInfo {
    #[serde(rename = "damageCauserName")]
    pub damage_causer_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerIdentity {
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchCharacterStart {
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "teamId")]
    pub team_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchCharacterEnd {
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    #[serde(rename = "ranking")]
    pub ranking: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct TelemetryDamageEvent {
    pub attacker_account_id: Option<String>,
    pub attacker_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub damage: f64,
    pub damage_type_category: Option<String>,
    pub damage_causer_name: Option<String>,
    pub event_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TelemetryKillEvent {
    pub killer_account_id: Option<String>,
    pub killer_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub assistant_account_id: Option<String>,
    pub damage_type_category: Option<String>,
    pub damage_causer_name: Option<String>,
    pub event_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TelemetryKnockEvent {
    pub attacker_account_id: Option<String>,
    pub attacker_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub damage_type_category: Option<String>,
    pub damage_causer_name: Option<String>,
    pub event_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TelemetryReviveEvent {
    pub reviver_account_id: Option<String>,
    pub reviver_name: Option<String>,
    pub victim_account_id: Option<String>,
    pub victim_name: Option<String>,
    pub event_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TelemetryWeaponDamageStat {
    pub pubg_account_id: String,
    pub pubg_player_name: String,
    pub weapon_name: String,
    pub total_damage: f64,
}

#[derive(Debug, Clone)]
pub struct TelemetryMatchDetail {
    pub player_stats: Vec<PlayerStats>,
    pub damage_events: Vec<TelemetryDamageEvent>,
    pub kill_events: Vec<TelemetryKillEvent>,
    pub knock_events: Vec<TelemetryKnockEvent>,
    pub revive_events: Vec<TelemetryReviveEvent>,
    pub weapon_damage_stats: Vec<TelemetryWeaponDamageStat>,
    pub match_start_at: Option<String>,
    pub match_end_at: Option<String>,
}

#[derive(Debug, Clone)]
struct PlayerStatsAccumulator {
    account_id: String,
    name: String,
    team_id: Option<i64>,
    damage: f64,
    kills: i64,
    assists: i64,
    revives: i64,
    placement: Option<i64>,
}

#[derive(Debug, Clone)]
struct WeaponDamageAccumulator {
    pubg_account_id: String,
    pubg_player_name: String,
    weapon_name: String,
    total_damage: f64,
}

pub fn parse_telemetry(json_data: &str) -> Result<Vec<TelemetryEvent>, AppError> {
    serde_json::from_str::<Vec<TelemetryEvent>>(json_data)
        .map_err(|error| AppError::Message(format!("failed to parse telemetry data: {error}")))
}

pub fn aggregate_player_stats(events: &[TelemetryEvent]) -> Vec<PlayerStats> {
    parse_match_detail(events).player_stats
}

pub fn parse_match_detail(events: &[TelemetryEvent]) -> TelemetryMatchDetail {
    let mut stats_map: HashMap<String, PlayerStatsAccumulator> = HashMap::new();
    let mut damage_events = Vec::new();
    let mut kill_events = Vec::new();
    let mut knock_events = Vec::new();
    let mut revive_events = Vec::new();
    let mut weapon_damage: HashMap<(String, String), WeaponDamageAccumulator> = HashMap::new();
    let mut player_names_by_account_id: HashMap<String, String> = HashMap::new();

    let mut match_start_at = None;
    let mut match_end_at = None;

    for event in events {
        if let TelemetryEvent::MatchStart {
            event_at,
            characters,
        } = event
        {
            if match_start_at.is_none() {
                match_start_at = normalized_timestamp(event_at.as_deref());
            }

            for character in characters {
                let Some(account_id) = normalized_account_id(character.account_id.as_deref())
                else {
                    continue;
                };
                let name = normalized_player_name(character.name.as_deref(), &account_id);
                player_names_by_account_id.insert(account_id.clone(), name.clone());

                stats_map
                    .entry(account_id.clone())
                    .or_insert(PlayerStatsAccumulator {
                        account_id,
                        name,
                        team_id: character.team_id,
                        damage: 0.0,
                        kills: 0,
                        assists: 0,
                        revives: 0,
                        placement: None,
                    });
            }
        }
    }

    for event in events {
        match event {
            TelemetryEvent::PlayerTakeDamage {
                event_at,
                attacker,
                victim,
                damage,
                damage_type_category,
                damage_causer_name,
            } => {
                let attacker_account_id = attacker
                    .as_ref()
                    .and_then(|identity| normalized_account_id(identity.account_id.as_deref()));
                let victim_account_id = victim
                    .as_ref()
                    .and_then(|identity| normalized_account_id(identity.account_id.as_deref()));
                let attacker_name = attacker_account_id.as_ref().map(|account_id| {
                    let name = normalized_player_name(
                        attacker
                            .as_ref()
                            .and_then(|identity| identity.name.as_deref()),
                        account_id,
                    );
                    player_names_by_account_id.insert(account_id.clone(), name.clone());
                    name
                });
                let victim_name = victim_account_id.as_ref().map(|account_id| {
                    let name = normalized_player_name(
                        victim
                            .as_ref()
                            .and_then(|identity| identity.name.as_deref()),
                        account_id,
                    );
                    player_names_by_account_id.insert(account_id.clone(), name.clone());
                    name
                });

                if let Some(account_id) = attacker_account_id.as_ref() {
                    let should_count = victim_account_id
                        .as_ref()
                        .is_some_and(|victim_id| victim_id != account_id);
                    if should_count {
                        let stats =
                            stats_map
                                .entry(account_id.clone())
                                .or_insert(PlayerStatsAccumulator {
                                    account_id: account_id.clone(),
                                    name: attacker_name
                                        .clone()
                                        .unwrap_or_else(|| account_id.clone()),
                                    team_id: None,
                                    damage: 0.0,
                                    kills: 0,
                                    assists: 0,
                                    revives: 0,
                                    placement: None,
                                });
                        stats.damage += *damage;

                        let weapon_name = display_damage_causer_name(
                            damage_causer_name.as_deref(),
                            damage_type_category.as_deref(),
                        );
                        let weapon_key = (account_id.clone(), weapon_name.clone());
                        let entry =
                            weapon_damage
                                .entry(weapon_key)
                                .or_insert(WeaponDamageAccumulator {
                                    pubg_account_id: account_id.clone(),
                                    pubg_player_name: attacker_name
                                        .clone()
                                        .unwrap_or_else(|| account_id.clone()),
                                    weapon_name,
                                    total_damage: 0.0,
                                });
                        entry.total_damage += *damage;
                    }
                }

                damage_events.push(TelemetryDamageEvent {
                    attacker_account_id,
                    attacker_name,
                    victim_account_id,
                    victim_name,
                    damage: round_damage(*damage),
                    damage_type_category: normalized_text(damage_type_category.as_deref()),
                    damage_causer_name: Some(display_damage_causer_name(
                        damage_causer_name.as_deref(),
                        damage_type_category.as_deref(),
                    )),
                    event_at: normalized_timestamp(event_at.as_deref()),
                });
            }
            TelemetryEvent::PlayerKillV2 {
                event_at,
                killer,
                victim,
                finisher,
                dbno_maker,
                assists_account_ids,
                damage_type_category,
                damage_causer_name,
                killer_damage_info,
                finish_damage_info,
            } => {
                let killer_account_id = killer
                    .as_ref()
                    .and_then(|identity| normalized_account_id(identity.account_id.as_deref()))
                    .or_else(|| {
                        dbno_maker.as_ref().and_then(|identity| {
                            normalized_account_id(identity.account_id.as_deref())
                        })
                    })
                    .or_else(|| {
                        finisher.as_ref().and_then(|identity| {
                            normalized_account_id(identity.account_id.as_deref())
                        })
                    });
                let victim_account_id = victim
                    .as_ref()
                    .and_then(|identity| normalized_account_id(identity.account_id.as_deref()));
                let killer_name = killer_account_id.as_ref().map(|account_id| {
                    let name = normalized_player_name(
                        killer
                            .as_ref()
                            .and_then(|identity| identity.name.as_deref())
                            .or_else(|| {
                                dbno_maker
                                    .as_ref()
                                    .and_then(|identity| identity.name.as_deref())
                            })
                            .or_else(|| {
                                finisher
                                    .as_ref()
                                    .and_then(|identity| identity.name.as_deref())
                            }),
                        account_id,
                    );
                    player_names_by_account_id.insert(account_id.clone(), name.clone());
                    name
                });
                let victim_name = victim_account_id.as_ref().map(|account_id| {
                    let name = normalized_player_name(
                        victim
                            .as_ref()
                            .and_then(|identity| identity.name.as_deref()),
                        account_id,
                    );
                    player_names_by_account_id.insert(account_id.clone(), name.clone());
                    name
                });
                let assistant_account_id = assists_account_ids
                    .iter()
                    .find_map(|account_id| normalized_account_id(Some(account_id.as_str())));
                let damage_source = display_damage_causer_name(
                    finish_damage_info
                        .as_ref()
                        .and_then(|info| info.damage_causer_name.as_deref())
                        .or_else(|| {
                            killer_damage_info
                                .as_ref()
                                .and_then(|info| info.damage_causer_name.as_deref())
                        })
                        .or_else(|| damage_causer_name.as_deref()),
                    damage_type_category.as_deref(),
                );

                if let Some(account_id) = killer_account_id.as_ref() {
                    let stats =
                        stats_map
                            .entry(account_id.clone())
                            .or_insert(PlayerStatsAccumulator {
                                account_id: account_id.clone(),
                                name: killer_name.clone().unwrap_or_else(|| account_id.clone()),
                                team_id: None,
                                damage: 0.0,
                                kills: 0,
                                assists: 0,
                                revives: 0,
                                placement: None,
                            });
                    stats.kills += 1;
                }

                for assistant_account_id in assists_account_ids
                    .iter()
                    .filter_map(|account_id| normalized_account_id(Some(account_id.as_str())))
                {
                    let assistant_name = player_names_by_account_id
                        .get(&assistant_account_id)
                        .cloned()
                        .unwrap_or_else(|| assistant_account_id.clone());
                    let stats = stats_map.entry(assistant_account_id.clone()).or_insert(
                        PlayerStatsAccumulator {
                            account_id: assistant_account_id.clone(),
                            name: assistant_name,
                            team_id: None,
                            damage: 0.0,
                            kills: 0,
                            assists: 0,
                            revives: 0,
                            placement: None,
                        },
                    );
                    stats.assists += 1;
                }

                kill_events.push(TelemetryKillEvent {
                    killer_account_id,
                    killer_name,
                    victim_account_id,
                    victim_name,
                    assistant_account_id,
                    damage_type_category: normalized_text(damage_type_category.as_deref()),
                    damage_causer_name: Some(damage_source),
                    event_at: normalized_timestamp(event_at.as_deref()),
                });
            }
            TelemetryEvent::PlayerMakeGroggy {
                event_at,
                attacker,
                victim,
                damage_type_category,
                damage_causer_name,
            } => {
                let attacker_account_id = attacker
                    .as_ref()
                    .and_then(|identity| normalized_account_id(identity.account_id.as_deref()));
                let victim_account_id = victim
                    .as_ref()
                    .and_then(|identity| normalized_account_id(identity.account_id.as_deref()));
                let attacker_name = attacker_account_id.as_ref().map(|account_id| {
                    let name = normalized_player_name(
                        attacker
                            .as_ref()
                            .and_then(|identity| identity.name.as_deref()),
                        account_id,
                    );
                    player_names_by_account_id.insert(account_id.clone(), name.clone());
                    name
                });
                let victim_name = victim_account_id.as_ref().map(|account_id| {
                    let name = normalized_player_name(
                        victim
                            .as_ref()
                            .and_then(|identity| identity.name.as_deref()),
                        account_id,
                    );
                    player_names_by_account_id.insert(account_id.clone(), name.clone());
                    name
                });

                knock_events.push(TelemetryKnockEvent {
                    attacker_account_id,
                    attacker_name,
                    victim_account_id,
                    victim_name,
                    damage_type_category: normalized_text(damage_type_category.as_deref()),
                    damage_causer_name: Some(display_damage_causer_name(
                        damage_causer_name.as_deref(),
                        damage_type_category.as_deref(),
                    )),
                    event_at: normalized_timestamp(event_at.as_deref()),
                });
            }
            TelemetryEvent::PlayerRevive {
                event_at,
                reviver,
                victim,
            } => {
                let reviver_account_id = reviver
                    .as_ref()
                    .and_then(|identity| normalized_account_id(identity.account_id.as_deref()));
                let victim_account_id = victim
                    .as_ref()
                    .and_then(|identity| normalized_account_id(identity.account_id.as_deref()));
                let reviver_name = reviver_account_id.as_ref().map(|account_id| {
                    let name = normalized_player_name(
                        reviver
                            .as_ref()
                            .and_then(|identity| identity.name.as_deref()),
                        account_id,
                    );
                    player_names_by_account_id.insert(account_id.clone(), name.clone());
                    name
                });
                let victim_name = victim_account_id.as_ref().map(|account_id| {
                    let name = normalized_player_name(
                        victim
                            .as_ref()
                            .and_then(|identity| identity.name.as_deref()),
                        account_id,
                    );
                    player_names_by_account_id.insert(account_id.clone(), name.clone());
                    name
                });

                if let Some(account_id) = reviver_account_id.as_ref() {
                    let stats =
                        stats_map
                            .entry(account_id.clone())
                            .or_insert(PlayerStatsAccumulator {
                                account_id: account_id.clone(),
                                name: reviver_name.clone().unwrap_or_else(|| account_id.clone()),
                                team_id: None,
                                damage: 0.0,
                                kills: 0,
                                assists: 0,
                                revives: 0,
                                placement: None,
                            });
                    stats.revives += 1;
                }

                revive_events.push(TelemetryReviveEvent {
                    reviver_account_id,
                    reviver_name,
                    victim_account_id,
                    victim_name,
                    event_at: normalized_timestamp(event_at.as_deref()),
                });
            }
            TelemetryEvent::MatchEnd {
                event_at,
                characters,
            } => {
                match_end_at = normalized_timestamp(event_at.as_deref());

                for character in characters {
                    let Some(account_id) = normalized_account_id(character.account_id.as_deref())
                    else {
                        continue;
                    };

                    if let Some(stats) = stats_map.get_mut(&account_id) {
                        stats.placement = character.ranking;
                    }
                }
            }
            TelemetryEvent::MatchStart { .. } | TelemetryEvent::Other => {}
        }
    }

    let mut player_stats = stats_map
        .into_values()
        .map(|stats| PlayerStats {
            pubg_account_id: stats.account_id,
            pubg_player_name: stats.name,
            damage: round_damage(stats.damage),
            kills: stats.kills,
            assists: stats.assists,
            revives: stats.revives,
            team_id: stats.team_id,
            placement: stats.placement,
        })
        .collect::<Vec<_>>();
    player_stats.sort_by(|left, right| {
        right
            .kills
            .cmp(&left.kills)
            .then_with(|| {
                right
                    .damage
                    .partial_cmp(&left.damage)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.pubg_player_name.cmp(&right.pubg_player_name))
    });

    let mut weapon_damage_stats = weapon_damage.into_values().collect::<Vec<_>>();
    weapon_damage_stats.sort_by(|left, right| {
        right
            .total_damage
            .partial_cmp(&left.total_damage)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.pubg_player_name.cmp(&right.pubg_player_name))
            .then_with(|| left.weapon_name.cmp(&right.weapon_name))
    });

    TelemetryMatchDetail {
        player_stats,
        damage_events,
        kill_events,
        knock_events,
        revive_events,
        weapon_damage_stats: weapon_damage_stats
            .into_iter()
            .map(|entry| TelemetryWeaponDamageStat {
                pubg_account_id: entry.pubg_account_id,
                pubg_player_name: entry.pubg_player_name,
                weapon_name: entry.weapon_name,
                total_damage: round_damage(entry.total_damage),
            })
            .collect(),
        match_start_at,
        match_end_at,
    }
}

fn normalized_account_id(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn normalized_player_name(value: Option<&str>, fallback_account_id: &str) -> String {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        fallback_account_id.to_string()
    } else {
        value.to_string()
    }
}

fn normalized_text(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn normalized_timestamp(value: Option<&str>) -> Option<String> {
    normalized_text(value)
}

pub fn display_damage_causer_name(
    damage_causer_name: Option<&str>,
    fallback: Option<&str>,
) -> String {
    normalized_text(damage_causer_name)
        .map(|value| friendly_damage_causer_name(&value).unwrap_or(value))
        .or_else(|| normalized_text(fallback))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn friendly_damage_causer_name(value: &str) -> Option<String> {
    let display_name = match value {
        "AIPawn_Base_Female_C" => "AI Player",
        "AIPawn_Base_Male_C" => "AI Player",
        "AirBoat_V2_C" => "Airboat",
        "AquaRail_A_01_C" => "Aquarail",
        "AquaRail_A_02_C" => "Aquarail",
        "AquaRail_A_03_C" => "Aquarail",
        "BP_ATV_C" => "Quad",
        "BP_BearV2_C" => "Bear",
        "BP_BRDM_C" => "BRDM-2",
        "BP_Bicycle_C" => "Mountain Bike",
        "BP_Blanc_C" => "Coupe SUV",
        "BP_CoupeRB_C" => "Coupe RB",
        "BP_DO_Circle_Train_Merged_C" => "Train",
        "BP_DO_Line_Train_Dino_Merged_C" => "Train",
        "BP_DO_Line_Train_Merged_C" => "Train",
        "BP_Dirtbike_C" => "Dirt Bike",
        "BP_DronePackage_Projectile_C" => "Drone",
        "BP_Eragel_CargoShip01_C" => "Ferry Damage",
        "BP_FakeLootProj_AmmoBox_C" => "Loot Truck",
        "BP_FakeLootProj_MilitaryCrate_C" => "Loot Truck",
        "BP_FireEffectController_C" => "Molotov Fire",
        "BP_FireEffectController_JerryCan_C" => "Jerrycan Fire",
        "BP_Food_Truck_C" => "Food Truck",
        "BP_Helicopter_C" => "Pillar Scout Helicopter",
        "BP_IncendiaryDebuff_C" => "Burn",
        "BP_JerryCanFireDebuff_C" => "Burn",
        "BP_JerryCan_FuelPuddle_C" => "Burn",
        "BP_KillTruck_C" => "Kill Truck",
        "BP_LootTruck_C" => "Loot Truck",
        "BP_M_Rony_A_01_C" => "Rony",
        "BP_M_Rony_A_02_C" => "Rony",
        "BP_M_Rony_A_03_C" => "Rony",
        "BP_Mirado_A_02_C" => "Mirado",
        "BP_Mirado_A_03_C" => "Mirado",
        "BP_Mirado_A_03_Esports_C" => "Mirado",
        "BP_Mirado_Open_03_C" => "Mirado (open top)",
        "BP_Mirado_Open_04_C" => "Mirado (open top)",
        "BP_Mirado_Open_05_C" => "Mirado (open top)",
        "BP_MolotovFireDebuff_C" => "Molotov Fire Damage",
        "BP_Motorbike_04_C" => "Motorcycle",
        "BP_Motorbike_04_Desert_C" => "Motorcycle",
        "BP_Motorbike_04_SideCar_C" => "Motorcycle (w/ Sidecar)",
        "BP_Motorbike_04_SideCar_Desert_C" => "Motorcycle (w/ Sidecar)",
        "BP_Motorbike_Solitario_C" => "Motorcycle",
        "BP_Motorglider_C" => "Motor Glider",
        "BP_Motorglider_Green_C" => "Motor Glider",
        "BP_Niva_01_C" => "Zima",
        "BP_Niva_02_C" => "Zima",
        "BP_Niva_03_C" => "Zima",
        "BP_Niva_04_C" => "Zima",
        "BP_Niva_05_C" => "Zima",
        "BP_Niva_06_C" => "Zima",
        "BP_Niva_07_C" => "Zima",
        "BP_Niva_Esports_C" => "Zima",
        "BP_PickupTruck_A_01_C" => "Pickup Truck (closed top)",
        "BP_PickupTruck_A_02_C" => "Pickup Truck (closed top)",
        "BP_PickupTruck_A_03_C" => "Pickup Truck (closed top)",
        "BP_PickupTruck_A_04_C" => "Pickup Truck (closed top)",
        "BP_PickupTruck_A_05_C" => "Pickup Truck (closed top)",
        "BP_PickupTruck_A_esports_C" => "Pickup Truck (closed top)",
        "BP_PickupTruck_B_01_C" => "Pickup Truck (open top)",
        "BP_PickupTruck_B_02_C" => "Pickup Truck (open top)",
        "BP_PickupTruck_B_03_C" => "Pickup Truck (open top)",
        "BP_PickupTruck_B_04_C" => "Pickup Truck (open top)",
        "BP_PickupTruck_B_05_C" => "Pickup Truck (open top)",
        "BP_Pillar_Car_C" => "Pillar Security Car",
        "BP_PonyCoupe_C" => "Pony Coupe",
        "BP_Porter_C" => "Porter",
        "BP_Scooter_01_A_C" => "Scooter",
        "BP_Scooter_02_A_C" => "Scooter",
        "BP_Scooter_03_A_C" => "Scooter",
        "BP_Scooter_04_A_C" => "Scooter",
        "BP_Snowbike_01_C" => "Snowbike",
        "BP_Snowbike_02_C" => "Snowbike",
        "BP_Snowmobile_01_C" => "Snowmobile",
        "BP_Snowmobile_02_C" => "Snowmobile",
        "BP_Snowmobile_03_C" => "Snowmobile",
        "BP_Spiketrap_C" => "Spike Trap",
        "BP_TslGasPump_C" => "Gas Pump",
        "BP_TukTukTuk_A_01_C" => "Tukshai",
        "BP_TukTukTuk_A_02_C" => "Tukshai",
        "BP_TukTukTuk_A_03_C" => "Tukshai",
        "BP_Van_A_01_C" => "Van",
        "BP_Van_A_02_C" => "Van",
        "BP_Van_A_03_C" => "Van",
        "BattleRoyaleModeController_Chimera_C" => "Bluezone",
        "BattleRoyaleModeController_Def_C" => "Bluezone",
        "BattleRoyaleModeController_Desert_C" => "Bluezone",
        "BattleRoyaleModeController_DihorOtok_C" => "Bluezone",
        "BattleRoyaleModeController_Heaven_C" => "Bluezone",
        "BattleRoyaleModeController_Kiki_C" => "Bluezone",
        "BattleRoyaleModeController_Savage_C" => "Bluezone",
        "BattleRoyaleModeController_Summerland_C" => "Bluezone",
        "BattleRoyaleModeController_Tiger_C" => "Bluezone",
        "BlackZoneController_Def_C" => "Blackzone",
        "Bluezonebomb_EffectActor_C" => "Bluezone Grenade",
        "Boat_PG117_C" => "PG-117",
        "Buff_DecreaseBreathInApnea_C" => "Drowning",
        "Buggy_A_01_C" => "Buggy",
        "Buggy_A_02_C" => "Buggy",
        "Buggy_A_03_C" => "Buggy",
        "Buggy_A_04_C" => "Buggy",
        "Buggy_A_05_C" => "Buggy",
        "Buggy_A_06_C" => "Buggy",
        "Carepackage_Container_C" => "Care Package",
        "Dacia_A_01_v2_C" => "Dacia",
        "Dacia_A_01_v2_snow_C" => "Dacia",
        "Dacia_A_02_v2_C" => "Dacia",
        "Dacia_A_03_v2_C" => "Dacia",
        "Dacia_A_03_v2_Esports_C" => "Dacia",
        "Dacia_A_04_v2_C" => "Dacia",
        "DroppedItemGroup" => "Object Fragments",
        "EmergencyAircraft_Tiger_C" => "Emergency Aircraft",
        "Jerrycan" => "Jerrycan",
        "JerrycanFire" => "Jerrycan Fire",
        "Lava" => "Lava",
        "Mortar_Projectile_C" => "Mortar Projectile",
        "None" => "None",
        "PG117_A_01_C" => "PG-117",
        "PanzerFaust100M_Projectile_C" => "Panzerfaust Projectile",
        "PlayerFemale_A_C" => "Player",
        "PlayerMale_A_C" => "Player",
        "ProjC4_C" => "C4",
        "ProjGrenade_C" => "Frag Grenade",
        "ProjIncendiary_C" => "Incendiary Projectile",
        "ProjMolotov_C" => "Molotov Cocktail",
        "ProjMolotov_DamageField_Direct_C" => "Molotov Cocktail Fire Field",
        "ProjStickyGrenade_C" => "Sticky Bomb",
        "RacingDestructiblePropaneTankActor_C" => "Propane Tank",
        "RacingModeContorller_Desert_C" => "Bluezone",
        "RedZoneBomb_C" => "Redzone",
        "RedZoneBombingField" => "Redzone",
        "RedZoneBombingField_Def_C" => "Redzone",
        "SandStormBuff_BP_C" => "Sandstorm",
        "TslDestructibleSurfaceManager" => "Destructible Surface",
        "TslPainCausingVolume" => "Lava",
        "Uaz_A_01_C" => "UAZ (open top)",
        "Uaz_Armored_C" => "UAZ (armored)",
        "Uaz_B_01_C" => "UAZ (soft top)",
        "Uaz_B_01_esports_C" => "UAZ (soft top)",
        "Uaz_C_01_C" => "UAZ (hard top)",
        "UltAIPawn_Base_Female_C" => "Player",
        "UltAIPawn_Base_Male_C" => "Player",
        "WeapACE32_C" => "ACE32",
        "WeapAK47_C" => "AKM",
        "WeapAUG_C" => "AUG A3",
        "WeapAWM_C" => "AWM",
        "WeapBerreta686_C" => "S686",
        "WeapBerylM762_C" => "Beryl",
        "WeapBizonPP19_C" => "Bizon",
        "WeapCowbarProjectile_C" => "Crowbar Projectile",
        "WeapCowbar_C" => "Crowbar",
        "WeapCrossbow_1_C" => "Crossbow",
        "WeapDP12_C" => "DBS",
        "WeapDP28_C" => "DP-28",
        "WeapDesertEagle_C" => "Deagle",
        "WeapDragunov_C" => "Dragunov",
        "WeapDuncansHK416_C" => "M416",
        "WeapFNFal_C" => "SLR",
        "WeapG18_C" => "P18C",
        "WeapG36C_C" => "G36C",
        "WeapGroza_C" => "Groza",
        "WeapHK416_C" => "M416",
        "WeapJS9_C" => "JS9",
        "WeapJuliesKar98k_C" => "Kar98k",
        "WeapK2_C" => "K2",
        "WeapKar98k_C" => "Kar98k",
        "WeapL6_C" => "Lynx AMR",
        "WeapLunchmeatsAK47_C" => "AKM",
        "WeapM16A4_C" => "M16A4",
        "WeapM1911_C" => "P1911",
        "WeapM249_C" => "M249",
        "WeapM24_C" => "M24",
        "WeapM9_C" => "P92",
        "WeapMG3_C" => "MG3",
        "WeapMP5K_C" => "MP5K",
        "WeapMP9_C" => "MP9",
        "WeapMacheteProjectile_C" => "Machete Projectile",
        "WeapMachete_C" => "Machete",
        "WeapMadsQBU88_C" => "QBU88",
        "WeapMini14_C" => "Mini 14",
        "WeapMk12_C" => "Mk12",
        "WeapMk14_C" => "Mk14 EBR",
        "WeapMk47Mutant_C" => "Mk47 Mutant",
        "WeapMosinNagant_C" => "Mosin-Nagant",
        "WeapNagantM1895_C" => "R1895",
        "WeapOriginS12_C" => "O12",
        "WeapP90_C" => "P90",
        "WeapPanProjectile_C" => "Pan Projectile",
        "WeapPan_C" => "Pan",
        "WeapPanzerFaust100M1_C" => "Panzerfaust",
        "WeapQBU88_C" => "QBU88",
        "WeapQBZ95_C" => "QBZ95",
        "WeapRhino_C" => "R45",
        "WeapSCAR-L_C" => "SCAR-L",
        "WeapSKS_C" => "SKS",
        "WeapSaiga12_C" => "S12K",
        "WeapSawnoff_C" => "Sawed-off",
        "WeapSickleProjectile_C" => "Sickle Projectile",
        "WeapSickle_C" => "Sickle",
        "WeapThompson_C" => "Tommy Gun",
        "WeapTurret_KillTruck_Main_C" => "Kill Truck Turret",
        "WeapUMP_C" => "UMP9",
        "WeapUZI_C" => "Micro Uzi",
        "WeapVSS_C" => "VSS",
        "WeapVector_C" => "Vector",
        "WeapWin94_C" => "Win94",
        "WeapWinchester_C" => "S1897",
        "Weapvz61Skorpion_C" => "Skorpion",
        _ => return None,
    };

    Some(display_name.to_string())
}

fn round_damage(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::{aggregate_player_stats, parse_match_detail, parse_telemetry};

    #[test]
    fn parses_and_aggregates_basic_stats() {
        let json = r#"[
          {"_T":"LogMatchStart","_D":"2026-01-01T10:00:00Z","characters":[{"accountId":"a1","name":"self","teamId":10},{"accountId":"a2","name":"mate","teamId":10}]},
          {"_T":"LogPlayerTakeDamage","_D":"2026-01-01T10:05:00Z","attacker":{"accountId":"a1","name":"self"},"victim":{"accountId":"a2","name":"mate"},"damage":99.4,"damageCauserName":"WeapUMP_C"},
          {"_T":"LogPlayerKillV2","_D":"2026-01-01T10:06:00Z","killer":{"accountId":"a1","name":"self"},"victim":{"accountId":"a3","name":"enemy"},"assists_AccountId":["a2"],"damageCauserName":"WeapUMP_C"},
          {"_T":"LogPlayerRevive","_D":"2026-01-01T10:07:00Z","reviver":{"accountId":"a2","name":"mate"},"victim":{"accountId":"a1","name":"self"}},
          {"_T":"LogMatchEnd","_D":"2026-01-01T10:30:00Z","characters":[{"accountId":"a1","ranking":1},{"accountId":"a2","ranking":1}]}
        ]"#;

        let events = parse_telemetry(json).expect("telemetry parses");
        let stats = aggregate_player_stats(&events);
        let detail = parse_match_detail(&events);

        let self_stats = stats
            .iter()
            .find(|entry| entry.pubg_account_id == "a1")
            .expect("self stats present");
        assert_eq!(self_stats.kills, 1);
        assert_eq!(self_stats.damage, 99.4);
        assert_eq!(self_stats.placement, Some(1));
        assert_eq!(detail.kill_events.len(), 1);
        assert_eq!(detail.revive_events.len(), 1);
        assert_eq!(detail.weapon_damage_stats.len(), 1);
        assert_eq!(
            detail.match_start_at.as_deref(),
            Some("2026-01-01T10:00:00Z")
        );
        assert_eq!(detail.match_end_at.as_deref(), Some("2026-01-01T10:30:00Z"));
    }

    #[test]
    fn tolerates_missing_account_ids() {
        let json = r#"[
          {"_T":"LogMatchStart","characters":[{"accountId":"a1","name":"self","teamId":10},{"name":"unknown","teamId":10}]},
          {"_T":"LogPlayerTakeDamage","attacker":{"accountId":"a1","name":"self"},"victim":{"name":"mystery"},"damage":50.0},
          {"_T":"LogPlayerKillV2","killer":{"name":"ghost"}},
          {"_T":"LogPlayerRevive","reviver":{"accountId":"a1","name":"self"}},
          {"_T":"LogMatchEnd","characters":[{"accountId":"a1","ranking":2},{}]}
        ]"#;

        let events = parse_telemetry(json).expect("telemetry parses with missing account ids");
        let stats = aggregate_player_stats(&events);

        assert_eq!(stats.len(), 1);
        let self_stats = stats
            .iter()
            .find(|entry| entry.pubg_account_id == "a1")
            .expect("self stats present");
        assert_eq!(self_stats.damage, 0.0);
        assert_eq!(self_stats.kills, 0);
        assert_eq!(self_stats.revives, 1);
        assert_eq!(self_stats.placement, Some(2));
    }

    #[test]
    fn maps_official_damage_causer_names_for_logs_and_weapon_stats() {
        let json = r#"[
          {"_T":"LogMatchStart","characters":[{"accountId":"a1","name":"self","teamId":10},{"accountId":"a2","name":"enemy","teamId":20}]},
          {"_T":"LogPlayerTakeDamage","_D":"2026-01-01T10:05:00Z","attacker":{"accountId":"a1","name":"self"},"victim":{"accountId":"a2","name":"enemy"},"damage":35.0,"damageCauserName":"WeapMk12_C"},
          {"_T":"LogPlayerKillV2","_D":"2026-01-01T10:06:00Z","killer":{"accountId":"a1","name":"self"},"victim":{"accountId":"a2","name":"enemy"},"finishDamageInfo":{"damageCauserName":"WeapMk12_C"}},
          {"_T":"LogPlayerKillV2","_D":"2026-01-01T10:07:00Z","killer":{"accountId":"a1","name":"self"},"victim":{"accountId":"a2","name":"enemy"},"damageCauserName":"ProjGrenade_C"}
        ]"#;

        let events = parse_telemetry(json).expect("telemetry parses");
        let detail = parse_match_detail(&events);

        assert_eq!(
            detail.damage_events[0].damage_causer_name.as_deref(),
            Some("Mk12")
        );
        assert_eq!(detail.weapon_damage_stats[0].weapon_name, "Mk12");
        assert_eq!(
            detail.kill_events[0].damage_causer_name.as_deref(),
            Some("Mk12")
        );
        assert_eq!(
            detail.kill_events[1].damage_causer_name.as_deref(),
            Some("Frag Grenade")
        );
    }

    #[test]
    fn maps_official_environment_and_vehicle_damage_causers() {
        let json = r#"[
          {"_T":"LogMatchStart","characters":[{"accountId":"a1","name":"self","teamId":10},{"accountId":"a2","name":"enemy","teamId":20}]},
          {"_T":"LogPlayerTakeDamage","_D":"2026-01-01T10:05:00Z","attacker":{"accountId":"a1","name":"self"},"victim":{"accountId":"a2","name":"enemy"},"damage":12.0,"damageCauserName":"BattleRoyaleModeController_Desert_C"},
          {"_T":"LogPlayerKillV2","_D":"2026-01-01T10:06:00Z","killer":{"accountId":"a1","name":"self"},"victim":{"accountId":"a2","name":"enemy"},"damageCauserName":"Uaz_A_01_C"}
        ]"#;

        let events = parse_telemetry(json).expect("telemetry parses");
        let detail = parse_match_detail(&events);

        assert_eq!(
            detail.damage_events[0].damage_causer_name.as_deref(),
            Some("Bluezone")
        );
        assert_eq!(detail.weapon_damage_stats[0].weapon_name, "Bluezone");
        assert_eq!(
            detail.kill_events[0].damage_causer_name.as_deref(),
            Some("UAZ (open top)")
        );
    }

    #[test]
    fn captures_knock_events_separately_from_final_kills() {
        let json = r#"[
          {"_T":"LogMatchStart","characters":[{"accountId":"a1","name":"self","teamId":10},{"accountId":"a2","name":"enemy","teamId":20}]},
          {"_T":"LogPlayerMakeGroggy","_D":"2026-01-01T10:05:00Z","attacker":{"accountId":"a1","name":"self"},"victim":{"accountId":"a2","name":"enemy"},"damageTypeCategory":"Damage_Gun","damageCauserName":"WeapACE32_C","dBNOId":77},
          {"_T":"LogPlayerKillV2","_D":"2026-01-01T10:06:00Z","killer":{"accountId":"a1","name":"self"},"victim":{"accountId":"a2","name":"enemy"},"dBNOMaker":{"accountId":"a1","name":"self"},"dBNODamageInfo":{"damageCauserName":"WeapACE32_C"},"finisher":{"accountId":"a1","name":"self"},"finishDamageInfo":{"damageCauserName":"WeapACE32_C"},"dBNOId":77}
        ]"#;

        let events = parse_telemetry(json).expect("telemetry parses");
        let detail = parse_match_detail(&events);

        assert_eq!(detail.knock_events.len(), 1);
        assert_eq!(detail.kill_events.len(), 1);
        assert_eq!(
            detail.knock_events[0].attacker_name.as_deref(),
            Some("self")
        );
        assert_eq!(detail.knock_events[0].victim_name.as_deref(), Some("enemy"));
        assert_eq!(
            detail.knock_events[0].damage_causer_name.as_deref(),
            Some("ACE32")
        );
        assert_eq!(detail.kill_events[0].killer_name.as_deref(), Some("self"));
        assert_eq!(detail.kill_events[0].victim_name.as_deref(), Some("enemy"));
        assert_eq!(
            detail.kill_events[0].damage_causer_name.as_deref(),
            Some("ACE32")
        );
    }
}
