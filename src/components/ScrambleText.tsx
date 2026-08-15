import { useEffect, useRef, useState } from "react";

const GLYPHS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$%?+=/*<>";
interface Props { text: string; duration?: number; delay?: number; className?: string; onComplete?: () => void }

export function ScrambleText({ text, duration = 750, delay = 0, className, onComplete }: Props) {
  const [display, setDisplay] = useState(text);
  const completeRef = useRef(onComplete);
  completeRef.current = onComplete;

  useEffect(() => {
    if (document.documentElement.dataset.scramble === "off" || document.documentElement.classList.contains("reduce-motion") || matchMedia("(prefers-reduced-motion: reduce)").matches) {
      setDisplay(text); completeRef.current?.(); return;
    }
    let frame = 0;
    let start = 0;
    const tick = (now: number) => {
      if (!start) start = now;
      const elapsed = now - start;
      if (elapsed < delay) { frame = requestAnimationFrame(tick); return; }
      const progress = Math.min(1, (elapsed - delay) / duration);
      const resolved = progress * (text.length + 3);
      setDisplay([...text].map((char, index) => {
        if (char === " ") return " ";
        const jitter = ((index * 17) % 7) / 7;
        if (index < resolved - jitter * 2) return char;
        return GLYPHS[Math.floor(Math.random() * GLYPHS.length)];
      }).join(""));
      if (progress < 1) frame = requestAnimationFrame(tick);
      else { setDisplay(text); completeRef.current?.(); }
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [text, duration, delay]);
  return <span className={className} aria-label={text}>{display}</span>;
}
