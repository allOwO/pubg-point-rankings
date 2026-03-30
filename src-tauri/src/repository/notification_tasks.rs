use rusqlite::{params, Connection};
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct CreateNotificationTaskInput {
    pub match_id: String,
    pub message_body: String,
    pub preview_match_time: String,
    pub preview_placement: Option<i64>,
    pub preview_battle_summary: String,
}

#[derive(Debug, Clone)]
pub struct NotificationTaskDto {
    pub id: i64,
    pub account_id: i64,
    pub match_id: String,
    pub status: String,
    pub retry_count: i64,
    pub next_retry_at: Option<String>,
    pub message_body: String,
    pub preview_match_time: String,
    pub preview_placement: Option<i64>,
    pub preview_battle_summary: String,
    pub last_error: Option<String>,
    pub sent_at: Option<String>,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationFailedTaskDto {
    pub id: i64,
    pub match_id: String,
    pub match_time: String,
    pub placement: Option<i64>,
    pub battle_summary: String,
    pub last_error: Option<String>,
    pub send_status: String,
}

#[derive(Debug, Clone)]
pub struct FailedNotificationTaskRow {
    pub id: i64,
    pub match_id: String,
    pub message_body: String,
}

pub struct NotificationTasksRepository<'a> {
    connection: &'a Connection,
    account_id: i64,
}

impl<'a> NotificationTasksRepository<'a> {
    pub fn new(connection: &'a Connection, account_id: i64) -> Self {
        Self {
            connection,
            account_id,
        }
    }

    pub fn upsert_pending(&self, input: CreateNotificationTaskInput) -> Result<(), AppError> {
        self.connection.execute(
            "INSERT INTO match_notification_tasks
             (account_id, match_id, status, retry_count, next_retry_at, message_body, preview_match_time,
              preview_placement, preview_battle_summary, last_error, sent_at, deleted_at, created_at, updated_at)
             VALUES (?1, ?2, 'pending', 0, NULL, ?3, ?4, ?5, ?6, NULL, NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT(account_id, match_id) DO UPDATE SET
               status = 'pending',
               retry_count = 0,
               next_retry_at = NULL,
               message_body = excluded.message_body,
               preview_match_time = excluded.preview_match_time,
               preview_placement = excluded.preview_placement,
               preview_battle_summary = excluded.preview_battle_summary,
               last_error = NULL,
               sent_at = NULL,
               deleted_at = NULL,
               updated_at = CURRENT_TIMESTAMP",
            params![
                self.account_id,
                input.match_id,
                input.message_body,
                input.preview_match_time,
                input.preview_placement,
                input.preview_battle_summary,
            ],
        )?;

        Ok(())
    }

    pub fn mark_sending(&self, task_id: i64) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE match_notification_tasks
             SET status = 'sending', updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1 AND account_id = ?2",
            params![task_id, self.account_id],
        )?;

