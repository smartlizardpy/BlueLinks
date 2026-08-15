import { useState } from "react";
import { GameButton } from "../components/GameButton";
import { MODE_LABELS, type GameMode, type Settings } from "../state/game";

interface Props {
  settings: Settings;
  appVersion: string;
  datasetVersion: string;
  onUpdate: (settings: Settings) => void;
  onClearHistory: () => Promise<void>;
  onResetPersonalBests: () => Promise<void>;
  onResetStreak: () => Promise<void>;
  onCheckUpdate: () => Promise<void>;
  updateFeedback: string;
  onBack: () => void;
}

function Toggle({ value, onChange, label }: { value: boolean; onChange: (value: boolean) => void; label: string }) {
  return <button className="setting-toggle" type="button" aria-label={label} aria-pressed={value} onClick={() => onChange(!value)}>{value ? "ON" : "OFF"}</button>;
}

export function SettingsScreen({ settings, appVersion, datasetVersion, onUpdate, onClearHistory, onResetPersonalBests, onResetStreak, onCheckUpdate, updateFeedback, onBack }: Props) {
  const [feedback, setFeedback] = useState("");
  const update = <K extends keyof Settings>(key: K, value: Settings[K]) => onUpdate({ ...settings, [key]: value });
  const action = async (label: string, operation: () => Promise<void>) => {
    await operation(); setFeedback(`${label} DONE.`); window.setTimeout(() => setFeedback(""), 1800);
  };
  return <main className="full-screen utility-screen">
    <section className="utility-content settings-content">
      <header className="utility-heading"><div><p className="eyebrow">BLUELINK</p><h1>SETTINGS</h1></div><GameButton variant="secondary" onClick={onBack}>BACK</GameButton></header>

      <div className="settings-section"><h2>GAME</h2>
        <label className="setting-row"><span>Default Mode</span><select value={settings.defaultMode} onChange={event => update("defaultMode", event.target.value as GameMode)}>{Object.entries(MODE_LABELS).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
        <label className="setting-row"><span>Default Difficulty</span><select value={settings.defaultDifficulty} onChange={event => update("defaultDifficulty", event.target.value as Settings["defaultDifficulty"])}><option value="normal">NORMAL</option><option value="evil">EVIL</option></select></label>
        <label className="setting-row"><span>Max Clicks</span><input type="number" min="1" max="50" value={settings.maxClicks} onChange={event => update("maxClicks", Number(event.target.value))} /></label>
        <label className="setting-row"><span>Time Limit</span><select value={settings.timeLimitSeconds} onChange={event => update("timeLimitSeconds", Number(event.target.value))}><option value={30}>30 SECONDS</option><option value={60}>60 SECONDS</option><option value={120}>120 SECONDS</option><option value={300}>300 SECONDS</option><option value={0}>UNLIMITED</option></select></label>
        <div className="setting-row"><span>3-second Countdown</span><Toggle label="3-second countdown" value={settings.countdown} onChange={value => update("countdown", value)} /></div>
        <div className="setting-row"><span>Show Timer During Game</span><Toggle label="Show timer" value={settings.showTimer} onChange={value => update("showTimer", value)} /></div>
        <div className="setting-row"><span>Show Click Count During Game</span><Toggle label="Show click count" value={settings.showClickCount} onChange={value => update("showClickCount", value)} /></div>
        <div className="setting-row"><span>Confirm Before Abandoning</span><Toggle label="Confirm before abandoning" value={settings.confirmBeforeAbandoning} onChange={value => update("confirmBeforeAbandoning", value)} /></div>
      </div>

      <div className="settings-section"><h2>ANIMATION</h2>
        <div className="setting-row"><span>Scramble Animation</span><Toggle label="Scramble animation" value={settings.scrambleAnimation} onChange={value => update("scrambleAnimation", value)} /></div>
        <div className="setting-row"><span>Reduced Motion</span><Toggle label="Reduced motion" value={settings.reducedMotion} onChange={value => update("reducedMotion", value)} /></div>
      </div>

      <div className="settings-section"><h2>DATA</h2>
        <div className="setting-row"><span>Save Run History</span><Toggle label="Save run history" value={settings.saveRunHistory} onChange={value => update("saveRunHistory", value)} /></div>
        <div className="setting-actions"><button onClick={() => void action("HISTORY CLEAR", onClearHistory)}>CLEAR RUN HISTORY</button><button onClick={() => void action("PERSONAL BEST RESET", onResetPersonalBests)}>RESET PERSONAL BESTS</button><button onClick={() => void action("STREAK RESET", onResetStreak)}>RESET STREAK</button></div>
        <div className="settings-feedback" aria-live="polite">{feedback}</div>
      </div>

      <div className="settings-section"><h2>UPDATES</h2>
        <div className="setting-row"><span>Automatically Check for Updates</span><Toggle label="Automatically check for updates" value={settings.automaticallyCheckForUpdates} onChange={value => update("automaticallyCheckForUpdates", value)} /></div>
        <div className="setting-row"><span>Check for Updates</span><button className="check-update" onClick={() => void onCheckUpdate()}>CHECK NOW</button></div>
        <div className="settings-feedback" aria-live="polite">{updateFeedback}</div>
        <div className="setting-row readonly"><span>Current Version</span><strong>{appVersion}</strong></div>
      </div>

      <div className="settings-section"><h2>ABOUT</h2>
        <div className="setting-row readonly"><span>BlueLink</span><strong>{appVersion}</strong></div>
        <div className="setting-row readonly"><span>Dataset</span><strong>{datasetVersion.toUpperCase()}</strong></div>
        <p className="attribution">Wikipedia content is provided by Wikimedia contributors under applicable Creative Commons licensing. BlueLink is not affiliated with the Wikimedia Foundation.</p>
      </div>
    </section>
  </main>;
}
