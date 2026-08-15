import { GameButton } from "./GameButton";

export type UpdatePhase = "hidden" | "available" | "downloading" | "ready" | "failed";

interface Props {
  phase: UpdatePhase;
  version: string;
  progress: number;
  onUpdate: () => void;
  onInstall: () => void;
  onLater: () => void;
}

export function UpdatePrompt({ phase, version, progress, onUpdate, onInstall, onLater }: Props) {
  if (phase === "hidden") return null;
  const title = phase === "downloading" ? "UPDATING" : phase === "ready" ? "UPDATE READY" : phase === "failed" ? "UPDATE FAILED" : "UPDATE AVAILABLE";
  return <div className="update-overlay" role="dialog" aria-modal="true" aria-labelledby="update-title">
    <section className="update-prompt">
      <p className="eyebrow" id="update-title">{title}</p>
      {phase === "available" && <h2>BLUELINK {version}</h2>}
      {phase === "downloading" && <><h2>{progress}%</h2><div className="update-progress"><span style={{ width: `${progress}%` }} /></div></>}
      {phase === "ready" && <p>THE UPDATE IS DOWNLOADED.</p>}
      {phase === "failed" && <p>YOUR CURRENT VERSION IS STILL SAFE.</p>}
      {phase !== "downloading" && <div className="button-row update-actions">
        <GameButton onClick={phase === "ready" ? onInstall : onUpdate}>{phase === "ready" ? "RESTART & UPDATE" : phase === "failed" ? "RETRY" : "UPDATE"}</GameButton>
        <GameButton variant="secondary" onClick={onLater}>LATER</GameButton>
      </div>}
    </section>
  </div>;
}
