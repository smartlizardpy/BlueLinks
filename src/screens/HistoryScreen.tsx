import { useState } from "react";
import { GameButton } from "../components/GameButton";
import { formatDuration } from "../lib/timerFormat";
import { MODE_LABELS, type RunRecord, type StatsSnapshot } from "../state/game";

interface Props { stats: StatsSnapshot; onBack: () => void }

export function HistoryScreen({ stats, onBack }: Props) {
  const [selected, setSelected] = useState<RunRecord | null>(null);
  return <main className="full-screen utility-screen">
    <section className="utility-content history-content">
      <header className="utility-heading"><div><p className="eyebrow">BLUELINK</p><h1>HISTORY</h1></div><GameButton variant="secondary" onClick={onBack}>BACK</GameButton></header>
      <div className="history-streak"><span>STREAK {stats.currentStreak}</span><span>BEST {stats.bestStreak}</span></div>
      {stats.history.length === 0 ? <div className="empty-history">NO RUNS YET.</div> : <div className="history-list">{stats.history.map(run => <button className={`history-row ${run.success ? "" : "failed"}`} key={run.id} onClick={() => setSelected(run === selected ? null : run)}>
        <span className="history-route-title">{run.startTitle} → {run.targetTitle}</span><span>{run.success ? formatDuration(run.durationMs) : "FAILED"} · {run.clicks} {run.clicks === 1 ? "CLICK" : "CLICKS"}</span><small>{MODE_LABELS[run.mode]} · DIFFICULTY {Math.round(run.difficulty * 100)}</small>
        {selected?.id === run.id && <span className="history-detail">{run.route.join("  ↓  ")}</span>}
      </button>)}</div>}
    </section>
  </main>;
}
