import { GameButton } from "../components/GameButton";
import type { GameError } from "../state/game";

interface Props { error: GameError; onRetry: () => void; onBack: () => void }
export function ErrorScreen({ error, onRetry, onBack }: Props) {
  const title = error.kind === "dataset" ? "ARTICLE DATA IS MISSING." : error.kind === "connection" ? "CONNECTION LOST." : "CAN'T REACH WIKIPEDIA.";
  return <main className="full-screen error-screen"><section><div className="brand"><h1>BLUELINK</h1></div><h2>{title}</h2><p>{error.message}</p><div className="button-row"><GameButton onClick={onRetry}>RETRY</GameButton><GameButton variant="secondary" onClick={onBack}>NEW RUN</GameButton></div></section></main>;
}
