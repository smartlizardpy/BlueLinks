import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { UpdatePrompt, type UpdatePhase } from "./components/UpdatePrompt";
import { commands } from "./lib/tauri";
import { ErrorScreen } from "./screens/ErrorScreen";
import { GameHeader } from "./screens/GameHeader";
import { HistoryScreen } from "./screens/HistoryScreen";
import { ResultScreen } from "./screens/ResultScreen";
import { SettingsScreen } from "./screens/SettingsScreen";
import { StartScreen } from "./screens/StartScreen";
import { MultiplayerResult, PlayerHandoff } from "./screens/MultiplayerScreens";
import type { AppState, Challenge, FinishPayload, GameError, GameMode, NavigationUpdate, Settings, StageUpdate, StatsSnapshot } from "./state/game";

const DEFAULT_SETTINGS: Settings = {
  defaultMode: "normal",
  defaultDifficulty: "normal",
  maxClicks: 6,
  timeLimitSeconds: 60,
  countdown: true,
  showTimer: true,
  showClickCount: true,
  confirmBeforeAbandoning: true,
  scrambleAnimation: true,
  reducedMotion: false,
  saveRunHistory: true,
  automaticallyCheckForUpdates: true,
};
const EMPTY_STATS: StatsSnapshot = { currentStreak: 0, bestStreak: 0, history: [] };
const wait = (milliseconds: number) => new Promise(resolve => window.setTimeout(resolve, milliseconds));

