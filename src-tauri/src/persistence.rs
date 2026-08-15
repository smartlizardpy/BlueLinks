use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    randomizer::normalize_title,
    types::{FinishPayload, GameMode, RunRecord, Settings, StatsSnapshot},
};

const SAVE_SCHEMA_VERSION: u32 = 1;
const HISTORY_LIMIT: usize = 100;

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct SaveData {
    schema_version: u32,
    settings: Settings,
    bests: HashMap<String, RunRecord>,
    history: Vec<RunRecord>,
    current_streak: u32,
    best_streak: u32,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            schema_version: SAVE_SCHEMA_VERSION,
            settings: Settings::default(),
            bests: HashMap::new(),
            history: Vec::new(),
            current_streak: 0,
            best_streak: 0,
        }
    }
}

pub struct Persistence {
    path: PathBuf,
}

impl Persistence {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn load(&self) -> SaveData {
        let Some(mut data) = fs::read(&self.path)
            .ok()
            .and_then(|value| serde_json::from_slice::<SaveData>(&value).ok())
        else {
            return SaveData::default();
        };
        if data.schema_version != SAVE_SCHEMA_VERSION {
            return SaveData::default();
        }
        data.settings = data.settings.sanitized();
        data.history.truncate(HISTORY_LIMIT);
        data
    }

