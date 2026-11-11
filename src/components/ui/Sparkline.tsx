import { useMemo } from "react";

interface Props {
  data: number[];
  ariaLabel: string;
  width?: number;
  height?: number;
  color?: string;
  fillOpacity?: number;
  max?: number;
  unit?: string;
}

export function Sparkline({
  data,
  ariaLabel,
  width = 200,
  height = 48,
  color = "currentColor",
  fillOpacity = 0.15,
  max = 100,
  unit = "",
}: Props) {
  const path = useMemo(() => {
    if (data.length < 2) return "";
    const step = width / (data.length - 1);
    const points = data.map((v, i) => ({
      x: i * step,
      y: height - (Math.min(v, max) / max) * height,
    }));

    let d = `M ${points[0].x} ${points[0].y}`;
    for (let i = 1; i < points.length; i++) {
      const prev = points[i - 1];
      const curr = points[i];
      const cpx = (prev.x + curr.x) / 2;
      d += ` C ${cpx} ${prev.y}, ${cpx} ${curr.y}, ${curr.x} ${curr.y}`;
    }
    return d;
  }, [data, width, height, max]);

  const fillPath = useMemo(() => {
    if (!path) return "";
    return `${path} L ${width} ${height} L 0 ${height} Z`;
  }, [path, width, height]);

  const lastValue = data.length > 0 ? data[data.length - 1] : 0;
  const fullLabel = data.length === 0
    ? `${ariaLabel} (aucune donnée)`
    : `${ariaLabel}, valeur actuelle ${Math.round(lastValue)}${unit}`;

  return (
    <svg
      width={width}
      height={height}
      className="overflow-hidden"
      role="img"
      aria-label={fullLabel}
    >
      <title>{fullLabel}</title>
      {fillPath && (
        <path d={fillPath} fill={color} opacity={fillOpacity} />
      )}
      {path && (
        <path d={path} fill="none" stroke={color} strokeWidth={1.5} />
      )}
    </svg>
  );
}