        Ok(())
    }

    pub fn mark_retry_scheduled(
        &self,
        task_id: i64,
        retry_count: i64,
        error: &str,
        next_retry_at: &str,
    ) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE match_notification_tasks
             SET status = 'retrying', retry_count = ?1, last_error = ?2, next_retry_at = ?3, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?4 AND account_id = ?5",
            params![retry_count, error, next_retry_at, task_id, self.account_id],
        )?;

        Ok(())
    }

    pub fn mark_failed_manual(&self, task_id: i64, error: &str) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE match_notification_tasks
             SET status = 'failed_manual', last_error = ?1, next_retry_at = NULL, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?2 AND account_id = ?3",
            params![error, task_id, self.account_id],
        )?;

        Ok(())
    }

    pub fn mark_sent(&self, task_id: i64) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE match_notification_tasks
             SET status = 'sent', last_error = NULL, next_retry_at = NULL, sent_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1 AND account_id = ?2",
            params![task_id, self.account_id],
        )?;

        Ok(())
    }

    pub fn mark_deleted(&self, task_id: i64) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE match_notification_tasks
             SET status = 'deleted', next_retry_at = NULL, deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1 AND account_id = ?2",
            params![task_id, self.account_id],
        )?;

        Ok(())
    }

    pub fn mark_cancelled_settled(&self, match_id: &str) -> Result<(), AppError> {
        self.connection.execute(
            "UPDATE match_notification_tasks
             SET status = 'cancelled_settled', next_retry_at = NULL, updated_at = CURRENT_TIMESTAMP
             WHERE account_id = ?1 AND match_id = ?2",
            params![self.account_id, match_id],
        )?;

        Ok(())
    }

    pub fn get_failed_manual(&self) -> Result<Vec<NotificationFailedTaskDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, match_id, preview_match_time, preview_placement, preview_battle_summary, last_error
             FROM match_notification_tasks
             WHERE account_id = ?1 AND status = 'failed_manual'
             ORDER BY updated_at DESC, id DESC",
        )?;

        let rows = statement.query_map([self.account_id], |row| {
            Ok(NotificationFailedTaskDto {
                id: row.get(0)?,
                match_id: row.get(1)?,
                match_time: row.get(2)?,
                placement: row.get(3)?,
                battle_summary: row.get(4)?,
                last_error: row.get(5)?,
                send_status: "failed".to_string(),
            })
        })?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_due_retries(&self, now_iso: &str) -> Result<Vec<NotificationTaskDto>, AppError> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, match_id, status, retry_count, next_retry_at, message_body,
                    preview_match_time, preview_placement, preview_battle_summary, last_error,
                    sent_at, deleted_at, created_at, updated_at
             FROM match_notification_tasks
             WHERE account_id = ?1
               AND status IN ('pending', 'retrying')
               AND (next_retry_at IS NULL OR next_retry_at <= ?2)
             ORDER BY CASE WHEN next_retry_at IS NULL THEN 0 ELSE 1 END ASC,
                      next_retry_at ASC,
                      id ASC",
        )?;

        let rows = statement.query_map(params![self.account_id, now_iso], Self::map_task_row)?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_pending_by_match(
        &self,
        match_id: &str,
    ) -> Result<Option<NotificationTaskDto>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, account_id, match_id, status, retry_count, next_retry_at, message_body,
                    preview_match_time, preview_placement, preview_battle_summary, last_error,
                    sent_at, deleted_at, created_at, updated_at
             FROM match_notification_tasks
             WHERE account_id = ?1 AND match_id = ?2 AND status IN ('pending', 'retrying', 'sending')
             LIMIT 1",
            params![self.account_id, match_id],
            Self::map_task_row,
        );

        match result {
            Ok(task) => Ok(Some(task)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_failed_manual_by_id(
        &self,
        task_id: i64,
    ) -> Result<Option<FailedNotificationTaskRow>, AppError> {
        let result = self.connection.query_row(
            "SELECT id, match_id, message_body
             FROM match_notification_tasks
             WHERE account_id = ?1 AND id = ?2 AND status = 'failed_manual'
             LIMIT 1",
            params![self.account_id, task_id],
            |row| {
                Ok(FailedNotificationTaskRow {
                    id: row.get(0)?,
                    match_id: row.get(1)?,
                    message_body: row.get(2)?,
                })
            },
        );

        match result {
            Ok(task) => Ok(Some(task)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn map_task_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<NotificationTaskDto> {
        Ok(NotificationTaskDto {
            id: row.get(0)?,
            account_id: row.get(1)?,
            match_id: row.get(2)?,
            status: row.get(3)?,
            retry_count: row.get(4)?,
            next_retry_at: row.get(5)?,
            message_body: row.get(6)?,
            preview_match_time: row.get(7)?,
            preview_placement: row.get(8)?,
            preview_battle_summary: row.get(9)?,
            last_error: row.get(10)?,
            sent_at: row.get(11)?,
            deleted_at: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::db::migrations::bootstrap_database;

    use super::{CreateNotificationTaskInput, NotificationTasksRepository};

    #[test]
    fn marks_task_failed_manual_after_final_retry_and_lists_it() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        bootstrap_database(&connection).expect("bootstrap schema");

        let account_id: i64 = connection
            .query_row(
                "SELECT id FROM accounts WHERE is_active = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("select active account");

        connection
            .execute(
                "INSERT INTO matches (account_id, match_id, platform, map_name, game_mode, played_at, status, created_at, updated_at)
             VALUES (?1, 'match-notify-1', 'steam', 'Erangel', 'squad', '2026-03-30T12:00:00Z', 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [account_id],
            )
            .expect("insert match");

        let repo = NotificationTasksRepository::new(&connection, account_id);
        repo.upsert_pending(CreateNotificationTaskInput {
            match_id: "match-notify-1".to_string(),
            message_body: "body".to_string(),
            preview_match_time: "2026-03-30T12:00:00Z".to_string(),
            preview_placement: Some(2),
            preview_battle_summary: "张三 → 李四 12 分".to_string(),
        })
        .expect("create pending task");

        repo.mark_retry_scheduled(1, 1, "network error", "2026-03-30T12:00:10Z")
            .expect("retry 1");
        repo.mark_retry_scheduled(1, 2, "network error", "2026-03-30T12:00:30Z")
            .expect("retry 2");
        repo.mark_failed_manual(1, "auth error")
            .expect("mark manual failure");

        let failed = repo.get_failed_manual().expect("load failed tasks");
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].battle_summary, "张三 → 李四 12 分");
        assert_eq!(failed[0].send_status, "failed");
    }

    #[test]
    fn returns_due_pending_and_retrying_tasks() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        bootstrap_database(&connection).expect("bootstrap schema");

        let account_id: i64 = connection
            .query_row(
                "SELECT id FROM accounts WHERE is_active = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("select active account");

        connection
            .execute(
                "INSERT INTO matches (account_id, match_id, platform, map_name, game_mode, played_at, status, created_at, updated_at)
                 VALUES (?1, 'match-notify-a', 'steam', 'Erangel', 'squad', '2026-03-30T12:00:00Z', 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [account_id],
            )
            .expect("insert match a");
        connection
            .execute(
                "INSERT INTO matches (account_id, match_id, platform, map_name, game_mode, played_at, status, created_at, updated_at)
                 VALUES (?1, 'match-notify-b', 'steam', 'Erangel', 'squad', '2026-03-30T12:00:00Z', 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [account_id],
            )
            .expect("insert match b");
        connection
            .execute(
                "INSERT INTO matches (account_id, match_id, platform, map_name, game_mode, played_at, status, created_at, updated_at)
                 VALUES (?1, 'match-notify-c', 'steam', 'Erangel', 'squad', '2026-03-30T12:00:00Z', 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [account_id],
            )
            .expect("insert match c");

        let repo = NotificationTasksRepository::new(&connection, account_id);
        repo.upsert_pending(CreateNotificationTaskInput {
            match_id: "match-notify-a".to_string(),
            message_body: "body a".to_string(),
            preview_match_time: "2026-03-30T12:00:00Z".to_string(),
            preview_placement: Some(1),
            preview_battle_summary: "a".to_string(),
        })
        .expect("create pending a");

        repo.upsert_pending(CreateNotificationTaskInput {
            match_id: "match-notify-b".to_string(),
            message_body: "body b".to_string(),
            preview_match_time: "2026-03-30T12:00:00Z".to_string(),
            preview_placement: Some(2),
            preview_battle_summary: "b".to_string(),
        })
        .expect("create pending b");
        repo.mark_retry_scheduled(2, 1, "network", "2026-03-30T12:10:00Z")
            .expect("retry b");

        repo.upsert_pending(CreateNotificationTaskInput {
            match_id: "match-notify-c".to_string(),
            message_body: "body c".to_string(),
            preview_match_time: "2026-03-30T12:00:00Z".to_string(),
            preview_placement: Some(3),
            preview_battle_summary: "c".to_string(),
        })
        .expect("create pending c");
        repo.mark_retry_scheduled(3, 1, "network", "2026-03-30T12:30:00Z")
            .expect("retry c");

        let due = repo
            .get_due_retries("2026-03-30T12:15:00Z")
            .expect("get due tasks");

        assert_eq!(due.len(), 2);
        assert_eq!(due[0].match_id, "match-notify-a");
        assert_eq!(due[1].match_id, "match-notify-b");

        let pending = repo
            .get_pending_by_match("match-notify-b")
            .expect("get pending by match")
            .expect("pending task exists");
        assert_eq!(pending.status, "retrying");
    }
}
