use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use rusqlite::Connection;

use crate::{
    platform::process,
    runtime::game_state::{GameProcessRuntime, GameProcessState},
    services::{
        polling::{load_polling_config, PollingConfig, PollingMode},
        sync::{sync_recent_match_with_retry, SyncRuntimeStatus},
    },
};

#[derive(Debug, Clone, Copy)]
struct SchedulerTickResult {
    sleep_for: Duration,
    should_trigger_recent_match_check: bool,
}

pub fn start_background_scheduler(runtime_state: Arc<Mutex<GameProcessRuntime>>) {
    start_background_scheduler_with_sync(runtime_state, None, None);
}

pub fn start_background_scheduler_with_sync(
    runtime_state: Arc<Mutex<GameProcessRuntime>>,
    db: Option<Arc<Mutex<Connection>>>,
    sync_runtime_status: Option<Arc<Mutex<SyncRuntimeStatus>>>,
) {
    thread::spawn(move || loop {
        let now = SystemTime::now();
        let is_running = process::is_pubg_running();
        let polling_config = load_runtime_polling_config(db.as_ref());

        let (sleep_for, should_trigger_recent_match_check) = {
            if let Ok(mut runtime) = runtime_state.lock() {
                let tick_result = tick_once(&mut runtime, is_running, now, &polling_config);

                (
                    tick_result.sleep_for,
                    tick_result.should_trigger_recent_match_check,
                )
            } else {
                (polling_config.not_running_process_check_interval, false)
            }
        };

        if should_trigger_recent_match_check && polling_config.auto_recent_match_enabled {
            if let (Some(db_state), Some(sync_status)) = (&db, &sync_runtime_status) {
                if let Ok(connection) = db_state.lock() {
                    sync_recent_match_with_retry(
                        &connection,
                        sync_status,
                        polling_config.recent_match_retry_limit,
                        polling_config.recent_match_retry_delay,
                    );
                }
            }
        }

        if let Some(db_state) = &db {
            if let Ok(connection) = db_state.lock() {
                let _ = crate::services::notifications::process_due_notifications(&connection);
            }
        }

        thread::sleep(sleep_for);
    });
}

fn tick_once(
    runtime: &mut GameProcessRuntime,
    is_running: bool,
    now: SystemTime,
    config: &PollingConfig,
) -> SchedulerTickResult {
    match config.polling_mode {
        PollingMode::Game => tick_game_mode(runtime, is_running, now, config),
        PollingMode::Manual => tick_manual_mode(runtime, is_running, now, config),
        PollingMode::Auto => tick_auto_mode(runtime, now, config),
    }
}

fn tick_game_mode(
    runtime: &mut GameProcessRuntime,
    is_running: bool,
    now: SystemTime,
    config: &PollingConfig,
) -> SchedulerTickResult {
    runtime.update_process_observation_with_cooldown(is_running, now, config.cooldown_window);

    let should_trigger_recent_match_check = should_check_recent_matches(runtime.state)
        && runtime.should_trigger_recent_match_check(
            now,
            recent_match_check_cadence(runtime.state, config),
        );

    if should_trigger_recent_match_check {
        runtime.mark_recent_match_check(now);
    }

    SchedulerTickResult {
        sleep_for: process_check_interval(runtime.state, config),
        should_trigger_recent_match_check,
    }
}

fn tick_manual_mode(
    runtime: &mut GameProcessRuntime,
    is_running: bool,
    now: SystemTime,
    config: &PollingConfig,
) -> SchedulerTickResult {
    runtime.update_process_observation_with_cooldown(is_running, now, config.cooldown_window);

    SchedulerTickResult {
        sleep_for: process_check_interval(runtime.state, config),
        should_trigger_recent_match_check: false,
    }
}

fn tick_auto_mode(
    runtime: &mut GameProcessRuntime,
    now: SystemTime,
    config: &PollingConfig,
) -> SchedulerTickResult {
    let should_trigger_recent_match_check =
        runtime.should_trigger_recent_match_check(now, config.running_recent_match_interval);

    if should_trigger_recent_match_check {
        runtime.mark_recent_match_check(now);
    }

    SchedulerTickResult {
        sleep_for: config.running_recent_match_interval,
        should_trigger_recent_match_check,
    }
}

fn process_check_interval(state: GameProcessState, config: &PollingConfig) -> Duration {
    match state {
        GameProcessState::Running => config.running_process_check_interval,
        GameProcessState::CooldownPolling => config.cooldown_polling_interval,
        GameProcessState::NotRunning => config.not_running_process_check_interval,
    }
}

fn recent_match_check_cadence(state: GameProcessState, config: &PollingConfig) -> Duration {
    match state {
        GameProcessState::Running => config.running_recent_match_interval,
        GameProcessState::CooldownPolling => config.cooldown_polling_interval,
        GameProcessState::NotRunning => config.not_running_process_check_interval,
    }
}

