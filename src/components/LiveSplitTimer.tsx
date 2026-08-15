import { formatDurationParts } from "../lib/timerFormat";

interface Props {
  milliseconds: number;
  size?: "header" | "result";
}

export function LiveSplitTimer({ milliseconds, size = "header" }: Props) {
  const { whole, fraction, label } = formatDurationParts(milliseconds);
  return <span className={`livesplit-timer ${size}`} aria-label={label}>
    <span className="livesplit-whole">{whole}</span><span className="livesplit-fraction">.{fraction}</span>
  </span>;
}
