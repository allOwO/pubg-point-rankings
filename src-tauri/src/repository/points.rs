use std::collections::{HashMap, HashSet};

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::{
    engine::calculator::apply_rounding,
    error::AppError,
    repository::{rules::PointRulesRepository, teammates::TeammatesRepository},
};

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
    pub rule_id: Option<i64>,
    pub active_rule_name: Option<String>,
    pub unsettled_match_count: i64,
    pub players: Vec<UnsettledPlayerSummaryDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecalculateUnsettledPointsResultDto {
    pub rule_id: i64,
    pub rule_name: String,
    pub recalculated_match_count: i64,
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

    pub fn repair_points_with_current_identities(
        &self,
        self_player_name: &str,
    ) -> Result<i64, AppError> {
        #[derive(Debug)]
        struct RepairRow {
            match_id: String,
            match_player_id: i64,
            teammate_id: Option<i64>,
            pubg_player_name: String,
            team_id: Option<i64>,
            damage: f64,
            kills: i64,
            revives: i64,
            damage_points_per_damage_snapshot: i64,
            kill_points_snapshot: i64,
            revive_points_snapshot: i64,
            rounding_mode_snapshot: String,
            match_player_points: i64,
            point_record_points: i64,
            is_self: bool,
            is_points_enabled_snapshot: bool,
        }

        let teammate_enabled_by_id = {
            let mut statement = self.connection.prepare(
                "SELECT id, is_points_enabled
                 FROM teammates
                 WHERE account_id = ?1",
            )?;

            let rows = statement.query_map([self.account_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)? == 1))
            })?;

            rows.collect::<Result<HashMap<_, _>, _>>()?
        };

        let repair_rows = {
            let mut statement = self.connection.prepare(
                "SELECT pr.match_id,
                        mp.id,
                        mp.teammate_id,
                        mp.pubg_player_name,
                        mp.team_id,
                        mp.damage,
                        mp.kills,
                        mp.revives,
                        pr.damage_points_per_damage_snapshot,
                        pr.kill_points_snapshot,
                        pr.revive_points_snapshot,
                        pr.rounding_mode_snapshot,
                        mp.points,
                        pr.points,
                        mp.is_self,
                        mp.is_points_enabled_snapshot
                 FROM point_records pr
                 INNER JOIN match_players mp
                   ON mp.account_id = pr.account_id
                  AND mp.id = pr.match_player_id
                 WHERE pr.account_id = ?1
                 ORDER BY pr.match_id ASC, mp.id ASC",
            )?;

            let rows = statement.query_map([self.account_id], |row| {
                Ok(RepairRow {
                    match_id: row.get(0)?,
                    match_player_id: row.get(1)?,
                    teammate_id: row.get(2)?,
                    pubg_player_name: row.get(3)?,
                    team_id: row.get(4)?,
                    damage: row.get(5)?,
                    kills: row.get(6)?,
                    revives: row.get(7)?,
                    damage_points_per_damage_snapshot: row.get(8)?,
                    kill_points_snapshot: row.get(9)?,
                    revive_points_snapshot: row.get(10)?,
                    rounding_mode_snapshot: row.get(11)?,
                    match_player_points: row.get(12)?,
                    point_record_points: row.get(13)?,
                    is_self: row.get::<_, i64>(14)? == 1,
                    is_points_enabled_snapshot: row.get::<_, i64>(15)? == 1,
                })
            })?;

            rows.collect::<Result<Vec<_>, _>>()?
        };

        if repair_rows.is_empty() {
            return Ok(0);
        }

        let self_team_id_by_match = repair_rows
            .iter()
            .filter(|row| row.pubg_player_name.eq_ignore_ascii_case(self_player_name))
            .map(|row| (row.match_id.clone(), row.team_id))
            .collect::<HashMap<_, _>>();

        let mut repaired_match_ids: HashSet<String> = HashSet::new();
        let tx = self.connection.unchecked_transaction()?;

        for row in repair_rows {
            let is_self = row.pubg_player_name.eq_ignore_ascii_case(self_player_name);
            let is_same_team = self_team_id_by_match
                .get(&row.match_id)
                .copied()
                .flatten()
                .is_some_and(|self_team_id| row.team_id == Some(self_team_id));
            let is_points_enabled_snapshot = is_self
                || (is_same_team
                    && row
                        .teammate_id
                        .and_then(|teammate_id| teammate_enabled_by_id.get(&teammate_id).copied())
                        .unwrap_or(true));

            let kill_points = row.kills.saturating_mul(row.kill_points_snapshot);
            let revive_points = row.revives.saturating_mul(row.revive_points_snapshot);
            let total_points = if is_points_enabled_snapshot {
                apply_rounding(
                    row.damage * (row.damage_points_per_damage_snapshot as f64)
                        + (kill_points as f64)
                        + (revive_points as f64),
                    &row.rounding_mode_snapshot,
                )
            } else {
                0
            };

            if total_points != row.match_player_points
                || total_points != row.point_record_points
                || is_self != row.is_self
                || is_points_enabled_snapshot != row.is_points_enabled_snapshot
            {
                tx.execute(
                    "UPDATE match_players
                     SET is_self = ?1,
                         is_points_enabled_snapshot = ?2,
                         points = ?3
                     WHERE account_id = ?4 AND id = ?5",
                    params![
                        if is_self { 1 } else { 0 },
                        if is_points_enabled_snapshot { 1 } else { 0 },
                        total_points,
                        self.account_id,
                        row.match_player_id,
                    ],
                )?;

                tx.execute(
                    "UPDATE point_records
                     SET points = ?1
                     WHERE account_id = ?2 AND match_player_id = ?3",
                    params![total_points, self.account_id, row.match_player_id],
                )?;
            }

            repaired_match_ids.insert(row.match_id);
        }

        tx.execute(
            "UPDATE teammates
             SET total_points = COALESCE((
                   SELECT SUM(pr.points)
                   FROM point_records pr
                   WHERE pr.account_id = teammates.account_id AND pr.teammate_id = teammates.id
                 ), 0),
                 updated_at = CURRENT_TIMESTAMP
             WHERE account_id = ?1",
            [self.account_id],
        )?;

        tx.commit()?;

        Ok(repaired_match_ids.len() as i64)
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
        let active_rule = self
            .connection
            .query_row(
                "SELECT id, name FROM point_rules WHERE account_id = ?1 AND is_active = 1 AND is_deleted = 0 LIMIT 1",
                [self.account_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .ok();

        let unsettled_groups = self.get_history_match_groups(None, 0, true)?;
        if unsettled_groups.is_empty() {
            return Ok(UnsettledBattleSummaryDto {
                rule_id: active_rule.as_ref().map(|(rule_id, _)| *rule_id),
                active_rule_name: active_rule.map(|(_, rule_name)| rule_name),
                unsettled_match_count: 0,
                players: Vec::new(),
            });
        }

        let summary_rule_id = unsettled_groups.first().map(|group| group.rule_id);
        let summary_rule_name = unsettled_groups
            .first()
            .map(|group| group.rule_name_snapshot.clone())
            .or_else(|| active_rule.as_ref().map(|(_, rule_name)| rule_name.clone()));

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
                    "{}::{}",
                    player
                        .pubg_account_id
                        .clone()
                        .unwrap_or_else(|| player.pubg_player_name.clone()),
                    if player.is_self { 1 } else { 0 },
                );

                let entry = aggregates.entry(key).or_insert_with(|| Aggregate {
                    teammate_id: player.teammate_id,
                    pubg_player_name: player.pubg_player_name.clone(),
                    fallback_display_nickname: player.display_nickname_snapshot.clone(),
                    is_self: player.is_self,
                    total_delta: 0,
                });

                if entry.teammate_id.is_none() {
                    entry.teammate_id = player.teammate_id;
                }

                if entry.fallback_display_nickname.is_none() {
                    entry.fallback_display_nickname = player.display_nickname_snapshot.clone();
                }

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
            rule_id: summary_rule_id,
            active_rule_name: summary_rule_name,
            unsettled_match_count: unsettled_groups.len() as i64,
            players,
        })
    }

    pub fn recalculate_unsettled_with_rule(
        &self,
        rule_id: i64,
    ) -> Result<RecalculateUnsettledPointsResultDto, AppError> {
        let rule = PointRulesRepository::new(self.connection, self.account_id)
            .get_by_id(rule_id)?
            .ok_or_else(|| AppError::Message("Rule not found".to_string()))?;

        let unsettled_groups = self.get_history_match_groups(None, 0, true)?;
        if unsettled_groups.is_empty() {
            return Ok(RecalculateUnsettledPointsResultDto {
                rule_id: rule.id,
                rule_name: rule.name,
                recalculated_match_count: 0,
            });
        }

        let tx = self.connection.unchecked_transaction()?;
        let teammates_repo = TeammatesRepository::new(&tx, self.account_id);
        let mut affected_teammate_ids: HashSet<i64> = HashSet::new();

        for group in &unsettled_groups {
            for player in &group.players {
                let kill_points = player.kills.saturating_mul(rule.kill_points);
                let revive_points = player.revives.saturating_mul(rule.revive_points);
                let total_points = if player.is_points_enabled_snapshot {
                    apply_rounding(
                        player.damage * (rule.damage_points_per_damage as f64)
                            + (kill_points as f64)
                            + (revive_points as f64),
                        &rule.rounding_mode,
                    )
                } else {
                    0
                };

                tx.execute(
                    "UPDATE match_players
                     SET points = ?1
                     WHERE account_id = ?2 AND id = ?3",
                    params![total_points, self.account_id, player.match_player_id],
                )?;

                tx.execute(
                    "UPDATE point_records
                     SET rule_id = ?1,
                         rule_name_snapshot = ?2,
                         damage_points_per_damage_snapshot = ?3,
                         kill_points_snapshot = ?4,
                         revive_points_snapshot = ?5,
                         rounding_mode_snapshot = ?6,
                         points = ?7
                     WHERE account_id = ?8 AND match_player_id = ?9",
                    params![
                        rule.id,
                        &rule.name,
                        rule.damage_points_per_damage,
                        rule.kill_points,
                        rule.revive_points,
                        &rule.rounding_mode,
                        total_points,
                        self.account_id,
                        player.match_player_id,
                    ],
                )?;

                if let Some(teammate_id) = player.teammate_id {
                    affected_teammate_ids.insert(teammate_id);
                }
            }
        }

        for teammate_id in affected_teammate_ids {
            let total_points: i64 = tx.query_row(
                "SELECT COALESCE(SUM(points), 0)
                 FROM point_records
                 WHERE account_id = ?1 AND teammate_id = ?2",
                params![self.account_id, teammate_id],
                |row| row.get(0),
            )?;
            teammates_repo.update_total_points(teammate_id, total_points)?;
        }

        tx.commit()?;

        Ok(RecalculateUnsettledPointsResultDto {
            rule_id: rule.id,
            rule_name: rule.name,
            recalculated_match_count: unsettled_groups.len() as i64,
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
               mp.team_id,
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
                team_id: row.get(11)?,
                pubg_account_id: row.get(12)?,
                pubg_player_name: row.get(13)?,
                display_nickname_snapshot: row.get(14)?,
                is_self: row.get::<_, i64>(15)? == 1,
                is_points_enabled_snapshot: row.get::<_, i64>(16)? == 1,
                damage: row.get(17)?,
                kills: row.get(18)?,
                revives: row.get(19)?,
                damage_points_per_damage_snapshot: row.get(20)?,
                kill_points_snapshot: row.get(21)?,
                revive_points_snapshot: row.get(22)?,
                rounding_mode_snapshot: row.get(23)?,
                total_points: row.get(24)?,
            })
        })?;

        let mut grouped_items: Vec<PointHistoryMatchGroupDto> = Vec::new();
        let mut group_indices: HashMap<String, usize> = HashMap::new();
        let mut group_player_team_ids: Vec<Vec<Option<i64>>> = Vec::new();
        let mut self_team_ids: Vec<Option<i64>> = Vec::new();

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
                group_player_team_ids.push(Vec::new());
                self_team_ids.push(None);
                group_indices.insert(row.match_id.clone(), index);
                index
            };

            if row.is_self {
                self_team_ids[group_index] = row.team_id;
            }

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
            group_player_team_ids[group_index].push(row.team_id);
        }

        for (group_index, group) in grouped_items.iter_mut().enumerate() {
            let players = std::mem::take(&mut group.players);
            let team_ids = &group_player_team_ids[group_index];
            let self_team_id = self_team_ids[group_index];

            group.players = players
                .into_iter()
                .zip(team_ids.iter().copied())
                .filter(|(player, team_id)| {
                    player.is_self || self_team_id.is_some_and(|value| *team_id == Some(value))
                })
                .map(|(player, _)| player)
                .collect();

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
    team_id: Option<i64>,
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
    use rusqlite::{params, Connection};

    use crate::db::migrations::bootstrap_database;

    use super::{
        calculate_battle_deltas_for_players, PointHistoryListItemDto,
        PointHistoryPlayerBreakdownDto, PointRecordsRepository,
    };

    fn setup_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        bootstrap_database(&connection).expect("bootstrap schema");
        connection
    }

    fn active_account_id(connection: &Connection) -> i64 {
        connection
            .query_row(
                "SELECT id FROM accounts WHERE is_active = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("select active account")
    }

    fn active_rule(connection: &Connection, account_id: i64) -> (i64, String) {
        connection
            .query_row(
                "SELECT id, name FROM point_rules WHERE account_id = ?1 AND is_active = 1 LIMIT 1",
                [account_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("select active rule")
    }

    fn insert_rule(
        connection: &Connection,
        account_id: i64,
        name: &str,
        damage_points_per_damage: i64,
        kill_points: i64,
        revive_points: i64,
        is_active: bool,
    ) -> i64 {
        connection
            .execute(
                "INSERT INTO point_rules
                 (account_id, name, damage_points_per_damage, kill_points, revive_points, is_active, is_deleted, rounding_mode, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 'round', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                params![
                    account_id,
                    name,
                    damage_points_per_damage,
                    kill_points,
                    revive_points,
                    if is_active { 1 } else { 0 },
                ],
            )
            .expect("insert rule");
        connection.last_insert_rowid()
    }

    struct PlayerSeed<'a> {
        teammate_id: Option<i64>,
        pubg_player_name: &'a str,
        team_id: Option<i64>,
        damage: f64,
        kills: i64,
        revives: i64,
        is_self: bool,
        is_points_enabled_snapshot: bool,
        points: i64,
    }

    fn insert_teammate(
        connection: &Connection,
        account_id: i64,
        id: i64,
        name: &str,
        is_points_enabled: bool,
        total_points: i64,
    ) {
        connection
            .execute(
                "INSERT INTO teammates
                 (id, account_id, platform, pubg_account_id, pubg_player_name, display_nickname, is_points_enabled, total_points, last_seen_at, created_at, updated_at)
                 VALUES (?1, ?2, 'steam', NULL, ?3, NULL, ?4, ?5, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                params![
                    id,
                    account_id,
                    name,
                    if is_points_enabled { 1 } else { 0 },
                    total_points,
                ],
            )
            .expect("insert teammate");
    }

    fn insert_match_with_players(
        connection: &Connection,
        account_id: i64,
        rule_id: i64,
        rule_name: &str,
        match_id: &str,
        played_at: &str,
        settled_at: Option<&str>,
        players: &[PlayerSeed<'_>],
    ) {
        connection
            .execute(
                "INSERT INTO matches
                 (account_id, match_id, platform, map_name, game_mode, played_at, status, created_at, updated_at)
                 VALUES (?1, ?2, 'steam', 'Erangel', 'squad', ?3, 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                params![account_id, match_id, played_at],
            )
            .expect("insert match");

        if let Some(settled_at) = settled_at {
            connection
                .execute(
                    "INSERT INTO point_match_meta
                     (account_id, match_id, note, settled_at, settlement_batch_id, created_at, updated_at)
                     VALUES (?1, ?2, NULL, ?3, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    params![account_id, match_id, settled_at],
                )
                .expect("insert settled meta");
        }

        for player in players {
            connection
                .execute(
                    "INSERT INTO match_players
                     (account_id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
                      team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at)
                     VALUES (?1, ?2, ?3, NULL, ?4, NULL, ?5, ?6, ?7, ?8, 1, ?9, ?10, ?11, CURRENT_TIMESTAMP)",
                    params![
                        account_id,
                        match_id,
                        player.teammate_id,
                        player.pubg_player_name,
                        player.team_id,
                        player.damage,
                        player.kills,
                        player.revives,
                        if player.is_self { 1 } else { 0 },
                        if player.is_points_enabled_snapshot { 1 } else { 0 },
                        player.points,
                    ],
                )
                .expect("insert match player");
            let match_player_id = connection.last_insert_rowid();

            connection
                .execute(
                    "INSERT INTO point_records
                     (account_id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
                      damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot,
                      rounding_mode_snapshot, points, note, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 10, 0, 'round', ?7, NULL, CURRENT_TIMESTAMP)",
                    params![
                        account_id,
                        match_id,
                        match_player_id,
                        player.teammate_id,
                        rule_id,
                        rule_name,
                        player.points,
                    ],
                )
                .expect("insert point record");
        }
    }

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

    #[test]
    fn recalculate_unsettled_updates_only_unsettled_matches_and_teammate_totals() {
        let connection = setup_connection();
        let account_id = active_account_id(&connection);
        let (old_rule_id, _) = active_rule(&connection, account_id);
        connection
            .execute(
                "UPDATE point_rules
                 SET name = 'Old Rule', damage_points_per_damage = 1, kill_points = 10, revive_points = 0, rounding_mode = 'round', updated_at = CURRENT_TIMESTAMP
                 WHERE account_id = ?1 AND id = ?2",
                params![account_id, old_rule_id],
            )
            .expect("update active rule");
        let new_rule_id = insert_rule(&connection, account_id, "New Rule", 2, 100, 50, false);

        insert_teammate(&connection, account_id, 101, "Alpha", true, 250);
        insert_teammate(&connection, account_id, 102, "Bravo", true, 280);

        insert_match_with_players(
            &connection,
            account_id,
            old_rule_id,
            "Old Rule",
            "unsettled-1",
            "2026-01-03T00:00:00Z",
            None,
            &[
                PlayerSeed {
                    teammate_id: Some(101),
                    pubg_player_name: "Alpha",
                    team_id: Some(1),
                    damage: 100.0,
                    kills: 1,
                    revives: 0,
                    is_self: true,
                    is_points_enabled_snapshot: true,
                    points: 110,
                },
                PlayerSeed {
                    teammate_id: Some(102),
                    pubg_player_name: "Bravo",
                    team_id: Some(1),
                    damage: 50.0,
                    kills: 2,
                    revives: 0,
                    is_self: false,
                    is_points_enabled_snapshot: true,
                    points: 70,
                },
            ],
        );

        insert_match_with_players(
            &connection,
            account_id,
            old_rule_id,
            "Old Rule",
            "settled-1",
            "2026-01-02T00:00:00Z",
            Some("2026-01-05T00:00:00Z"),
            &[
                PlayerSeed {
                    teammate_id: Some(101),
                    pubg_player_name: "Alpha",
                    team_id: Some(1),
                    damage: 100.0,
                    kills: 0,
                    revives: 0,
                    is_self: true,
                    is_points_enabled_snapshot: true,
                    points: 100,
                },
                PlayerSeed {
                    teammate_id: Some(102),
                    pubg_player_name: "Bravo",
                    team_id: Some(1),
                    damage: 80.0,
                    kills: 1,
                    revives: 0,
                    is_self: false,
                    is_points_enabled_snapshot: true,
                    points: 180,
                },
            ],
        );

        let repository = PointRecordsRepository::new(&connection, account_id);
        let result = repository
            .recalculate_unsettled_with_rule(new_rule_id)
            .expect("recalculate unsettled matches");

        assert_eq!(result.rule_id, new_rule_id);
        assert_eq!(result.rule_name, "New Rule");
        assert_eq!(result.recalculated_match_count, 1);

        let unsettled_records: Vec<(String, i64, i64)> = {
            let mut statement = connection
                .prepare(
                    "SELECT rule_name_snapshot, damage_points_per_damage_snapshot, points
                     FROM point_records
                     WHERE account_id = ?1 AND match_id = 'unsettled-1'
                     ORDER BY teammate_id ASC",
                )
                .expect("prepare unsettled records");

            statement
                .query_map([account_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .expect("query unsettled records")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect unsettled records")
        };
        assert_eq!(unsettled_records[0], ("New Rule".to_string(), 2, 300));
        assert_eq!(unsettled_records[1], ("New Rule".to_string(), 2, 300));

        let settled_record: (String, i64) = connection
            .query_row(
                "SELECT rule_name_snapshot, points
                 FROM point_records
                 WHERE account_id = ?1 AND match_id = 'settled-1' AND teammate_id = 101",
                [account_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("select settled record");
        assert_eq!(settled_record, ("Old Rule".to_string(), 100));

        let unsettled_match_player_points: Vec<i64> = {
            let mut statement = connection
                .prepare(
                    "SELECT points FROM match_players
                     WHERE account_id = ?1 AND match_id = 'unsettled-1'
                     ORDER BY teammate_id ASC",
                )
                .expect("prepare unsettled match players");
            statement
                .query_map([account_id], |row| row.get(0))
                .expect("query unsettled match players")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect unsettled match players")
        };
        assert_eq!(unsettled_match_player_points, vec![300, 300]);

        let teammate_totals: Vec<(i64, i64)> = {
            let mut statement = connection
                .prepare(
                    "SELECT id, total_points FROM teammates
                     WHERE account_id = ?1
                     ORDER BY id ASC",
                )
                .expect("prepare teammate totals");
            statement
                .query_map([account_id], |row| Ok((row.get(0)?, row.get(1)?)))
                .expect("query teammate totals")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect teammate totals")
        };
        assert_eq!(teammate_totals, vec![(101, 400), (102, 480)]);
    }

    #[test]
    fn repair_points_with_current_identities_updates_self_enabled_flags_and_totals() {
        let connection = setup_connection();
        let account_id = active_account_id(&connection);
        let (rule_id, rule_name) = active_rule(&connection, account_id);

        insert_teammate(&connection, account_id, 201, "Bravo", false, 999);
        insert_match_with_players(
            &connection,
            account_id,
            rule_id,
            &rule_name,
            "repair-1",
            "2026-01-04T00:00:00Z",
            None,
            &[
                PlayerSeed {
                    teammate_id: None,
                    pubg_player_name: "SelfPlayer",
                    team_id: Some(1),
                    damage: 100.0,
                    kills: 1,
                    revives: 1,
                    is_self: false,
                    is_points_enabled_snapshot: false,
                    points: 0,
                },
                PlayerSeed {
                    teammate_id: Some(201),
                    pubg_player_name: "Bravo",
                    team_id: Some(1),
                    damage: 80.0,
                    kills: 1,
                    revives: 0,
                    is_self: false,
                    is_points_enabled_snapshot: false,
                    points: 0,
                },
                PlayerSeed {
                    teammate_id: None,
                    pubg_player_name: "Random",
                    team_id: Some(2),
                    damage: 120.0,
                    kills: 3,
                    revives: 0,
                    is_self: false,
                    is_points_enabled_snapshot: true,
                    points: 999,
                },
            ],
        );

        let repaired_match_count = PointRecordsRepository::new(&connection, account_id)
            .repair_points_with_current_identities("SelfPlayer")
            .expect("repair point history");

        assert_eq!(repaired_match_count, 1);

        let repaired_players: Vec<(String, bool, bool, i64)> = {
            let mut statement = connection
                .prepare(
                    "SELECT pubg_player_name, is_self, is_points_enabled_snapshot, points
                     FROM match_players
                     WHERE account_id = ?1 AND match_id = 'repair-1'
                     ORDER BY id ASC",
                )
                .expect("prepare repaired players");

            statement
                .query_map([account_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)? == 1,
                        row.get::<_, i64>(2)? == 1,
                        row.get::<_, i64>(3)?,
                    ))
                })
                .expect("query repaired players")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect repaired players")
        };

        assert_eq!(
            repaired_players,
            vec![
                ("SelfPlayer".to_string(), true, true, 110),
                ("Bravo".to_string(), false, false, 0),
                ("Random".to_string(), false, false, 0),
            ]
        );

        let repaired_records: Vec<(String, i64)> = {
            let mut statement = connection
                .prepare(
                    "SELECT mp.pubg_player_name, pr.points
                     FROM point_records pr
                     INNER JOIN match_players mp
                       ON mp.account_id = pr.account_id
                      AND mp.id = pr.match_player_id
                     WHERE pr.account_id = ?1 AND pr.match_id = 'repair-1'
                     ORDER BY mp.id ASC",
                )
                .expect("prepare repaired records");

            statement
                .query_map([account_id], |row| Ok((row.get(0)?, row.get(1)?)))
                .expect("query repaired records")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect repaired records")
        };
        assert_eq!(
            repaired_records,
            vec![
                ("SelfPlayer".to_string(), 110),
                ("Bravo".to_string(), 0),
                ("Random".to_string(), 0),
            ]
        );

        let teammate_total: i64 = connection
            .query_row(
                "SELECT total_points FROM teammates WHERE account_id = ?1 AND id = 201",
                [account_id],
                |row| row.get(0),
            )
            .expect("select teammate total");
        assert_eq!(teammate_total, 0);
    }

    #[test]
    fn history_and_unsettled_summary_include_same_team_non_participants_but_exclude_enemies() {
        let connection = setup_connection();
        let account_id = active_account_id(&connection);
        let (rule_id, rule_name) = active_rule(&connection, account_id);

        insert_teammate(&connection, account_id, 301, "SquadMate", false, 0);

        insert_match_with_players(
            &connection,
            account_id,
            rule_id,
            &rule_name,
            "team-filter-1",
            "2026-01-06T00:00:00Z",
            None,
            &[
                PlayerSeed {
                    teammate_id: None,
                    pubg_player_name: "SelfPlayer",
                    team_id: Some(10),
                    damage: 100.0,
                    kills: 1,
                    revives: 0,
                    is_self: true,
                    is_points_enabled_snapshot: true,
                    points: 110,
                },
                PlayerSeed {
                    teammate_id: Some(301),
                    pubg_player_name: "SquadMate",
                    team_id: Some(10),
                    damage: 80.0,
                    kills: 0,
                    revives: 0,
                    is_self: false,
                    is_points_enabled_snapshot: false,
                    points: 0,
                },
                PlayerSeed {
                    teammate_id: None,
                    pubg_player_name: "Enemy",
                    team_id: Some(20),
                    damage: 200.0,
                    kills: 4,
                    revives: 0,
                    is_self: false,
                    is_points_enabled_snapshot: true,
                    points: 240,
                },
            ],
        );

        let repository = PointRecordsRepository::new(&connection, account_id);
        let history = repository
            .get_history_groups(10, 0)
            .expect("load history groups");

        let PointHistoryListItemDto::MatchGroup(group) = &history[0] else {
            panic!("expected match group");
        };
        let player_names = group
            .players
            .iter()
            .map(|player| player.pubg_player_name.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            player_names,
            vec!["SelfPlayer".to_string(), "SquadMate".to_string()]
        );
        assert_eq!(group.players[0].is_points_enabled_snapshot, true);
        assert_eq!(group.players[1].is_points_enabled_snapshot, false);
        assert_eq!(group.players[1].total_points, 0);

        let unsettled_summary = repository
            .get_unsettled_summary()
            .expect("load unsettled summary");
        let unsettled_names = unsettled_summary
            .players
            .iter()
            .map(|player| player.pubg_player_name.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            unsettled_names,
            vec!["SelfPlayer".to_string(), "SquadMate".to_string()]
        );
        assert_eq!(unsettled_summary.players[0].total_delta, 0);
        assert_eq!(unsettled_summary.players[1].total_delta, 0);
    }

    #[test]
    fn unsettled_summary_merges_same_player_even_if_teammate_id_changed() {
        let connection = setup_connection();
        let account_id = active_account_id(&connection);
        let (rule_id, rule_name) = active_rule(&connection, account_id);

        insert_teammate(&connection, account_id, 401, "Bravo", false, 0);
        insert_teammate(&connection, account_id, 402, "Bravo", false, 0);

        insert_match_with_players(
            &connection,
            account_id,
            rule_id,
            &rule_name,
            "duplicate-summary-1",
            "2026-01-07T00:00:00Z",
            None,
            &[
                PlayerSeed {
                    teammate_id: None,
                    pubg_player_name: "SelfPlayer",
                    team_id: Some(1),
                    damage: 100.0,
                    kills: 1,
                    revives: 0,
                    is_self: true,
                    is_points_enabled_snapshot: true,
                    points: 110,
                },
                PlayerSeed {
                    teammate_id: Some(401),
                    pubg_player_name: "Bravo",
                    team_id: Some(1),
                    damage: 50.0,
                    kills: 1,
                    revives: 0,
                    is_self: false,
                    is_points_enabled_snapshot: false,
                    points: 0,
                },
            ],
        );

        insert_match_with_players(
            &connection,
            account_id,
            rule_id,
            &rule_name,
            "duplicate-summary-2",
            "2026-01-08T00:00:00Z",
            None,
            &[
                PlayerSeed {
                    teammate_id: None,
                    pubg_player_name: "SelfPlayer",
                    team_id: Some(1),
                    damage: 120.0,
                    kills: 1,
                    revives: 0,
                    is_self: true,
                    is_points_enabled_snapshot: true,
                    points: 130,
                },
                PlayerSeed {
                    teammate_id: Some(402),
                    pubg_player_name: "Bravo",
                    team_id: Some(1),
                    damage: 20.0,
                    kills: 0,
                    revives: 0,
                    is_self: false,
                    is_points_enabled_snapshot: false,
                    points: 0,
                },
            ],
        );

        let unsettled_summary = PointRecordsRepository::new(&connection, account_id)
            .get_unsettled_summary()
            .expect("load unsettled summary");

        let bravo_entries = unsettled_summary
            .players
            .iter()
            .filter(|player| player.pubg_player_name == "Bravo")
            .collect::<Vec<_>>();

        assert_eq!(bravo_entries.len(), 1);
        assert_eq!(bravo_entries[0].total_delta, 0);
    }
}
