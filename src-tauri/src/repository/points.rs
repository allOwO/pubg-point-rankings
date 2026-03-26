use std::collections::HashMap;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::{engine::calculator::apply_rounding, error::AppError};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointRecordDto {
    pub id: i64,
    pub account_id: i64,
    pub match_id: String,
    pub match_player_id: i64,
    pub teammate_id: Option<i64>,
    pub rule_id: i64,
    pub rule_name_snapshot: String,
    pub damage_points_per_damage_snapshot: i64,
    pub kill_points_snapshot: i64,
    pub revive_points_snapshot: i64,
    pub rounding_mode_snapshot: String,
    pub points: i64,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CreatePointRecordInput {
    pub match_id: String,
    pub match_player_id: i64,
    pub teammate_id: Option<i64>,
    pub rule_id: i64,
    pub rule_name_snapshot: String,
    pub damage_points_per_damage_snapshot: i64,
    pub kill_points_snapshot: i64,
    pub revive_points_snapshot: i64,
    pub rounding_mode_snapshot: String,
    pub points: i64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointHistoryPlayerBreakdownDto {
    pub match_player_id: i64,
    pub teammate_id: Option<i64>,
    pub pubg_account_id: Option<String>,
    pub pubg_player_name: String,
    pub display_nickname_snapshot: Option<String>,
    pub is_self: bool,
    pub is_points_enabled_snapshot: bool,
    pub damage: f64,
    pub kills: i64,
    pub revives: i64,
    pub damage_points_per_damage_snapshot: i64,
    pub kill_points_snapshot: i64,
    pub revive_points_snapshot: i64,
    pub damage_points: i64,
    pub kill_points: i64,
    pub revive_points: i64,
    pub total_points: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointBattleDeltaDto {
    pub match_player_id: i64,
    pub teammate_id: Option<i64>,
    pub pubg_player_name: String,
    pub display_nickname_snapshot: Option<String>,
    pub delta: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointHistoryMatchGroupDto {
    #[serde(rename = "type")]
    pub item_type: String,
    pub match_id: String,
    pub played_at: String,
    pub map_name: Option<String>,
    pub game_mode: Option<String>,
    pub rule_id: i64,
    pub rule_name_snapshot: String,
    pub is_settled: bool,
    pub settled_at: Option<String>,
    pub settlement_batch_id: Option<i64>,
    pub note: Option<String>,
    pub players: Vec<PointHistoryPlayerBreakdownDto>,
    pub battle_deltas: Vec<PointBattleDeltaDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointHistoryRuleChangeMarkerDto {
    #[serde(rename = "type")]
    pub item_type: String,
    pub previous_rule_name: String,
    pub next_rule_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum PointHistoryListItemDto {
    MatchGroup(PointHistoryMatchGroupDto),
    RuleChangeMarker(PointHistoryRuleChangeMarkerDto),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsettledPlayerSummaryDto {
    pub teammate_id: Option<i64>,
    pub pubg_player_name: String,
    pub display_nickname: Option<String>,
    pub is_self: bool,
    pub total_delta: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsettledBattleSummaryDto {
    pub active_rule_name: Option<String>,
    pub unsettled_match_count: i64,
    pub players: Vec<UnsettledPlayerSummaryDto>,
}

pub struct PointRecordsRepository<'a> {
    connection: &'a Connection,
    account_id: i64,
}

impl<'a> PointRecordsRepository<'a> {
    pub fn new(connection: &'a Connection, account_id: i64) -> Self {
        Self {
            connection,
            account_id,
        }
    }

    pub fn get_all(&self, limit: i64, offset: i64) -> Result<Vec<PointRecordDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
              damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot,
              points, note, created_at
             FROM point_records WHERE account_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
        )?;

        let rows = statement.query_map(params![self.account_id, limit, offset], |row| {
            Self::map_row(row)
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_by_match(&self, match_id: &str) -> Result<Vec<PointRecordDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
              damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot,
              points, note, created_at
             FROM point_records WHERE account_id = ?1 AND match_id = ?2 ORDER BY points DESC",
        )?;

        let rows =
            statement.query_map(params![self.account_id, match_id], |row| Self::map_row(row))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_by_teammate(&self, teammate_id: i64) -> Result<Vec<PointRecordDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
              damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot,
              points, note, created_at
             FROM point_records WHERE account_id = ?1 AND teammate_id = ?2 ORDER BY created_at DESC",
        )?;

        let rows = statement.query_map(params![self.account_id, teammate_id], |row| {
            Self::map_row(row)
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn exists_for_match(&self, match_id: &str) -> Result<bool, AppError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM point_records WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    pub fn create(&self, input: CreatePointRecordInput) -> Result<PointRecordDto, AppError> {
        self.connection.execute(
            "INSERT INTO point_records
             (account_id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot, damage_points_per_damage_snapshot,
              kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot, points, note, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                input.match_id,
                input.match_player_id,
                input.teammate_id,
                input.rule_id,
                input.rule_name_snapshot,
                input.damage_points_per_damage_snapshot,
                input.kill_points_snapshot,
                input.revive_points_snapshot,
                input.rounding_mode_snapshot,
                input.points,
                input.note,
            ],
        )?;

        let id = self.connection.last_insert_rowid();
        let result = self.connection.query_row(
            "SELECT id, account_id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
              damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot, rounding_mode_snapshot,
              points, note, created_at
             FROM point_records WHERE account_id = ?1 AND id = ?2",
            params![self.account_id, id],
            Self::map_row,
        );

        match result {
            Ok(record) => Ok(record),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_total_for_teammate(&self, teammate_id: i64) -> Result<i64, AppError> {
        let total: i64 = self.connection.query_row(
            "SELECT COALESCE(SUM(points), 0) FROM point_records WHERE account_id = ?1 AND teammate_id = ?2",
            params![self.account_id, teammate_id],
            |row| row.get(0),
        )?;

        Ok(total)
    }

    pub fn delete_for_match(&self, match_id: &str) -> Result<(), AppError> {
        self.connection.execute(
            "DELETE FROM point_records WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
        )?;
        Ok(())
    }

    pub fn get_history_groups(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PointHistoryListItemDto>, AppError> {
        let groups = self.get_history_match_groups(Some(limit), offset, false)?;
        Ok(groups
            .into_iter()
            .map(PointHistoryListItemDto::MatchGroup)
            .collect())
    }

    pub fn get_unsettled_summary(&self) -> Result<UnsettledBattleSummaryDto, AppError> {
        let active_rule_name = self
            .connection
            .query_row(
                "SELECT name FROM point_rules WHERE account_id = ?1 AND is_active = 1 AND is_deleted = 0 LIMIT 1",
                [self.account_id],
                |row| row.get::<_, String>(0),
            )
            .ok();

        let unsettled_groups = self.get_history_match_groups(None, 0, true)?;
        if unsettled_groups.is_empty() {
            return Ok(UnsettledBattleSummaryDto {
                active_rule_name,
                unsettled_match_count: 0,
                players: Vec::new(),
            });
        }

        #[derive(Debug, Clone)]
        struct Aggregate {
            teammate_id: Option<i64>,
            pubg_player_name: String,
            fallback_display_nickname: Option<String>,
            is_self: bool,
            total_delta: i64,
        }

        let mut aggregates: HashMap<String, Aggregate> = HashMap::new();

        for group in &unsettled_groups {
            let players_by_id: HashMap<i64, &PointHistoryPlayerBreakdownDto> = group
                .players
                .iter()
                .map(|player| (player.match_player_id, player))
                .collect();

            for delta in &group.battle_deltas {
                let player = match players_by_id.get(&delta.match_player_id) {
                    Some(value) => *value,
                    None => continue,
                };

                let key = format!(
                    "{}::{}::{}",
                    player
                        .teammate_id
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "null".to_string()),
                    player.pubg_player_name,
                    if player.is_self { 1 } else { 0 },
                );

                let entry = aggregates.entry(key).or_insert_with(|| Aggregate {
                    teammate_id: player.teammate_id,
                    pubg_player_name: player.pubg_player_name.clone(),
                    fallback_display_nickname: player.display_nickname_snapshot.clone(),
                    is_self: player.is_self,
                    total_delta: 0,
                });

                entry.total_delta += delta.delta;
            }
        }

        let mut players = aggregates
            .into_values()
            .map(|aggregate| {
                let display_nickname = match aggregate.teammate_id {
                    Some(teammate_id) => self
                        .connection
                        .query_row(
                            "SELECT display_nickname
                             FROM teammates
                             WHERE account_id = ?1 AND id = ?2",
                            params![self.account_id, teammate_id],
                            |row| row.get::<_, Option<String>>(0),
                        )
                        .unwrap_or(aggregate.fallback_display_nickname),
                    None => aggregate.fallback_display_nickname,
                };

                UnsettledPlayerSummaryDto {
                    teammate_id: aggregate.teammate_id,
                    pubg_player_name: aggregate.pubg_player_name,
                    display_nickname,
                    is_self: aggregate.is_self,
                    total_delta: aggregate.total_delta,
                }
            })
            .collect::<Vec<_>>();

        players.sort_by(|left, right| {
            right
                .total_delta
                .cmp(&left.total_delta)
                .then_with(|| left.pubg_player_name.cmp(&right.pubg_player_name))
                .then_with(|| left.teammate_id.cmp(&right.teammate_id))
        });

        Ok(UnsettledBattleSummaryDto {
            active_rule_name,
            unsettled_match_count: unsettled_groups.len() as i64,
            players,
        })
    }

    fn get_history_match_groups(
        &self,
        limit: Option<i64>,
        offset: i64,
        unsettled_only: bool,
    ) -> Result<Vec<PointHistoryMatchGroupDto>, AppError> {
        let mut match_ids_statement = if unsettled_only {
            self.connection.prepare(
                "SELECT m.match_id
                 FROM matches m
                 WHERE m.account_id = ?1
                   AND EXISTS (
                     SELECT 1
                     FROM point_records pr
                     WHERE pr.account_id = m.account_id AND pr.match_id = m.match_id
                   )
                   AND NOT EXISTS (
                     SELECT 1
                     FROM point_match_meta pmm
                     WHERE pmm.account_id = m.account_id
                       AND pmm.match_id = m.match_id
                       AND pmm.settled_at IS NOT NULL
                   )
                 ORDER BY m.played_at DESC, m.match_id DESC",
            )?
        } else {
            self.connection.prepare(
                "SELECT m.match_id
                 FROM matches m
                 WHERE m.account_id = ?1
                   AND EXISTS (
                     SELECT 1
                     FROM point_records pr
                     WHERE pr.account_id = m.account_id AND pr.match_id = m.match_id
                   )
                 ORDER BY m.played_at DESC, m.match_id DESC",
            )?
        };

        let all_match_ids_rows =
            match_ids_statement.query_map([self.account_id], |row| row.get::<_, String>(0))?;
        let mut all_match_ids = all_match_ids_rows.collect::<Result<Vec<_>, _>>()?;

        if let Some(limit_value) = limit {
            let start = offset.max(0) as usize;
            let end = (start + limit_value.max(0) as usize).min(all_match_ids.len());
            all_match_ids = if start >= all_match_ids.len() {
                Vec::new()
            } else {
                all_match_ids[start..end].to_vec()
            };
        }

        if all_match_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = std::iter::repeat_n("?", all_match_ids.len())
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "SELECT
               m.match_id,
               m.played_at,
               m.map_name,
               m.game_mode,
               pr.rule_id,
               pr.rule_name_snapshot,
               pmm.note,
               pmm.settled_at,
               pmm.settlement_batch_id,
               mp.id,
               mp.teammate_id,
               mp.pubg_account_id,
               mp.pubg_player_name,
               mp.display_nickname_snapshot,
               mp.is_self,
               mp.is_points_enabled_snapshot,
               mp.damage,
               mp.kills,
               mp.revives,
               pr.damage_points_per_damage_snapshot,
               pr.kill_points_snapshot,
               pr.revive_points_snapshot,
               pr.rounding_mode_snapshot,
               pr.points
             FROM matches m
             INNER JOIN point_records pr
               ON pr.account_id = m.account_id AND pr.match_id = m.match_id
             INNER JOIN match_players mp
               ON mp.account_id = pr.account_id AND mp.id = pr.match_player_id
             LEFT JOIN point_match_meta pmm
               ON pmm.account_id = m.account_id AND pmm.match_id = m.match_id
             WHERE m.account_id = ?
               AND m.match_id IN ({})
             ORDER BY m.played_at DESC, m.match_id DESC, mp.id ASC",
            placeholders,
        );

        let mut query_values: Vec<rusqlite::types::Value> =
            Vec::with_capacity(1 + all_match_ids.len());
        query_values.push(self.account_id.into());
        for match_id in &all_match_ids {
            query_values.push(match_id.clone().into());
        }

        let mut statement = self.connection.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(query_values), |row| {
            Ok(HistoryGroupRow {
                match_id: row.get(0)?,
                played_at: row.get(1)?,
                map_name: row.get(2)?,
                game_mode: row.get(3)?,
                rule_id: row.get(4)?,
                rule_name_snapshot: row.get(5)?,
                note: row.get(6)?,
                settled_at: row.get(7)?,
                settlement_batch_id: row.get(8)?,
                match_player_id: row.get(9)?,
                teammate_id: row.get(10)?,
                pubg_account_id: row.get(11)?,
                pubg_player_name: row.get(12)?,
                display_nickname_snapshot: row.get(13)?,
                is_self: row.get::<_, i64>(14)? == 1,
                is_points_enabled_snapshot: row.get::<_, i64>(15)? == 1,
                damage: row.get(16)?,
                kills: row.get(17)?,
                revives: row.get(18)?,
                damage_points_per_damage_snapshot: row.get(19)?,
                kill_points_snapshot: row.get(20)?,
                revive_points_snapshot: row.get(21)?,
                rounding_mode_snapshot: row.get(22)?,
                total_points: row.get(23)?,
            })
        })?;

        let mut grouped_items: Vec<PointHistoryMatchGroupDto> = Vec::new();
        let mut group_indices: HashMap<String, usize> = HashMap::new();

        for row in rows {
            let row = row?;
            let group_index = if let Some(index) = group_indices.get(&row.match_id) {
                *index
            } else {
                let index = grouped_items.len();
                grouped_items.push(PointHistoryMatchGroupDto {
                    item_type: "match_group".to_string(),
                    match_id: row.match_id.clone(),
                    played_at: row.played_at.clone(),
                    map_name: row.map_name.clone(),
                    game_mode: row.game_mode.clone(),
                    rule_id: row.rule_id,
                    rule_name_snapshot: row.rule_name_snapshot.clone(),
                    is_settled: row.settled_at.is_some(),
                    settled_at: row.settled_at.clone(),
                    settlement_batch_id: row.settlement_batch_id,
                    note: row.note.clone(),
                    players: Vec::new(),
                    battle_deltas: Vec::new(),
                });
                group_indices.insert(row.match_id.clone(), index);
                index
            };

            let damage_points = apply_rounding(
                row.damage * (row.damage_points_per_damage_snapshot as f64),
                &row.rounding_mode_snapshot,
            );

            grouped_items[group_index]
                .players
                .push(PointHistoryPlayerBreakdownDto {
                    match_player_id: row.match_player_id,
                    teammate_id: row.teammate_id,
                    pubg_account_id: row.pubg_account_id,
                    pubg_player_name: row.pubg_player_name,
                    display_nickname_snapshot: row.display_nickname_snapshot,
                    is_self: row.is_self,
                    is_points_enabled_snapshot: row.is_points_enabled_snapshot,
                    damage: row.damage,
                    kills: row.kills,
                    revives: row.revives,
                    damage_points_per_damage_snapshot: row.damage_points_per_damage_snapshot,
                    kill_points_snapshot: row.kill_points_snapshot,
                    revive_points_snapshot: row.revive_points_snapshot,
                    damage_points,
                    kill_points: row.kills.saturating_mul(row.kill_points_snapshot),
                    revive_points: row.revives.saturating_mul(row.revive_points_snapshot),
                    total_points: row.total_points,
                });
        }

        for group in &mut grouped_items {
            group.battle_deltas = calculate_battle_deltas_for_players(&group.players);
        }

        Ok(grouped_items)
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PointRecordDto> {
        Ok(PointRecordDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            match_id: row.get(2)?,
            match_player_id: row.get(3)?,
            teammate_id: row.get(4)?,
            rule_id: row.get(5)?,
            rule_name_snapshot: row.get(6)?,
            damage_points_per_damage_snapshot: row.get(7)?,
            kill_points_snapshot: row.get(8)?,
            revive_points_snapshot: row.get(9)?,
            rounding_mode_snapshot: row.get(10)?,
            points: row.get(11)?,
            note: row.get(12)?,
            created_at: row.get(13)?,
        })
    }
}

#[derive(Debug, Clone)]
struct HistoryGroupRow {
    match_id: String,
    played_at: String,
    map_name: Option<String>,
    game_mode: Option<String>,
    rule_id: i64,
    rule_name_snapshot: String,
    note: Option<String>,
    settled_at: Option<String>,
    settlement_batch_id: Option<i64>,
    match_player_id: i64,
    teammate_id: Option<i64>,
    pubg_account_id: Option<String>,
    pubg_player_name: String,
    display_nickname_snapshot: Option<String>,
    is_self: bool,
    is_points_enabled_snapshot: bool,
    damage: f64,
    kills: i64,
    revives: i64,
    damage_points_per_damage_snapshot: i64,
    kill_points_snapshot: i64,
    revive_points_snapshot: i64,
    rounding_mode_snapshot: String,
    total_points: i64,
}

fn calculate_battle_deltas_for_players(
    players: &[PointHistoryPlayerBreakdownDto],
) -> Vec<PointBattleDeltaDto> {
    let participants: Vec<&PointHistoryPlayerBreakdownDto> = players
        .iter()
        .filter(|player| player.is_points_enabled_snapshot)
        .collect();

    if participants.len() < 2 {
        return players
            .iter()
            .map(|player| PointBattleDeltaDto {
                match_player_id: player.match_player_id,
                teammate_id: player.teammate_id,
                pubg_player_name: player.pubg_player_name.clone(),
                display_nickname_snapshot: player.display_nickname_snapshot.clone(),
                delta: 0,
            })
            .collect();
    }

    let mut highest = participants[0];
    let mut lowest = participants[0];

    for participant in participants.iter().skip(1) {
        if participant.total_points > highest.total_points
            || (participant.total_points == highest.total_points
                && participant.match_player_id < highest.match_player_id)
        {
            highest = participant;
        }

        if participant.total_points < lowest.total_points
            || (participant.total_points == lowest.total_points
                && participant.match_player_id < lowest.match_player_id)
        {
            lowest = participant;
        }
    }

    let gap = highest.total_points - lowest.total_points;
    if gap == 0 {
        return players
            .iter()
            .map(|player| PointBattleDeltaDto {
                match_player_id: player.match_player_id,
                teammate_id: player.teammate_id,
                pubg_player_name: player.pubg_player_name.clone(),
                display_nickname_snapshot: player.display_nickname_snapshot.clone(),
                delta: 0,
            })
            .collect();
    }

    players
        .iter()
        .map(|player| {
            let delta = if player.match_player_id == highest.match_player_id {
                gap
            } else if player.match_player_id == lowest.match_player_id {
                -gap
            } else {
                0
            };

            PointBattleDeltaDto {
                match_player_id: player.match_player_id,
                teammate_id: player.teammate_id,
                pubg_player_name: player.pubg_player_name.clone(),
                display_nickname_snapshot: player.display_nickname_snapshot.clone(),
                delta,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{calculate_battle_deltas_for_players, PointHistoryPlayerBreakdownDto};

    fn player(
        match_player_id: i64,
        total_points: i64,
        enabled: bool,
    ) -> PointHistoryPlayerBreakdownDto {
        PointHistoryPlayerBreakdownDto {
            match_player_id,
            teammate_id: Some(match_player_id),
            pubg_account_id: None,
            pubg_player_name: format!("player-{match_player_id}"),
            display_nickname_snapshot: None,
            is_self: match_player_id == 1,
            is_points_enabled_snapshot: enabled,
            damage: 100.0,
            kills: 1,
            revives: 0,
            damage_points_per_damage_snapshot: 2,
            kill_points_snapshot: 300,
            revive_points_snapshot: 0,
            damage_points: 200,
            kill_points: 300,
            revive_points: 0,
            total_points,
        }
    }

    #[test]
    fn battle_delta_assigns_gap_to_highest_and_lowest_with_stable_tie_break() {
        let players = vec![
            player(1, 1000, true),
            player(2, 900, true),
            player(3, 800, true),
            player(4, 700, true),
        ];

        let deltas = calculate_battle_deltas_for_players(&players);

        assert_eq!(deltas.len(), 4);
        assert_eq!(deltas[0].delta, 300);
        assert_eq!(deltas[1].delta, 0);
        assert_eq!(deltas[2].delta, 0);
        assert_eq!(deltas[3].delta, -300);
    }

    #[test]
    fn battle_delta_is_zero_when_less_than_two_enabled_players() {
        let players = vec![
            player(1, 1000, true),
            player(2, 500, false),
            player(3, 300, false),
        ];

        let deltas = calculate_battle_deltas_for_players(&players);
        assert!(deltas.iter().all(|value| value.delta == 0));
    }

    #[test]
    fn battle_delta_is_zero_when_highest_equals_lowest() {
        let players = vec![
            player(1, 1000, true),
            player(2, 1000, true),
            player(3, 1000, true),
        ];

        let deltas = calculate_battle_deltas_for_players(&players);
        assert!(deltas.iter().all(|value| value.delta == 0));
    }
}
