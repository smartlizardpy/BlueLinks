import { GameButton } from "../components/GameButton";
import { LiveSplitTimer } from "../components/LiveSplitTimer";
import { multiplayerWinner } from "../lib/multiplayer";
import type { FinishPayload } from "../state/game";

export function PlayerHandoff({ onReady, onExit }: { onReady: () => void; onExit: () => void }) {
  return <main className="full-screen result-screen"><section className="result-content handoff-screen">
    <p className="eyebrow">PLAYER 1 COMPLETE</p><h1 className="done">PASS IT OVER.</h1>
    <p className="handoff-copy">PLAYER 2 GETS THE SAME START AND TARGET.<br />PLAYER 1'S RESULT STAYS HIDDEN.</p>
    <div className="button-row result-buttons"><GameButton onClick={onReady}>PLAYER 2 READY →</GameButton><GameButton variant="secondary" onClick={onExit}>EXIT</GameButton></div>
  </section></main>;
}

export function MultiplayerResult({ playerOne, playerTwo, onAgain, onNewRun }: { playerOne: FinishPayload; playerTwo: FinishPayload; onAgain: () => void; onNewRun: () => void }) {
  const winner = multiplayerWinner(playerOne, playerTwo);
  return <main className="full-screen result-screen"><section className="result-content comparison-screen">
    <p className="eyebrow">HEAD TO HEAD</p><h1 className="done">{winner === 0 ? "DRAW." : `PLAYER ${winner} WINS.`}</h1>
    <div className="comparison-grid">
      {[playerOne, playerTwo].map((run, index) => <article className={winner === index + 1 ? "winner" : ""} key={index}>
        <h2>PLAYER {index + 1}</h2><LiveSplitTimer milliseconds={run.durationMs} size="result" />
        <p>{run.success ? `${run.clicks} ${run.clicks === 1 ? "CLICK" : "CLICKS"}` : "RUN FAILED"}</p>
      </article>)}
    </div>
    <div className="button-row result-buttons"><GameButton onClick={onAgain}>PLAY AGAIN</GameButton><GameButton variant="secondary" onClick={onNewRun}>NEW RUN</GameButton></div>
  </section></main>;
}