fn should_check_recent_matches(state: GameProcessState) -> bool {
    !matches!(state, GameProcessState::NotRunning)
}

fn load_runtime_polling_config(db: Option<&Arc<Mutex<Connection>>>) -> PollingConfig {
    let defaults = PollingConfig::default();

    let Some(db_state) = db else {
        return defaults;
    };

    let Ok(connection) = db_state.lock() else {
        return defaults;
    };

    load_polling_config(&connection).unwrap_or(defaults)
}

#[cfg(test)]
mod tests {
    use super::{
        process_check_interval, recent_match_check_cadence, should_check_recent_matches, tick_once,
    };
    use crate::runtime::game_state::{GameProcessRuntime, GameProcessState};
    use crate::services::polling::{PollingConfig, PollingMode};
    use std::time::{Duration, SystemTime};

    #[test]
    fn uses_expected_cadence_per_state() {
        let config = PollingConfig::default();
        assert_eq!(
            process_check_interval(GameProcessState::Running, &config),
            Duration::from_secs(5)
        );
        assert_eq!(
            process_check_interval(GameProcessState::CooldownPolling, &config),
            Duration::from_secs(120)
        );
        assert_eq!(
            process_check_interval(GameProcessState::NotRunning, &config),
            Duration::from_secs(30)
        );

        assert_eq!(
            recent_match_check_cadence(GameProcessState::Running, &config),
            Duration::from_secs(30)
        );
        assert_eq!(
            recent_match_check_cadence(GameProcessState::CooldownPolling, &config),
            Duration::from_secs(120)
        );
        assert_eq!(
            recent_match_check_cadence(GameProcessState::NotRunning, &config),
            Duration::from_secs(30)
        );

        assert!(should_check_recent_matches(GameProcessState::Running));
        assert!(should_check_recent_matches(
            GameProcessState::CooldownPolling
        ));
        assert!(!should_check_recent_matches(GameProcessState::NotRunning));
    }

    #[test]
    fn triggers_recent_match_check_when_no_previous_check_exists() {
        let config = PollingConfig::default();
        let mut runtime = GameProcessRuntime::default();
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);

        let warm_up = tick_once(&mut runtime, true, now, &config);
        assert_eq!(runtime.state, GameProcessState::Running);
        assert!(warm_up.should_trigger_recent_match_check);

        let next_check = now + Duration::from_secs(31);
        let result = tick_once(&mut runtime, true, next_check, &config);

        assert!(result.should_trigger_recent_match_check);
        assert_eq!(runtime.last_recent_match_check_at, Some(next_check));
    }

    #[test]
    fn does_not_trigger_recent_match_check_when_game_is_not_running() {
        let config = PollingConfig::default();
        let mut runtime = GameProcessRuntime::default();
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);

        let result = tick_once(&mut runtime, false, now, &config);

        assert_eq!(runtime.state, GameProcessState::NotRunning);
        assert!(!result.should_trigger_recent_match_check);
        assert_eq!(runtime.last_recent_match_check_at, None);
    }

    #[test]
    fn manual_mode_never_triggers_recent_match_check() {
        let mut config = PollingConfig::default();
        config.polling_mode = PollingMode::Manual;

        let mut runtime = GameProcessRuntime::default();
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let result = tick_once(&mut runtime, true, now, &config);

        assert!(!result.should_trigger_recent_match_check);
    }

    #[test]
    fn auto_mode_triggers_even_when_process_is_not_running() {
        let mut config = PollingConfig::default();
        config.polling_mode = PollingMode::Auto;

        let mut runtime = GameProcessRuntime::default();
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let result = tick_once(&mut runtime, false, now, &config);

        assert!(result.should_trigger_recent_match_check);
        assert_eq!(result.sleep_for, config.running_recent_match_interval);
    }

    #[test]
    fn keeps_cooldown_state_before_expiry_and_reverts_after_expiry() {
        let config = PollingConfig::default();
        let base_time = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let mut runtime = GameProcessRuntime::default();

        tick_once(&mut runtime, true, base_time, &config);
        assert_eq!(runtime.state, GameProcessState::Running);

        let cooldown_started = base_time + Duration::from_secs(5);
        tick_once(&mut runtime, false, cooldown_started, &config);
        assert_eq!(runtime.state, GameProcessState::CooldownPolling);

        let before_expiry = cooldown_started + Duration::from_secs(39 * 60);
        tick_once(&mut runtime, false, before_expiry, &config);
        assert_eq!(runtime.state, GameProcessState::CooldownPolling);

        let after_expiry = cooldown_started + Duration::from_secs(40 * 60);
        let result = tick_once(&mut runtime, false, after_expiry, &config);
        assert_eq!(runtime.state, GameProcessState::NotRunning);
        assert_eq!(result.sleep_for, Duration::from_secs(30));
    }
}
