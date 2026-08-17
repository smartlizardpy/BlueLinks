use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum GameMode {
    #[default]
    Normal,
    MaxClicks,
    TimeLimit,
    FewestClicks,
    Speedrun,
    Evil,
    TwoPlayer,
    Gauntlet,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum DifficultyPreset {
    #[default]
    Normal,
    Evil,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub default_mode: GameMode,
    pub default_difficulty: DifficultyPreset,
    pub max_clicks: u32,
    pub time_limit_seconds: u32,
    pub countdown: bool,
    pub show_timer: bool,
    pub show_click_count: bool,
    pub confirm_before_abandoning: bool,
    pub scramble_animation: bool,
    pub reduced_motion: bool,
    pub save_run_history: bool,
    pub automatically_check_for_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_mode: GameMode::Normal,
            default_difficulty: DifficultyPreset::Normal,
            max_clicks: 6,
            time_limit_seconds: 60,
            countdown: true,
            show_timer: true,
            show_click_count: true,
            confirm_before_abandoning: true,
            scramble_animation: true,
            reduced_motion: false,
            save_run_history: true,
            automatically_check_for_updates: true,
        }
    }
}

impl Settings {
    pub fn sanitized(mut self) -> Self {
        self.max_clicks = self.max_clicks.clamp(1, 50);
        if !matches!(self.time_limit_seconds, 0 | 30 | 60 | 120 | 300) {
            self.time_limit_seconds = 60;
        }
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArticleRef {
    pub id: u32,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub start: ArticleRef,
    pub target: ArticleRef,
    pub click_limit: u32,
    pub time_limit_seconds: Option<u32>,
    pub difficulty: f32,
    pub mode: GameMode,
    #[serde(default)]
    pub targets: Vec<ArticleRef>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RunOutcome {
    Success,
    MaxClicksExceeded,
    TimeExpired,
    ConnectionLost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishPayload {
    pub start_title: String,
    pub target_title: String,
    pub duration_ms: u64,
    pub clicks: u32,
    pub click_limit: u32,
    pub within_click_limit: bool,
    pub route: Vec<String>,
    pub is_personal_best: bool,
    pub success: bool,
    pub outcome: RunOutcome,
    pub mode: GameMode,
    pub difficulty: f32,
    pub streak: u32,
    pub best_streak: u32,
    pub stage_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StageUpdate {
    pub target: ArticleRef,
    pub stage: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationUpdate {
    pub current_title: String,
    pub clicks: u32,
}

#[derive(Debug, Clone)]
pub struct ArticleMeta {
    pub id: u32,
    pub title: String,
    pub normalized_title: String,
    pub is_redirect: bool,
    pub is_disambiguation: bool,
    pub in_degree: u32,
    pub out_degree: u32,
    pub topic_mask: u32,
    pub community_id: u32,
    pub graph_signature: [u32; 4],
    /// How many times more likely this article is to be drawn than a plain one.
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RunRecord {
    pub id: String,
    pub start_title: String,
    pub target_title: String,
    pub duration_ms: u64,
    pub clicks: u32,
    pub route: Vec<String>,
    pub finished_at: String,
    pub success: bool,
    pub outcome: Option<RunOutcome>,
    pub mode: GameMode,
    pub difficulty: f32,
}

impl Default for RunRecord {
    fn default() -> Self {
        Self {
            id: String::new(),
            start_title: String::new(),
            target_title: String::new(),
            duration_ms: 0,
            clicks: 0,
            route: Vec::new(),
            finished_at: String::new(),
            success: true,
            outcome: None,
            mode: GameMode::Normal,
            difficulty: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatsSnapshot {
    pub current_streak: u32,
    pub best_streak: u32,
    pub history: Vec<RunRecord>,
}
