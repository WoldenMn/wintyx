import { useEffect, useRef, useState, type ReactNode } from "react";
import { getScoreColor } from "../../lib/helpers";

const PROGRESS_BAR_COLORS: Record<string, string> = {
  "text-success": "bg-success",
  "text-warning": "bg-warning",
  "text-danger": "bg-danger",
  "text-fg-primary": "bg-fg-primary",
};

interface StatCardProps {
  label: string;
  value: string | number;
  suffix?: string;
  score?: number;
  icon?: ReactNode;
  subtitle?: string;
  className?: string;
  onClick?: () => void;
  ariaLabel?: string;
  progress?: number;
}

export function StatCard({
  label,
  value,
  suffix,
  score,
  icon,
  subtitle,
  className = "",
  onClick,
  ariaLabel,
  progress,
}: StatCardProps) {
  const color = score !== undefined ? getScoreColor(score) : "text-fg-primary";

  const [displayValue, setDisplayValue] = useState<number | string>(value);
  const prevNumericRef = useRef<number | null>(
    typeof value === "number" ? value : null,
  );

  useEffect(() => {
    if (typeof value !== "number") {
      setDisplayValue(value);
      prevNumericRef.current = null;
      return;
    }
    const from = prevNumericRef.current ?? value;
    const to = value;
    if (from === to) {
      setDisplayValue(to);
      prevNumericRef.current = to;
      return;
    }
    const duration = 200;
    let start: number | null = null;
    let raf = 0;
    const tick = (now: number) => {
      if (start === null) start = now;
      const t = Math.max(0, Math.min(1, (now - start) / duration));
      const eased = 1 - Math.pow(1 - t, 3);
      const current = from + (to - from) * eased;
      setDisplayValue(t < 1 ? Math.round(current) : to);
      if (t < 1) raf = requestAnimationFrame(tick);
      else prevNumericRef.current = to;
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [value]);

  const baseClasses =
    "rounded-xl border border-bg-hover bg-bg-secondary p-4 theme-card text-left w-full";
  const interactiveClasses = onClick
    ? " cursor-pointer hover:border-accent/60 hover:bg-bg-hover transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
    : "";
  const combined = `${baseClasses}${interactiveClasses} ${className}`;
  const resolvedAria =
    ariaLabel ?? (onClick ? `${label} : ${value}${suffix ?? ""}` : undefined);

  const content = (
    <>
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs text-fg-dim">{label}</span>
        {icon}
      </div>
      <div className="flex items-baseline gap-2">
        <span className={`text-3xl font-semibold tabular-nums ${color}`}>{displayValue}</span>
        {suffix && <span className="text-sm text-fg-dim">{suffix}</span>}
      </div>
      {subtitle && <p className="text-xs text-fg-dim mt-1">{subtitle}</p>}
      {progress !== undefined && (
        <div className="mt-3 h-1 rounded-full bg-bg-hover overflow-hidden">
          <div
            className={`h-full rounded-full ${PROGRESS_BAR_COLORS[color] ?? "bg-fg-primary"}`}
            style={{ width: `${Math.max(0, Math.min(100, progress))}%` }}
          />
        </div>
      )}
    </>
  );

  if (onClick) {
    return (
      <button
        type="button"
        onClick={onClick}
        className={combined}
        aria-label={resolvedAria}
      >
        {content}
      </button>
    );
  }

  return <div className={combined}>{content}</div>;
}

export default StatCard;
