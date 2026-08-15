import { useEffect, useState } from "react";
import { GameButton } from "../components/GameButton";
import { LiveSplitTimer } from "../components/LiveSplitTimer";
import { ScrambleText } from "../components/ScrambleText";
import type { FinishPayload } from "../state/game";

interface Props { result: FinishPayload; onAgain: () => void; onNewRun: () => void }
export function ResultScreen({ result, onAgain, onNewRun }: Props) {
  const [ready, setReady] = useState(false);
  useEffect(() => {
    const key = (event: KeyboardEvent) => { if (event.key === "Enter") onAgain(); };
    addEventListener("keydown", key); return () => removeEventListener("keydown", key);
  }, [onAgain]);
  return <main className="full-screen result-screen">
    <section className="result-content">
      <h1 className="done"><ScrambleText text={result.success ? "DONE." : result.outcome === "timeExpired" ? "TIME'S UP." : result.outcome === "maxClicksExceeded" ? "LIMIT MISSED." : "RUN FAILED."} duration={300} onComplete={() => setReady(true)} /></h1>
      <div className="result-time"><LiveSplitTimer milliseconds={result.durationMs} size="result" /></div>
      <div className="result-clicks">{result.mode === "maxClicks" ? `${result.clicks}/${result.clickLimit}` : result.clicks} {result.clicks === 1 ? "CLICK" : "CLICKS"}</div>
      {result.mode === "gauntlet" && <div className="result-streak">{result.stageCount}/{result.stageCount} STAGES</div>}
      {result.isPersonalBest && <div className="new-best"><ScrambleText text="NEW BEST" duration={280} delay={120} /></div>}
      {result.success && <div className="result-streak">STREAK {result.streak} · BEST {result.bestStreak}</div>}
      <div className="route" aria-label="Completed route">
        {result.route.map((title, index) => <div className="route-item" key={`${index}-${title}`}>
          <ScrambleText text={title} duration={Math.min(420, 220 + title.length * 4)} delay={Math.min(360, index * 45)} />
          {index < result.route.length - 1 && <span className="route-down" aria-hidden>↓</span>}
        </div>)}
      </div>
      {ready && <div className="button-row result-buttons"><GameButton reveal onClick={onAgain}>AGAIN</GameButton><GameButton reveal variant="secondary" onClick={onNewRun}>NEW RUN</GameButton></div>}
    </section>
  </main>;
}
