import { useEffect, useRef, useState } from "react";
import { LiveSplitTimer } from "../components/LiveSplitTimer";
import type { Challenge, Settings } from "../state/game";

interface Props { challenge: Challenge; settings: Settings; running: boolean; clicks: number; stage: number; confirming: boolean; onEnd: () => void; onConfirmEnd: () => void; onCancelEnd: () => void }
export function GameHeader({ challenge, settings, running, clicks, stage, confirming, onEnd, onConfirmEnd, onCancelEnd }: Props) {
  const [elapsed, setElapsed] = useState(0);
  const started = useRef(0);
  useEffect(() => {
    if (!running) { setElapsed(0); started.current = 0; return; }
    started.current = performance.now(); let frame = 0;
    const tick = (now: number) => { setElapsed(now - started.current); frame = requestAnimationFrame(tick); };
    frame = requestAnimationFrame(tick); return () => cancelAnimationFrame(frame);
  }, [running]);
  const displayTime = challenge.mode === "timeLimit" && challenge.timeLimitSeconds ? Math.max(0, challenge.timeLimitSeconds * 1000 - elapsed) : elapsed;
  return <header className="game-header">
    <div className="header-challenge">
      <span className="header-title">{challenge.start.title}</span><span className="header-arrow">→</span><span className="header-title target">{challenge.target.title}</span>
    </div>
    <div className="header-stats">{challenge.mode === "gauntlet" && <span className="stage-count">{stage}/{Math.max(1, challenge.targets.length)}</span>}{settings.showTimer && <LiveSplitTimer milliseconds={displayTime} />}{settings.showClickCount && <span className={`click-budget ${challenge.mode === "maxClicks" && clicks > challenge.clickLimit ? "over" : ""}`}>{challenge.mode === "maxClicks" ? `${clicks}/${challenge.clickLimit}` : clicks} {clicks === 1 ? "CLICK" : "CLICKS"}</span>}{confirming
      // Wikipedia's WebView covers everything below this bar, so the question
      // has to be asked up here where the app can still draw.
      ? <span className="end-confirm" role="group" aria-label="End this run?"><span>END THIS RUN?</span><button className="end-run" onClick={onConfirmEnd} autoFocus>YES</button><button className="end-run" onClick={onCancelEnd}>NO</button></span>
      : <button className="end-run" onClick={onEnd}>END RUN</button>}</div>
  </header>;
}
