use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const GAME_PROCESS_COOLDOWN: Duration = Duration::from_secs(40 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameProcessState {
    NotRunning,
    Running,
    CooldownPolling,
}

#[derive(Debug, Clone)]
pub struct GameProcessRuntime {
    pub state: GameProcessState,
    pub last_seen_running_at: Option<SystemTime>,
    pub cooldown_started_at: Option<SystemTime>,
    pub last_process_check_at: Option<SystemTime>,
    pub last_recent_match_check_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameProcessStatusSnapshot {
    pub state: GameProcessState,
    pub last_seen_running_at_ms: Option<u64>,
    pub cooldown_started_at_ms: Option<u64>,
    pub last_process_check_at_ms: Option<u64>,
    pub last_recent_match_check_at_ms: Option<u64>,
}

impl Default for GameProcessRuntime {
    fn default() -> Self {
        Self {
            state: GameProcessState::NotRunning,
            last_seen_running_at: None,
            cooldown_started_at: None,
            last_process_check_at: None,
            last_recent_match_check_at: None,
        }
    }
}

impl GameProcessRuntime {
    pub fn update_process_observation(&mut self, process_running: bool, now: SystemTime) {
        self.update_process_observation_with_cooldown(process_running, now, GAME_PROCESS_COOLDOWN);
    }

    pub fn update_process_observation_with_cooldown(
        &mut self,
        process_running: bool,
        now: SystemTime,
        cooldown: Duration,
    ) {
        self.last_process_check_at = Some(now);

        if process_running {
            self.state = GameProcessState::Running;
            self.last_seen_running_at = Some(now);
            self.cooldown_started_at = None;
            return;
        }

        match self.state {
            GameProcessState::Running => {
                self.state = GameProcessState::CooldownPolling;
                self.cooldown_started_at = Some(now);
            }
            GameProcessState::CooldownPolling => {
                if let Some(cooldown_started_at) = self.cooldown_started_at {
                    if let Some(elapsed) = elapsed_since(cooldown_started_at, now) {
                        if elapsed >= cooldown {
                            self.state = GameProcessState::NotRunning;
                            self.cooldown_started_at = None;
                        }
                    }
                } else {
                    self.cooldown_started_at = Some(now);
                }
            }
            GameProcessState::NotRunning => {
                self.cooldown_started_at = None;
            }
        }
    }

    pub fn should_trigger_recent_match_check(&self, now: SystemTime, cadence: Duration) -> bool {
        match self.last_recent_match_check_at {
            Some(last_check) => {
                elapsed_since(last_check, now).is_some_and(|elapsed| elapsed >= cadence)
            }
            None => true,
        }
    }

    pub fn mark_recent_match_check(&mut self, now: SystemTime) {
        self.last_recent_match_check_at = Some(now);
    }

    pub fn snapshot(&self) -> GameProcessStatusSnapshot {
        GameProcessStatusSnapshot {
            state: self.state,
            last_seen_running_at_ms: to_unix_ms(self.last_seen_running_at),
            cooldown_started_at_ms: to_unix_ms(self.cooldown_started_at),
            last_process_check_at_ms: to_unix_ms(self.last_process_check_at),
            last_recent_match_check_at_ms: to_unix_ms(self.last_recent_match_check_at),
        }
    }
}

fn elapsed_since(start: SystemTime, end: SystemTime) -> Option<Duration> {
    end.duration_since(start).ok()
}

fn to_unix_ms(value: Option<SystemTime>) -> Option<u64> {
    value.and_then(|time| {
        time.duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
    })
}

#[cfg(test)]
mod tests {
    use super::{GameProcessRuntime, GameProcessState};
    use std::time::{Duration, SystemTime};

    #[test]
    fn transitions_not_running_to_running_when_process_detected() {
        let base_time = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let mut runtime = GameProcessRuntime::default();

        runtime.update_process_observation(true, base_time);

        assert_eq!(runtime.state, GameProcessState::Running);
        assert_eq!(runtime.last_seen_running_at, Some(base_time));
        assert_eq!(runtime.cooldown_started_at, None);
    }

    #[test]
    fn transitions_running_to_cooldown_when_process_disappears() {
        let base_time = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let mut runtime = GameProcessRuntime::default();
        runtime.update_process_observation(true, base_time);

        let after_exit = base_time + Duration::from_secs(5);
        runtime.update_process_observation(false, after_exit);

        assert_eq!(runtime.state, GameProcessState::CooldownPolling);
        assert_eq!(runtime.cooldown_started_at, Some(after_exit));
    }

    #[test]
    fn transitions_cooldown_to_running_when_process_reappears() {
        let base_time = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let mut runtime = GameProcessRuntime::default();
        runtime.update_process_observation(true, base_time);
        runtime.update_process_observation(false, base_time + Duration::from_secs(5));

        let comeback_time = base_time + Duration::from_secs(15);
        runtime.update_process_observation(true, comeback_time);

        assert_eq!(runtime.state, GameProcessState::Running);
        assert_eq!(runtime.last_seen_running_at, Some(comeback_time));
        assert_eq!(runtime.cooldown_started_at, None);
    }

    #[test]
    fn transitions_cooldown_to_not_running_after_cooldown_expiry() {
        let base_time = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let mut runtime = GameProcessRuntime::default();
        runtime.update_process_observation(true, base_time);

        let cooldown_started = base_time + Duration::from_secs(5);
        runtime.update_process_observation_with_cooldown(
            false,
            cooldown_started,
            Duration::from_secs(10),
        );
        assert_eq!(runtime.state, GameProcessState::CooldownPolling);

        let still_cooling = cooldown_started + Duration::from_secs(9);
        runtime.update_process_observation_with_cooldown(
            false,
            still_cooling,
            Duration::from_secs(10),
        );
        assert_eq!(runtime.state, GameProcessState::CooldownPolling);

        let cooldown_expired = cooldown_started + Duration::from_secs(10);
        runtime.update_process_observation_with_cooldown(
            false,
            cooldown_expired,
            Duration::from_secs(10),
        );
        assert_eq!(runtime.state, GameProcessState::NotRunning);
        assert_eq!(runtime.cooldown_started_at, None);
    }
}