    fn save(&self, data: &SaveData) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let temporary = self.path.with_extension("tmp");
        let value = serde_json::to_vec_pretty(data).map_err(|error| error.to_string())?;
        fs::write(&temporary, value).map_err(|error| error.to_string())?;
        fs::rename(temporary, &self.path).map_err(|error| error.to_string())
    }

    fn pair_key(start: &str, target: &str, mode: GameMode) -> String {
        format!(
            "{}\0{}\0{:?}",
            normalize_title(start),
            normalize_title(target),
            mode
        )
    }

    fn performance_is_better(payload: &FinishPayload, old: &RunRecord) -> bool {
        if payload.mode == GameMode::FewestClicks {
            payload.clicks < old.clicks
                || (payload.clicks == old.clicks && payload.duration_ms < old.duration_ms)
        } else {
            payload.duration_ms < old.duration_ms
                || (payload.duration_ms == old.duration_ms && payload.clicks < old.clicks)
        }
    }

    pub fn settings(&self) -> Settings {
        self.load().settings
    }

    pub fn update_settings(&self, settings: Settings) -> Result<Settings, String> {
        let mut data = self.load();
        data.settings = settings.sanitized();
        self.save(&data)?;
        Ok(data.settings)
    }

    pub fn stats(&self) -> StatsSnapshot {
        let data = self.load();
        StatsSnapshot {
            current_streak: data.current_streak,
            best_streak: data.best_streak,
            history: data.history,
        }
    }

    pub fn record(&self, payload: &mut FinishPayload) -> Result<bool, String> {
        let mut data = self.load();
        let key = Self::pair_key(&payload.start_title, &payload.target_title, payload.mode);
        let better = payload.success
            && data
                .bests
                .get(&key)
                .is_none_or(|old| Self::performance_is_better(payload, old));

        if payload.success {
            data.current_streak = data.current_streak.saturating_add(1);
            data.best_streak = data.best_streak.max(data.current_streak);
        } else {
            data.current_streak = 0;
        }
        payload.streak = data.current_streak;
        payload.best_streak = data.best_streak;

        let record = RunRecord {
            id: uuid::Uuid::new_v4().to_string(),
            start_title: payload.start_title.clone(),
            target_title: payload.target_title.clone(),
            duration_ms: payload.duration_ms,
            clicks: payload.clicks,
            route: payload.route.clone(),
            finished_at: format!("{:?}", std::time::SystemTime::now()),
            success: payload.success,
            outcome: Some(payload.outcome),
            mode: payload.mode,
            difficulty: payload.difficulty,
        };
        if better {
            data.bests.insert(key, record.clone());
        }
        if data.settings.save_run_history {
            data.history.insert(0, record);
            data.history.truncate(HISTORY_LIMIT);
        }
        self.save(&data)?;
        Ok(better)
    }

    pub fn clear_history(&self) -> Result<(), String> {
        let mut data = self.load();
        data.history.clear();
        self.save(&data)
    }

    pub fn reset_personal_bests(&self) -> Result<(), String> {
        let mut data = self.load();
        data.bests.clear();
        self.save(&data)
    }

    pub fn reset_streak(&self) -> Result<(), String> {
        let mut data = self.load();
        data.current_streak = 0;
        data.best_streak = 0;
        self.save(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RunOutcome;

    fn temporary_persistence() -> Persistence {
        Persistence::new(std::env::temp_dir().join(format!(
            "bluelink-persistence-{}.json",
            uuid::Uuid::new_v4()
        )))
    }

    fn payload(success: bool, clicks: u32, duration_ms: u64) -> FinishPayload {
        FinishPayload {
            start_title: "Minecraft".into(),
            target_title: "Sweden".into(),
            duration_ms,
            clicks,
            click_limit: 6,
            within_click_limit: success,
            route: vec!["Minecraft".into(), "Sweden".into()],
            is_personal_best: false,
            success,
            outcome: if success {
                RunOutcome::Success
            } else {
                RunOutcome::MaxClicksExceeded
            },
            mode: GameMode::MaxClicks,
            difficulty: 0.75,
            streak: 0,
            best_streak: 0,
            stage_count: 1,
        }
    }

    #[test]
    fn key_is_normalized() {
        assert_eq!(
            Persistence::pair_key("Roman_Empire", "Jazz", GameMode::Normal),
            Persistence::pair_key("roman empire", "jazz", GameMode::Normal)
        );
    }

    #[test]
    fn settings_survive_reload_and_are_sanitized() {
        let persistence = temporary_persistence();
        let mut settings = Settings {
            max_clicks: 500,
            ..Settings::default()
        };
        settings.default_mode = GameMode::TimeLimit;
        persistence.update_settings(settings).unwrap();
        assert_eq!(persistence.settings().default_mode, GameMode::TimeLimit);
        assert_eq!(persistence.settings().max_clicks, 50);
    }

    #[test]
    fn failed_runs_do_not_become_personal_bests_and_reset_streak() {
        let persistence = temporary_persistence();
        let mut success = payload(true, 3, 1_000);
        assert!(persistence.record(&mut success).unwrap());
        assert_eq!(success.streak, 1);
        let mut failed = payload(false, 7, 500);
        assert!(!persistence.record(&mut failed).unwrap());
        assert_eq!(failed.streak, 0);
    }

    #[test]
    fn corrupt_persistence_restores_defaults() {
        let persistence = temporary_persistence();
        if let Some(parent) = persistence.path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&persistence.path, b"not json").unwrap();
        assert_eq!(persistence.settings(), Settings::default());
    }

    #[test]
    fn history_streak_and_personal_best_survive_reload() {
        let persistence = temporary_persistence();
        let path = persistence.path.clone();
        let mut first = payload(true, 3, 1_000);
        assert!(persistence.record(&mut first).unwrap());
        let mut worse = payload(true, 4, 2_000);
        assert!(!persistence.record(&mut worse).unwrap());

        let reloaded = Persistence::new(path);
        let stats = reloaded.stats();
        assert_eq!(stats.history.len(), 2);
        assert_eq!(stats.current_streak, 2);
        assert_eq!(stats.best_streak, 2);
    }

    #[test]
    fn history_is_capped_and_unknown_schema_restores_defaults() {
        let persistence = temporary_persistence();
        for index in 0..105 {
            persistence
                .record(&mut payload(true, 3, 1_000 + index))
                .unwrap();
        }
        assert_eq!(persistence.stats().history.len(), HISTORY_LIMIT);
        fs::write(&persistence.path, br#"{"schema_version":999}"#).unwrap();
        assert_eq!(persistence.settings(), Settings::default());
        assert!(persistence.stats().history.is_empty());
    }
}
