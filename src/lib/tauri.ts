import { invoke } from "@tauri-apps/api/core";
import type { Challenge, Settings, StatsSnapshot } from "../state/game";

export const commands = {
  generateChallenge: (previous?: Challenge) => invoke<Challenge>("generate_challenge", { previous }),
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (settings: Settings) => invoke<Settings>("update_settings", { settings }),
  getStats: () => invoke<StatsSnapshot>("get_stats"),
  clearRunHistory: () => invoke<void>("clear_run_history"),
  resetPersonalBests: () => invoke<void>("reset_personal_bests"),
  resetStreak: () => invoke<void>("reset_streak"),
  appVersion: () => invoke<string>("app_version"),
  datasetVersion: () => invoke<string>("dataset_version"),
  startRun: (challenge: Challenge) => invoke<string>("start_run", { challenge }),
  replayRun: () => invoke<string>("replay_run"),
  cancelRun: () => invoke<void>("cancel_run"),
};
