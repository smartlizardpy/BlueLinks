export interface DurationParts { whole: string; fraction: string; label: string }

export function formatDurationParts(milliseconds: number): DurationParts {
  const safe = Math.max(0, Math.floor(milliseconds));
  const centiseconds = Math.floor((safe % 1000) / 10);
  const totalSeconds = Math.floor(safe / 1000);
  const seconds = totalSeconds % 60;
  const totalMinutes = Math.floor(totalSeconds / 60);
  const minutes = totalMinutes % 60;
  const hours = Math.floor(totalMinutes / 60);
  const two = (value: number) => value.toString().padStart(2, "0");
  const whole = hours > 0 ? `${hours}:${two(minutes)}:${two(seconds)}` : `${two(totalMinutes)}:${two(seconds)}`;
  const fraction = two(centiseconds);
  return { whole, fraction, label: `${whole}.${fraction}` };
}

export function formatDuration(milliseconds: number): string {
  return formatDurationParts(milliseconds).label;
}
