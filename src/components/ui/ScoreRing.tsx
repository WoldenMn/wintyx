import { useEffect, useState } from "react";
import { getScoreColor } from "../../lib/helpers";

interface ScoreRingProps {
  /** Valeur 0-100. */
  value: number;
  /** Libelle court affiche sous la valeur au centre. */
  label?: string;
  /** Diametre en px. */
  size?: number;
  /** Classe de couleur Tailwind (ex: getUsageColor). Defaut : couleur de score. */
  colorClass?: string;
  /** Epaisseur du trait en px. */
  stroke?: number;
  ariaLabel?: string;
}

export function ScoreRing({ value, label, size = 88, colorClass, stroke = 7, ariaLabel }: ScoreRingProps) {
  const clamped = Math.max(0, Math.min(100, Math.round(value)));
  const color = colorClass ?? getScoreColor(clamped);
  const r = (size - stroke) / 2;
  const circumference = 2 * Math.PI * r;
  const [filled, setFilled] = useState(false);
  useEffect(() => {
    const frame = requestAnimationFrame(() => setFilled(true));
    return () => cancelAnimationFrame(frame);
  }, []);
  const dash = filled ? (clamped / 100) * circumference : 0;
  const center = size / 2;

  return (
    <div
      role="img"
      aria-label={ariaLabel ?? `${label ? label + " : " : ""}${clamped} sur 100`}
      className={`relative inline-flex items-center justify-center ${color}`}
      style={{ width: size, height: size }}
    >
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true">
        <circle cx={center} cy={center} r={r} fill="none" stroke="var(--t-ring-track)" strokeWidth={stroke} />
        <circle
          cx={center}
          cy={center}
          r={r}
          fill="none"
          stroke="currentColor"
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={`${dash} ${circumference}`}
          transform={`rotate(-90 ${center} ${center})`}
          style={{ transition: "stroke-dasharray 600ms var(--ease-brand)" }}
        />
      </svg>
      <div className="absolute inset-0 flex flex-col items-center justify-center">
        <span className="text-2xl font-semibold tabular-nums leading-none">{clamped}</span>
        {label && <span className="text-xs text-fg-muted mt-1">{label}</span>}
      </div>
    </div>
  );
}

export default ScoreRing;
