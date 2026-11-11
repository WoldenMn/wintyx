import { useId } from "react";

interface BrandMarkProps {
  size?: number;
  className?: string;
}

/** Symbole Wintyx sans tuile : un W dont le dernier jambage forme une coche. Charte : docs/BRAND.md. */
export function BrandMark({ size = 20, className }: BrandMarkProps) {
  const gradientId = `wintyx-ion-${useId().replace(/[^a-zA-Z0-9_-]/g, "")}`;
  return (
    <svg
      width={size}
      height={(size * 520) / 720}
      viewBox="160 240 720 520"
      className={className}
      aria-hidden="true"
      focusable="false"
    >
      <defs>
        <linearGradient id={gradientId} x1="0" y1="1" x2="1" y2="0">
          <stop offset="0" stopColor="var(--t-accent-hover)" />
          <stop offset="1" stopColor="var(--t-accent-contrast)" />
        </linearGradient>
      </defs>
      <polyline
        points="214,372 352,690 494,438"
        fill="none"
        stroke="var(--t-fg-primary)"
        strokeWidth={96}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <polyline
        points="494,438 624,690 822,300"
        fill="none"
        stroke={`url(#${gradientId})`}
        strokeWidth={96}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