export default function App() {
  const [state, setState] = useState<AppState>("HOME_EMPTY");
  const [challenge, setChallenge] = useState<Challenge | null>(null);
  const [clicks, setClicks] = useState(0);
  const [countdown, setCountdown] = useState(0);
  const [result, setResult] = useState<FinishPayload | null>(null);
  const [error, setError] = useState<GameError | null>(null);
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [stats, setStats] = useState<StatsSnapshot>(EMPTY_STATS);
  const [appVersion, setAppVersion] = useState("1.0.0");
  const [datasetVersion, setDatasetVersion] = useState("unknown");
  const [stage, setStage] = useState(1);
  const [playerOne, setPlayerOne] = useState<FinishPayload | null>(null);
  const [playerTwo, setPlayerTwo] = useState<FinishPayload | null>(null);
  const playerOneRef = useRef<FinishPayload | null>(null);
  const updateRef = useRef<Update | null>(null);
  const autoCheckStarted = useRef(false);
  const [settingsLoaded, setSettingsLoaded] = useState(false);
  const [updatePhase, setUpdatePhase] = useState<UpdatePhase>("hidden");
  const [confirmEnd, setConfirmEnd] = useState(false);
  const [updateVersion, setUpdateVersion] = useState("");
  const [updateProgress, setUpdateProgress] = useState(0);
  const [updateFeedback, setUpdateFeedback] = useState("");

  useEffect(() => {
    void Promise.all([commands.getSettings(), commands.appVersion(), commands.datasetVersion()]).then(([stored, version, dataset]) => {
      setSettings(stored); setAppVersion(version); setDatasetVersion(dataset); setSettingsLoaded(true);
    }).catch(() => setSettingsLoaded(true));
  }, []);

  useEffect(() => {
    document.documentElement.classList.toggle("reduce-motion", settings.reducedMotion);
    document.documentElement.dataset.scramble = settings.scrambleAnimation ? "on" : "off";
  }, [settings.reducedMotion, settings.scrambleAnimation]);

  useEffect(() => {
    const unlisteners = [
      listen("game:run_started", () => setState("GAME_RUNNING")),
      listen<NavigationUpdate>("game:navigation", event => setClicks(event.payload.clicks)),
      listen<StageUpdate>("game:stage", event => {
        setStage(event.payload.stage);
        setChallenge(current => current ? { ...current, start: current.target, target: event.payload.target } : current);
      }),
      listen<FinishPayload>("game:finish", event => {
        if (event.payload.mode === "twoPlayer") {
          if (!playerOneRef.current) {
            playerOneRef.current = event.payload; setPlayerOne(event.payload); setState("PLAYER_HANDOFF");
          } else {
            setPlayerTwo(event.payload); setState("MULTIPLAYER_RESULT");
          }
          return;
        }
        setResult(event.payload); setState("GAME_FINISHING");
        requestAnimationFrame(() => setState("RESULT"));
      }),
      listen<GameError>("game:error", event => { setError(event.payload); setState("ERROR"); }),
    ];
    return () => { void Promise.all(unlisteners).then(items => items.forEach(unlisten => unlisten())); };
  }, []);

  const saveSettings = useCallback((next: Settings) => {
    setSettings(next);
    void commands.updateSettings(next).then(setSettings).catch(() => undefined);
  }, []);

  const checkForUpdate = useCallback(async (manual = false) => {
    if (manual) setUpdateFeedback("CHECKING…");
    try {
      const found = await check({ timeout: 12_000 });
      if (!found) { if (manual) setUpdateFeedback("YOU'RE UP TO DATE."); return; }
      if (updateRef.current && updateRef.current !== found) void updateRef.current.close();
      updateRef.current = found; setUpdateVersion(found.version); setUpdatePhase("available");
      if (manual) setUpdateFeedback(`VERSION ${found.version} AVAILABLE.`);
    } catch {
      if (manual) setUpdateFeedback("UNABLE TO CHECK RIGHT NOW.");
    }
  }, []);

  useEffect(() => {
    if (!settingsLoaded || !settings.automaticallyCheckForUpdates || autoCheckStarted.current) return;
    autoCheckStarted.current = true;
    const timer = window.setTimeout(() => void checkForUpdate(false), 1800);
    return () => window.clearTimeout(timer);
  }, [checkForUpdate, settings.automaticallyCheckForUpdates, settingsLoaded]);

  const downloadUpdate = useCallback(async () => {
    const update = updateRef.current; if (!update) return;
    setUpdatePhase("downloading"); setUpdateProgress(0);
    let downloaded = 0; let total = 0;
    try {
      await update.download(event => {
        if (event.event === "Started") total = event.data.contentLength ?? 0;
        if (event.event === "Progress") downloaded += event.data.chunkLength;
        if (event.event === "Progress" && total > 0) setUpdateProgress(Math.min(99, Math.round(downloaded / total * 100)));
        if (event.event === "Finished") setUpdateProgress(100);
      });
      setUpdatePhase("ready");
    } catch { setUpdatePhase("failed"); }
  }, []);

  const installUpdate = useCallback(async () => {
    try { await updateRef.current?.install(); await relaunch(); }
    catch { setUpdatePhase("failed"); }
  }, []);

  const randomize = useCallback(async () => {
    if (state === "HOME_RANDOMIZING") return;
    setState("HOME_RANDOMIZING");
    try { const next = await commands.generateChallenge(challenge ?? undefined); setChallenge(next); setState("HOME_READY"); }
    catch (cause) { setError({ kind: "dataset", message: String(cause) }); setState("ERROR"); }
  }, [challenge, state]);

  const launchRun = useCallback(async (selected: Challenge) => {
    setClicks(0); setStage(1); setConfirmEnd(false); setState("GAME_LOADING");
    try { await commands.startRun(selected); }
    catch (cause) { setError({ kind: "initial-load", message: String(cause) }); setState("ERROR"); }
  }, []);

  const start = useCallback(async () => {
    if (!challenge || state !== "HOME_READY") return;
    if (settings.countdown) {
      setState("COUNTDOWN");
      for (const value of [3, 2, 1]) { setCountdown(value); await wait(650); }
      setCountdown(0);
    }
    await launchRun(challenge);
  }, [challenge, launchRun, settings.countdown, state]);

  const again = useCallback(async () => {
    if (!challenge || state !== "RESULT") return;
    setClicks(0); setState("GAME_LOADING");
    try { await commands.replayRun(); }
    catch (cause) { setError({ kind: "initial-load", message: String(cause) }); setState("ERROR"); }
  }, [challenge, state]);

  const startPlayerTwo = useCallback(async () => {
    setClicks(0); setStage(1); setConfirmEnd(false); setState("GAME_LOADING");
    try { await commands.replayRun(); }
    catch (cause) { setError({ kind: "initial-load", message: String(cause) }); setState("ERROR"); }
  }, []);

  const replayMultiplayer = useCallback(async () => {
    playerOneRef.current = null; setPlayerOne(null); setPlayerTwo(null);
    if (challenge) await launchRun(challenge);
  }, [challenge, launchRun]);

  const newRun = useCallback(() => {
    void commands.cancelRun(); playerOneRef.current = null; setPlayerOne(null); setPlayerTwo(null); setChallenge(null); setResult(null); setError(null); setClicks(0); setStage(1); setState("HOME_EMPTY");
  }, []);

  const abandonRun = useCallback(async () => {
    setConfirmEnd(false);
    await commands.cancelRun(); setChallenge(null); setResult(null); setError(null); setClicks(0); setState("HOME_EMPTY");
  }, []);

  const endRun = useCallback(async () => {
    if (settings.confirmBeforeAbandoning) { setConfirmEnd(true); return; }
    await abandonRun();
  }, [abandonRun, settings.confirmBeforeAbandoning]);

  const changeMode = useCallback((mode: GameMode) => {
    saveSettings({ ...settings, defaultMode: mode });
    setChallenge(null); setResult(null); setState("HOME_EMPTY");
  }, [saveSettings, settings]);

  const backToHome = useCallback(() => setState(challenge ? "HOME_READY" : "HOME_EMPTY"), [challenge]);
  const openHistory = useCallback(async () => { setStats(await commands.getStats()); setState("HISTORY"); }, []);

  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      if (state === "SETTINGS" || state === "HISTORY") backToHome();
      else if (state === "RESULT") newRun();
    };
    addEventListener("keydown", key); return () => removeEventListener("keydown", key);
  }, [backToHome, newRun, state]);

  const updater = <UpdatePrompt phase={updatePhase} version={updateVersion} progress={updateProgress} onUpdate={() => void downloadUpdate()} onInstall={() => void installUpdate()} onLater={() => setUpdatePhase("hidden")} />;
  const safeScreen = (screen: React.ReactNode) => <>{screen}{updater}</>;

  if (state === "SETTINGS") return safeScreen(<SettingsScreen settings={settings} appVersion={appVersion} datasetVersion={datasetVersion} onUpdate={saveSettings} onClearHistory={commands.clearRunHistory} onResetPersonalBests={commands.resetPersonalBests} onResetStreak={commands.resetStreak} onCheckUpdate={() => checkForUpdate(true)} updateFeedback={updateFeedback} onBack={backToHome} />);
  if (state === "HISTORY") return safeScreen(<HistoryScreen stats={stats} onBack={backToHome} />);
  if (state === "ERROR" && error) return safeScreen(<ErrorScreen error={error} onRetry={challenge ? () => void launchRun(challenge) : randomize} onBack={newRun} />);
  if (state === "PLAYER_HANDOFF") return <PlayerHandoff onReady={() => void startPlayerTwo()} onExit={newRun} />;
  if (state === "MULTIPLAYER_RESULT" && playerOne && playerTwo) return safeScreen(<MultiplayerResult playerOne={playerOne} playerTwo={playerTwo} onAgain={() => void replayMultiplayer()} onNewRun={newRun} />);
  if (state === "RESULT" && result) return safeScreen(<ResultScreen result={result} onAgain={again} onNewRun={newRun} />);
  if (state === "COUNTDOWN") return <main className="full-screen countdown-screen" aria-live="assertive"><span>{countdown}</span></main>;
  if ((state === "GAME_LOADING" || state === "GAME_RUNNING" || state === "GAME_FINISHING") && challenge) return <div className="game-shell"><GameHeader challenge={challenge} settings={settings} running={state === "GAME_RUNNING"} clicks={clicks} stage={stage} confirming={confirmEnd} onEnd={endRun} onConfirmEnd={() => void abandonRun()} onCancelEnd={() => setConfirmEnd(false)} /><div className="wiki-loading" aria-live="polite">{state === "GAME_LOADING" ? "LOADING WIKIPEDIA…" : ""}</div></div>;
  return safeScreen(<StartScreen state={state} challenge={challenge} settings={settings} onModeChange={changeMode} onRandomize={randomize} onStart={start} onSettings={() => setState("SETTINGS")} onHistory={() => void openHistory()} />);
}
