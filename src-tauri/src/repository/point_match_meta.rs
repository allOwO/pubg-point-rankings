use rusqlite::{params, Connection};
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointMatchMetaDto {
    pub id: i64,
    pub account_id: i64,
    pub match_id: String,
    pub note: Option<String>,
    pub settled_at: Option<String>,
    pub settlement_batch_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleThroughMatchResultDto {
    pub settlement_batch_id: i64,
    pub settled_match_count: i64,
}

pub struct PointMatchMetaRepository<'a> {
    connection: &'a Connection,
    account_id: i64,
}

impl<'a> PointMatchMetaRepository<'a> {
    pub fn new(connection: &'a Connection, account_id: i64) -> Self {
        Self {
            connection,
            account_id,
        }
    }

    pub fn get_by_match(&self, match_id: &str) -> Result<Option<PointMatchMetaDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_id, match_id, note, settled_at, settlement_batch_id, created_at, updated_at
             FROM point_match_meta
             WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            Self::map_row,
        );

        match result {
            Ok(meta) => Ok(Some(meta)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn upsert_note(&self, match_id: &str, note: Option<String>) -> Result<(), AppError> {
        self.ensure_match_exists(match_id)?;

        self.connection.execute(
            "INSERT INTO point_match_meta
             (account_id, match_id, note, settled_at, settlement_batch_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT(account_id, match_id) DO UPDATE SET
               note = excluded.note,
               updated_at = CURRENT_TIMESTAMP",
            params![self.account_id, match_id, note],
        )?;

        Ok(())
    }

    pub fn settle_through_match(
        &self,
        end_match_id: &str,
    ) -> Result<SettleThroughMatchResultDto, AppError> {
        let unsettled_match_ids = self.get_unsettled_match_ids()?;
        if unsettled_match_ids.is_empty() {
            return Err(AppError::Message(
                "No unsettled matches available".to_string(),
            ));
        }

        let end_index = unsettled_match_ids
            .iter()
            .position(|match_id| match_id == end_match_id)
            .ok_or_else(|| {
                AppError::Message("Target endMatchId is not in unsettled interval".to_string())
            })?;

        let matches_to_settle = &unsettled_match_ids[..=end_index];
        let start_match_id = matches_to_settle
            .first()
            .ok_or_else(|| AppError::Message("No matches to settle".to_string()))?
            .clone();

        let rule_snapshot = self.connection.query_row(
            "SELECT rule_id, rule_name_snapshot
             FROM point_records
             WHERE account_id = ?1 AND match_id = ?2
             ORDER BY id ASC
             LIMIT 1",
            params![self.account_id, end_match_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        );

        let (rule_id_snapshot, rule_name_snapshot) = match rule_snapshot {
            Ok((rule_id, rule_name)) => (Some(rule_id), Some(rule_name)),
            Err(rusqlite::Error::QueryReturnedNoRows) => (None, None),
            Err(error) => return Err(error.into()),
        };

        let tx = self.connection.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO point_settlement_batches
             (account_id, start_match_id, end_match_id, rule_id_snapshot, rule_name_snapshot, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                start_match_id,
                end_match_id,
                rule_id_snapshot,
                rule_name_snapshot,
            ],
        )?;
        let settlement_batch_id = tx.last_insert_rowid();

        for match_id in matches_to_settle {
            tx.execute(
                "INSERT INTO point_match_meta
                 (account_id, match_id, note, settled_at, settlement_batch_id, created_at, updated_at)
                 VALUES (?1, ?2, NULL, CURRENT_TIMESTAMP, ?3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                 ON CONFLICT(account_id, match_id) DO UPDATE SET
                   settled_at = CASE
                     WHEN point_match_meta.settled_at IS NULL THEN excluded.settled_at
                     ELSE point_match_meta.settled_at
                   END,
                   settlement_batch_id = CASE
                     WHEN point_match_meta.settled_at IS NULL THEN excluded.settlement_batch_id
                     ELSE point_match_meta.settlement_batch_id
                   END,
                   updated_at = CURRENT_TIMESTAMP",
                params![self.account_id, match_id, settlement_batch_id],
            )?;
        }

        tx.commit()?;

        Ok(SettleThroughMatchResultDto {
            settlement_batch_id,
            settled_match_count: matches_to_settle.len() as i64,
        })
    }

    pub fn settle_single_match(&self, match_id: &str) -> Result<(), AppError> {
        self.ensure_match_exists(match_id)?;

        let already_settled: i64 = self.connection.query_row(
            "SELECT COUNT(*)
             FROM point_match_meta
             WHERE account_id = ?1
               AND match_id = ?2
               AND settled_at IS NOT NULL",
            params![self.account_id, match_id],
            |row| row.get(0),
        )?;
        if already_settled > 0 {
            return Ok(());
        }

        let has_points: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM point_records WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            |row| row.get(0),
        )?;
        if has_points == 0 {
            return Ok(());
        }

        let rule_snapshot = self.connection.query_row(
            "SELECT rule_id, rule_name_snapshot
             FROM point_records
             WHERE account_id = ?1 AND match_id = ?2
             ORDER BY id ASC
             LIMIT 1",
            params![self.account_id, match_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        );

        let (rule_id_snapshot, rule_name_snapshot) = match rule_snapshot {
            Ok((rule_id, rule_name)) => (Some(rule_id), Some(rule_name)),
            Err(rusqlite::Error::QueryReturnedNoRows) => (None, None),
            Err(error) => return Err(error.into()),
        };

        let tx = self.connection.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO point_settlement_batches
             (account_id, start_match_id, end_match_id, rule_id_snapshot, rule_name_snapshot, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)",
            params![
                self.account_id,
                match_id,
                match_id,
                rule_id_snapshot,
                rule_name_snapshot,
            ],
        )?;
        let settlement_batch_id = tx.last_insert_rowid();

        tx.execute(
            "INSERT INTO point_match_meta
             (account_id, match_id, note, settled_at, settlement_batch_id, created_at, updated_at)
             VALUES (?1, ?2, NULL, CURRENT_TIMESTAMP, ?3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT(account_id, match_id) DO UPDATE SET
               settled_at = CASE
                 WHEN point_match_meta.settled_at IS NULL THEN excluded.settled_at
                 ELSE point_match_meta.settled_at
               END,
               settlement_batch_id = CASE
                 WHEN point_match_meta.settled_at IS NULL THEN excluded.settlement_batch_id
                 ELSE point_match_meta.settlement_batch_id
               END,
               updated_at = CURRENT_TIMESTAMP",
            params![self.account_id, match_id, settlement_batch_id],
        )?;

        tx.commit()?;

        Ok(())
    }

    fn ensure_match_exists(&self, match_id: &str) -> Result<(), AppError> {
        let exists: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM matches WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
            |row| row.get(0),
        )?;

        if exists == 0 {
            return Err(AppError::Message("Match not found".to_string()));
        }

        Ok(())
    }

    fn get_unsettled_match_ids(&self) -> Result<Vec<String>, AppError> {
        let mut statement = self.connection.prepare(
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
             ORDER BY m.played_at ASC, m.match_id ASC",
        )?;

        let rows = statement.query_map([self.account_id], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PointMatchMetaDto> {
        Ok(PointMatchMetaDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            match_id: row.get(2)?,
            note: row.get(3)?,
            settled_at: row.get(4)?,
            settlement_batch_id: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use crate::db::migrations::bootstrap_database;

    use super::PointMatchMetaRepository;

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

    fn active_rule_snapshot(connection: &Connection, account_id: i64) -> (i64, String) {
        connection
            .query_row(
                "SELECT id, name FROM point_rules WHERE account_id = ?1 AND is_active = 1 LIMIT 1",
                [account_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("select active rule")
    }

    fn insert_match_with_point(
        connection: &Connection,
        account_id: i64,
        rule_id: i64,
        rule_name: &str,
        match_id: &str,
        played_at: &str,
    ) {
        connection
            .execute(
                "INSERT INTO matches
                 (account_id, match_id, platform, map_name, game_mode, played_at, status, created_at, updated_at)
                 VALUES (?1, ?2, 'steam', 'Erangel', 'squad', ?3, 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                params![account_id, match_id, played_at],
            )
            .expect("insert match");

        connection
            .execute(
                "INSERT INTO match_players
                 (account_id, match_id, teammate_id, pubg_account_id, pubg_player_name, display_nickname_snapshot,
                  team_id, damage, kills, revives, placement, is_self, is_points_enabled_snapshot, points, created_at)
                 VALUES (?1, ?2, NULL, NULL, 'Player', NULL, 1, 100, 1, 0, 1, 1, 1, 500, CURRENT_TIMESTAMP)",
                params![account_id, match_id],
            )
            .expect("insert player");
        let match_player_id = connection.last_insert_rowid();

        connection
            .execute(
                "INSERT INTO point_records
                 (account_id, match_id, match_player_id, teammate_id, rule_id, rule_name_snapshot,
                  damage_points_per_damage_snapshot, kill_points_snapshot, revive_points_snapshot,
                  rounding_mode_snapshot, points, note, created_at)
                 VALUES (?1, ?2, ?3, NULL, ?4, ?5, 2, 300, 150, 'round', 500, NULL, CURRENT_TIMESTAMP)",
                params![account_id, match_id, match_player_id, rule_id, rule_name],
            )
            .expect("insert point record");
    }

    #[test]
    fn upsert_note_inserts_then_updates_single_meta_row() {
        let connection = setup_connection();
        let account_id = active_account_id(&connection);
        let (rule_id, rule_name) = active_rule_snapshot(&connection, account_id);

        insert_match_with_point(
            &connection,
            account_id,
            rule_id,
            &rule_name,
            "match-note",
            "2026-01-01T00:00:00Z",
        );

        let repository = PointMatchMetaRepository::new(&connection, account_id);
        repository
            .upsert_note("match-note", Some("first note".to_string()))
            .expect("insert note");
        repository
            .upsert_note("match-note", Some("updated note".to_string()))
            .expect("update note");

        let note: String = connection
            .query_row(
                "SELECT note FROM point_match_meta WHERE account_id = ?1 AND match_id = 'match-note'",
                [account_id],
                |row| row.get(0),
            )
            .expect("select note");
        let row_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM point_match_meta WHERE account_id = ?1 AND match_id = 'match-note'",
                [account_id],
                |row| row.get(0),
            )
            .expect("count meta rows");

        assert_eq!(note, "updated note");
        assert_eq!(row_count, 1);
    }

    #[test]
    fn settle_through_match_sets_only_target_interval_and_skips_previously_settled() {
        let connection = setup_connection();
        let account_id = active_account_id(&connection);
        let (rule_id, rule_name) = active_rule_snapshot(&connection, account_id);

        insert_match_with_point(
            &connection,
            account_id,
            rule_id,
            &rule_name,
            "match-1",
            "2026-01-01T00:00:00Z",
        );
        insert_match_with_point(
            &connection,
            account_id,
            rule_id,
            &rule_name,
            "match-2",
            "2026-01-02T00:00:00Z",
        );
        insert_match_with_point(
            &connection,
            account_id,
            rule_id,
            &rule_name,
            "match-3",
            "2026-01-03T00:00:00Z",
        );

        let repository = PointMatchMetaRepository::new(&connection, account_id);

        let first_batch = repository
            .settle_through_match("match-2")
            .expect("settle interval through match-2");
        let second_batch = repository
            .settle_through_match("match-3")
            .expect("settle remaining through match-3");

        assert_eq!(first_batch.settled_match_count, 2);
        assert_eq!(second_batch.settled_match_count, 1);

        let match_1_batch: i64 = connection
            .query_row(
                "SELECT settlement_batch_id FROM point_match_meta WHERE account_id = ?1 AND match_id = 'match-1'",
                [account_id],
                |row| row.get(0),
            )
            .expect("match-1 batch");
        let match_2_batch: i64 = connection
            .query_row(
                "SELECT settlement_batch_id FROM point_match_meta WHERE account_id = ?1 AND match_id = 'match-2'",
                [account_id],
                |row| row.get(0),
            )
            .expect("match-2 batch");
        let match_3_batch: i64 = connection
            .query_row(
                "SELECT settlement_batch_id FROM point_match_meta WHERE account_id = ?1 AND match_id = 'match-3'",
                [account_id],
                |row| row.get(0),
            )
            .expect("match-3 batch");

        let first_batch_start_end: (String, String) = connection
            .query_row(
                "SELECT start_match_id, end_match_id FROM point_settlement_batches WHERE id = ?1",
                [first_batch.settlement_batch_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("first batch interval");
        let second_batch_start_end: (String, String) = connection
            .query_row(
                "SELECT start_match_id, end_match_id FROM point_settlement_batches WHERE id = ?1",
                [second_batch.settlement_batch_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("second batch interval");

        assert_eq!(match_1_batch, first_batch.settlement_batch_id);
        assert_eq!(match_2_batch, first_batch.settlement_batch_id);
        assert_eq!(match_3_batch, second_batch.settlement_batch_id);
        assert_eq!(
            first_batch_start_end,
            ("match-1".to_string(), "match-2".to_string())
        );
        assert_eq!(
            second_batch_start_end,
            ("match-3".to_string(), "match-3".to_string())
        );
    }

    #[test]
    fn settle_single_match_sets_only_target_match() {
        let connection = setup_connection();
        let account_id = active_account_id(&connection);
        let (rule_id, rule_name) = active_rule_snapshot(&connection, account_id);

        insert_match_with_point(
            &connection,
            account_id,
            rule_id,
            &rule_name,
            "match-single-1",
            "2026-01-01T00:00:00Z",
        );
        insert_match_with_point(
            &connection,
            account_id,
            rule_id,
            &rule_name,
            "match-single-2",
            "2026-01-02T00:00:00Z",
        );

        let repository = PointMatchMetaRepository::new(&connection, account_id);
        repository
            .settle_single_match("match-single-2")
            .expect("settle exact match-single-2");

        let match_1_settled_at: Option<String> = connection
            .query_row(
                "SELECT settled_at FROM point_match_meta WHERE account_id = ?1 AND match_id = 'match-single-1'",
                [account_id],
                |row| row.get(0),
            )
            .unwrap_or(None);
        let match_2_settled_at: Option<String> = connection
            .query_row(
                "SELECT settled_at FROM point_match_meta WHERE account_id = ?1 AND match_id = 'match-single-2'",
                [account_id],
                |row| row.get(0),
            )
            .expect("match-single-2 settled_at");

        assert!(match_1_settled_at.is_none());
        assert!(match_2_settled_at.is_some());
    }
}
