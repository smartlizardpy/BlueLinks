import { useEffect, useState } from "react";
import { GameButton } from "../components/GameButton";
import { ScrambleText } from "../components/ScrambleText";
import { MODE_LABELS, type AppState, type Challenge, type GameMode, type Settings } from "../state/game";

interface Props { state: AppState; challenge: Challenge | null; settings: Settings; onModeChange: (mode: GameMode) => void; onRandomize: () => void; onStart: () => void; onSettings: () => void; onHistory: () => void }

export function StartScreen({ state, challenge, settings, onModeChange, onRandomize, onStart, onSettings, onHistory }: Props) {
  const [startDone, setStartDone] = useState(false);
  const [targetDone, setTargetDone] = useState(false);
  const randomizing = state === "HOME_RANDOMIZING";
  useEffect(() => { setStartDone(false); setTargetDone(false); }, [challenge]);
  const revealed = Boolean(challenge && startDone && targetDone && !randomizing);

  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (event.key === "Enter" && revealed) onStart();
      else if (event.key.toLowerCase() === "r" && !randomizing) onRandomize();
    };
    addEventListener("keydown", key); return () => removeEventListener("keydown", key);
  }, [revealed, randomizing, onRandomize, onStart]);

  return <main className="full-screen home-screen">
    <section className="home-content" aria-live="polite">
      <header className="brand"><h1>BLUELINK</h1><p>WIKIPEDIA SPEEDRUN</p></header>
      <div className="challenge-stack">
        <p className="eyebrow">START</p>
        <div className="hero-title">
          {challenge ? <ScrambleText key={`s-${challenge.start.id}`} text={challenge.start.title} duration={760} onComplete={() => setStartDone(true)} /> : "??????????????"}
        </div>
        <div className="route-arrow" aria-hidden>↓</div>
        <p className="eyebrow">TARGET</p>
        <div className="hero-title">
          {challenge ? <ScrambleText key={`t-${challenge.target.id}`} text={challenge.target.title} duration={760} delay={120} onComplete={() => setTargetDone(true)} /> : "??????????????"}
        </div>
        <div className={`click-objective ${revealed ? "visible" : ""}`}>{challenge?.mode === "maxClicks" ? `${challenge.clickLimit} CLICKS MAX` : challenge?.mode === "timeLimit" && challenge.timeLimitSeconds ? `${challenge.timeLimitSeconds} SECONDS` : challenge?.mode === "evil" ? `DIFFICULTY ${Math.round(challenge.difficulty * 100)}` : challenge?.mode === "twoPlayer" ? "SAME ROUTE · TWO PLAYERS" : challenge?.mode === "gauntlet" ? "5 TARGETS · ONE TIMER" : challenge?.mode === "fewestClicks" ? "LOWEST CLICK COUNT WINS" : challenge?.mode === "speedrun" ? "FASTEST TIME WINS" : "STANDARD RUN"}</div>
      </div>
      <div className="button-row">
        <GameButton variant={challenge ? "secondary" : "primary"} onClick={onRandomize} disabled={randomizing}>
          {challenge ? "RANDOMIZE AGAIN" : randomizing ? "RANDOMIZING…" : "RANDOMIZE"}
        </GameButton>
        {revealed && <GameButton reveal onClick={onStart}>START →</GameButton>}
      </div>
      <div className="home-tools">
        <label>MODE <select aria-label="Game mode" value={settings.defaultMode} onChange={event => onModeChange(event.target.value as GameMode)}>{Object.entries(MODE_LABELS).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
        <button onClick={onHistory}>HISTORY</button><button onClick={onSettings}>SETTINGS</button>
      </div>
    </section>
  </main>;
}
