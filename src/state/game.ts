export type AppState =
  | "HOME_EMPTY" | "HOME_RANDOMIZING" | "HOME_READY"
  | "GAME_LOADING" | "GAME_RUNNING" | "GAME_FINISHING"
  | "COUNTDOWN" | "RESULT" | "PLAYER_HANDOFF" | "MULTIPLAYER_RESULT"
  | "SETTINGS" | "HISTORY" | "ERROR";

export type GameMode = "normal" | "maxClicks" | "timeLimit" | "fewestClicks" | "speedrun" | "evil" | "twoPlayer" | "gauntlet";
export type DifficultyPreset = "normal" | "evil";
export type RunOutcome = "success" | "maxClicksExceeded" | "timeExpired" | "connectionLost";

export interface Settings {
  defaultMode: GameMode;
  defaultDifficulty: DifficultyPreset;
  maxClicks: number;
  timeLimitSeconds: number;
  countdown: boolean;
  showTimer: boolean;
  showClickCount: boolean;
  confirmBeforeAbandoning: boolean;
  scrambleAnimation: boolean;
  reducedMotion: boolean;
  saveRunHistory: boolean;
  automaticallyCheckForUpdates: boolean;
}

export interface ArticleRef { id: number; title: string }
export interface Challenge {
  start: ArticleRef;
  target: ArticleRef;
  clickLimit: number;
  timeLimitSeconds?: number;
  difficulty: number;
  mode: GameMode;
  targets: ArticleRef[];
}
export interface NavigationUpdate { currentTitle: string; clicks: number }
export interface FinishPayload {
  startTitle: string;
  targetTitle: string;
  durationMs: number;
  clicks: number;
  clickLimit: number;
  withinClickLimit: boolean;
  route: string[];
  isPersonalBest: boolean;
  success: boolean;
  outcome: RunOutcome;
  mode: GameMode;
  difficulty: number;
  streak: number;
  bestStreak: number;
  stageCount: number;
}
export interface StageUpdate { target: ArticleRef; stage: number; total: number }
export interface RunRecord {
  id: string;
  startTitle: string;
  targetTitle: string;
  durationMs: number;
  clicks: number;
  route: string[];
  finishedAt: string;
  success: boolean;
  outcome?: RunOutcome;
  mode: GameMode;
  difficulty: number;
}
export interface StatsSnapshot { currentStreak: number; bestStreak: number; history: RunRecord[] }
export interface GameError { kind: "initial-load" | "connection" | "dataset" | "runtime"; message: string }

export const MODE_LABELS: Record<GameMode, string> = {
  normal: "NORMAL",
  maxClicks: "MAX CLICKS",
  timeLimit: "TIME LIMIT",
  fewestClicks: "FEWEST CLICKS",
  speedrun: "SPEEDRUN",
  evil: "EVIL",
  twoPlayer: "2 PLAYER",
  gauntlet: "GAUNTLET",
};
