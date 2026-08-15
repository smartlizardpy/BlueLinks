use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{
    randomizer::normalize_title,
    types::{ArticleRef, Challenge, GameMode, NavigationUpdate, RunOutcome, StageUpdate},
};

#[derive(Debug, Clone, PartialEq)]
pub enum TargetProgress {
    StageAdvanced(StageUpdate),
    Finished(RunOutcome),
}

pub struct RunSession {
    pub run_id: String,
    pub challenge: Challenge,
    pub current_title: String,
    pub clicks: u32,
    pub route: Vec<String>,
    pub started_at: Option<Instant>,
    pub transaction_open: bool,
    pub completed: bool,
    pub target_index: usize,
}

impl RunSession {
    pub fn note_navigation_attempt(&mut self, title: &str) {
        if self.started_at.is_some()
            && normalize_title(title) != normalize_title(&self.current_title)
            && !self.transaction_open
        {
            self.transaction_open = true;
        }
    }

    pub fn commit_loaded_article(&mut self, title: String) -> NavigationUpdate {
        if normalize_title(&title) != normalize_title(&self.current_title) {
            if self.transaction_open {
                self.clicks += 1;
            }
            self.current_title = title.clone();
            if normalize_title(self.route.last().map(String::as_str).unwrap_or(""))
                != normalize_title(&title)
            {
                self.route.push(title.clone());
            }
        }
        self.transaction_open = false;
        NavigationUpdate {
            current_title: title,
            clicks: self.clicks,
        }
    }

    pub fn active_target(&self) -> &ArticleRef {
        self.challenge
            .targets
            .get(self.target_index)
            .unwrap_or(&self.challenge.target)
    }

    pub fn stage_count(&self) -> u32 {
        self.challenge.targets.len().max(1) as u32
    }

    pub fn commit_target(&mut self) -> TargetProgress {
        let reached = self.active_target().clone();
        if self.transaction_open
            && normalize_title(&reached.title) != normalize_title(&self.current_title)
        {
            self.clicks += 1;
        }
        if normalize_title(self.route.last().map(String::as_str).unwrap_or(""))
            != normalize_title(&reached.title)
        {
            self.route.push(reached.title.clone());
        }
        self.current_title = reached.title;
        self.transaction_open = false;
        if self.challenge.mode == GameMode::Gauntlet
            && self.target_index + 1 < self.challenge.targets.len()
        {
            self.target_index += 1;
            return TargetProgress::StageAdvanced(StageUpdate {
                target: self.active_target().clone(),
                stage: self.target_index as u32 + 1,
                total: self.stage_count(),
            });
        }
        self.completed = true;
        if self.challenge.mode == GameMode::MaxClicks && self.clicks > self.challenge.click_limit {
            TargetProgress::Finished(RunOutcome::MaxClicksExceeded)
        } else {
            TargetProgress::Finished(RunOutcome::Success)
        }
    }

    pub fn exceeded_click_limit(&self) -> bool {
        self.challenge.mode == GameMode::MaxClicks && self.clicks > self.challenge.click_limit
    }
}

#[derive(Clone, Default)]
pub struct GameManager {
    pub inner: Arc<Mutex<Option<RunSession>>>,
    pub last_challenge: Arc<Mutex<Option<Challenge>>>,
}

impl GameManager {
    pub fn prepare(&self, challenge: Challenge) -> Result<String, String> {
        let id = uuid::Uuid::new_v4().to_string();
        *self
            .last_challenge
            .lock()
            .map_err(|_| "Game state lock was poisoned")? = Some(challenge.clone());
        *self
            .inner
            .lock()
            .map_err(|_| "Game state lock was poisoned")? = Some(RunSession {
            run_id: id.clone(),
            current_title: challenge.start.title.clone(),
            route: vec![challenge.start.title.clone()],
            challenge,
            clicks: 0,
            started_at: None,
            transaction_open: false,
            completed: false,
            target_index: 0,
        });
        Ok(id)
    }

    pub fn cancel(&self) {
        if let Ok(mut run) = self.inner.lock() {
            *run = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArticleRef, GameMode};

    fn challenge(mode: GameMode, click_limit: u32) -> Challenge {
        Challenge {
            start: ArticleRef {
                id: 1,
                title: "Minecraft".into(),
            },
            target: ArticleRef {
                id: 2,
                title: "Sweden".into(),
            },
            click_limit,
            time_limit_seconds: None,
            difficulty: 0.75,
            mode,
            targets: vec![ArticleRef {
                id: 2,
                title: "Sweden".into(),
            }],
        }
    }

    fn session(mode: GameMode, click_limit: u32) -> RunSession {
        RunSession {
            run_id: "test".into(),
            challenge: challenge(mode, click_limit),
            current_title: "Minecraft".into(),
            clicks: 0,
            route: vec!["Minecraft".into()],
            started_at: Some(Instant::now()),
            transaction_open: false,
            completed: false,
            target_index: 0,
        }
    }

    #[test]
    fn normal_navigation_counts_once() {
        let mut run = session(GameMode::Normal, 6);
        run.note_navigation_attempt("Video game");
        run.note_navigation_attempt("Video game");
        run.commit_loaded_article("Video game".into());
        assert_eq!(run.clicks, 1);
        assert_eq!(run.route, ["Minecraft", "Video game"]);
    }

    #[test]
    fn reload_and_fragment_do_not_count() {
        let mut run = session(GameMode::Normal, 6);
        run.note_navigation_attempt("Minecraft");
        run.commit_loaded_article("Minecraft".into());
        assert_eq!(run.clicks, 0);
        assert_eq!(run.route, ["Minecraft"]);
    }

    #[test]
    fn redirect_chain_to_target_counts_once() {
        let mut run = session(GameMode::Normal, 6);
        run.note_navigation_attempt("Kingdom of Sweden");
        run.note_navigation_attempt("Sweden");
        assert_eq!(
            run.commit_target(),
            TargetProgress::Finished(RunOutcome::Success)
        );
        assert_eq!(run.clicks, 1);
        assert_eq!(run.route, ["Minecraft", "Sweden"]);
    }

    #[test]
    fn max_clicks_success_and_failure_are_distinct() {
        let mut success = session(GameMode::MaxClicks, 1);
        success.note_navigation_attempt("Sweden");
        assert_eq!(
            success.commit_target(),
            TargetProgress::Finished(RunOutcome::Success)
        );
        let mut failure = session(GameMode::MaxClicks, 0);
        failure.note_navigation_attempt("Sweden");
        assert_eq!(
            failure.commit_target(),
            TargetProgress::Finished(RunOutcome::MaxClicksExceeded)
        );
    }

    #[test]
    fn gauntlet_advances_through_five_targets_before_finishing() {
        let mut run = session(GameMode::Gauntlet, 10);
        run.challenge.targets = (2..=6)
            .map(|id| ArticleRef {
                id,
                title: format!("Target {id}"),
            })
            .collect();
        run.challenge.target = run.challenge.targets[0].clone();
        for stage in 2..=5 {
            run.note_navigation_attempt(&format!("Target {}", stage));
            assert_eq!(
                run.commit_target(),
                TargetProgress::StageAdvanced(StageUpdate {
                    target: ArticleRef {
                        id: stage + 1,
                        title: format!("Target {}", stage + 1)
                    },
                    stage,
                    total: 5,
                })
            );
        }
        run.note_navigation_attempt("Target 6");
        assert_eq!(
            run.commit_target(),
            TargetProgress::Finished(RunOutcome::Success)
        );
        assert!(run.completed);
        assert_eq!(run.clicks, 5);
    }
}
