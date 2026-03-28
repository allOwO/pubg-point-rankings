use serde::Serialize;

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub pubg_account_id: String,
    pub pubg_player_name: String,
    pub damage: f64,
    pub kills: i64,
    pub assists: i64,
    pub revives: i64,
    pub team_id: Option<i64>,
    pub placement: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculatedPoints {
    pub pubg_account_id: String,
    pub pubg_player_name: String,
    pub damage: f64,
    pub kills: i64,
    pub assists: i64,
    pub revives: i64,
    pub damage_points: i64,
    pub kill_points: i64,
    pub revive_points: i64,
    pub total_points: i64,
    pub is_points_enabled: bool,
    pub team_id: Option<i64>,
    pub placement: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct RuleConfig {
    pub id: i64,
    pub name: String,
    pub damage_points_per_damage: i64,
    pub kill_points: i64,
    pub revive_points: i64,
    pub rounding_mode: String,
}

pub fn apply_rounding(value: f64, mode: &str) -> i64 {
    match mode {
        "floor" => value.floor() as i64,
        "ceil" => value.ceil() as i64,
        _ => value.round() as i64,
    }
}

pub fn calculate_points(
    players: &[PlayerStats],
    rule: &RuleConfig,
    enabled_player_keys: &std::collections::HashSet<String>,
) -> Vec<CalculatedPoints> {
    players
        .iter()
        .map(|player| {
            let is_enabled = enabled_player_keys.contains(&player_identity_key(
                &player.pubg_account_id,
                &player.pubg_player_name,
            ));
            let damage_points_value = player.damage * (rule.damage_points_per_damage as f64);
            let damage_points = apply_rounding(damage_points_value, &rule.rounding_mode);
            let kill_points = player.kills.saturating_mul(rule.kill_points);
            let revive_points = player.revives.saturating_mul(rule.revive_points);
            let total_points = if is_enabled {
                apply_rounding(
                    damage_points_value + (kill_points as f64) + (revive_points as f64),
                    &rule.rounding_mode,
                )
            } else {
                0
            };

            CalculatedPoints {
                pubg_account_id: player.pubg_account_id.clone(),
                pubg_player_name: player.pubg_player_name.clone(),
                damage: player.damage,
                kills: player.kills,
                assists: player.assists,
                revives: player.revives,
                damage_points,
                kill_points,
                revive_points,
                total_points,
                is_points_enabled: is_enabled,
                team_id: player.team_id,
                placement: player.placement,
            }
        })
        .collect()
}

fn player_identity_key(pubg_account_id: &str, pubg_player_name: &str) -> String {
    let trimmed_account_id = pubg_account_id.trim();
    if !trimmed_account_id.is_empty() {
        return format!("account:{trimmed_account_id}");
    }

    format!("name:{}", pubg_player_name.trim().to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{apply_rounding, calculate_points, PlayerStats, RuleConfig};

    #[test]
    fn applies_rounding_modes() {
        assert_eq!(apply_rounding(1.2, "floor"), 1);
        assert_eq!(apply_rounding(1.2, "round"), 1);
        assert_eq!(apply_rounding(1.8, "round"), 2);
        assert_eq!(apply_rounding(1.2, "ceil"), 2);
    }

    #[test]
    fn disables_points_for_non_enabled_players() {
        let players = vec![PlayerStats {
            pubg_account_id: "account.one".to_string(),
            pubg_player_name: "one".to_string(),
            damage: 100.0,
            kills: 2,
            assists: 1,
            revives: 1,
            team_id: Some(1),
            placement: Some(1),
        }];

        let rule = RuleConfig {
            id: 1,
            name: "default".to_string(),
            damage_points_per_damage: 2,
            kill_points: 300,
            revive_points: 150,
            rounding_mode: "round".to_string(),
        };

        let enabled = HashSet::new();
        let calculated = calculate_points(&players, &rule, &enabled);
        assert_eq!(calculated.len(), 1);
        assert_eq!(calculated[0].total_points, 0);
    }

    #[test]
    fn enables_points_when_account_identity_key_matches() {
        let players = vec![PlayerStats {
            pubg_account_id: "account.one".to_string(),
            pubg_player_name: "one".to_string(),
            damage: 100.0,
            kills: 2,
            assists: 1,
            revives: 1,
            team_id: Some(1),
            placement: Some(1),
        }];

        let rule = RuleConfig {
            id: 1,
            name: "default".to_string(),
            damage_points_per_damage: 2,
            kill_points: 300,
            revive_points: 150,
            rounding_mode: "round".to_string(),
        };

        let enabled = HashSet::from(["account:account.one".to_string()]);
        let calculated = calculate_points(&players, &rule, &enabled);

        assert_eq!(calculated.len(), 1);
        assert!(calculated[0].is_points_enabled);
        assert_eq!(calculated[0].total_points, 950);
    }

    #[test]
    fn enables_points_when_name_identity_key_matches_without_account_id() {
        let players = vec![PlayerStats {
            pubg_account_id: String::new(),
            pubg_player_name: "One".to_string(),
            damage: 100.0,
            kills: 2,
            assists: 1,
            revives: 1,
            team_id: Some(1),
            placement: Some(1),
        }];

        let rule = RuleConfig {
            id: 1,
            name: "default".to_string(),
            damage_points_per_damage: 2,
            kill_points: 300,
            revive_points: 150,
            rounding_mode: "round".to_string(),
        };

        let enabled = HashSet::from(["name:one".to_string()]);
        let calculated = calculate_points(&players, &rule, &enabled);

        assert_eq!(calculated.len(), 1);
        assert!(calculated[0].is_points_enabled);
        assert_eq!(calculated[0].total_points, 950);
    }
}
