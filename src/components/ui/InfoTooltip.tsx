import { useId, useState } from "react";
import { getHelpText } from "../../lib/help";

interface Props {
  /** Texte direct ; a utiliser quand l'id n'est pas dans help.ts */
  text?: string;
  /** Id dans HELP_TEXTS (src/lib/help.ts). Prioritaire sur text si fourni et trouve. */
  textId?: string;
  /** Label accessible du bouton (default: "Aide") */
  ariaLabel?: string;
  /** Taille du badge (default: "sm") */
  size?: "sm" | "md";
}

export function InfoTooltip({ text, textId, ariaLabel = "Aide", size = "sm" }: Props) {
  const [open, setOpen] = useState(false);
  const tipId = useId();
  const resolved = (textId && getHelpText(textId)) ?? text;

  if (!resolved) return null;

  const dimension = size === "md" ? "w-5 h-5 text-xs" : "w-4 h-4 text-xs";

  return (
    <span className="relative inline-flex">
      <button
        type="button"
        aria-label={ariaLabel}
        aria-describedby={open ? tipId : undefined}
        onClick={() => setOpen((v) => !v)}
        onFocus={() => setOpen(true)}
        onBlur={() => setOpen(false)}
        onMouseEnter={() => setOpen(true)}
        onMouseLeave={() => setOpen(false)}
        className={`${dimension} rounded-full bg-bg-elevated border border-border text-fg-dim hover:text-fg-primary hover:border-accent inline-flex items-center justify-center font-medium focus:outline-none focus-visible:ring-2 focus-visible:ring-accent shrink-0`}
      >
        ?
      </button>
      {open && (
        <span
          id={tipId}
          role="tooltip"
          className="absolute z-40 bottom-full left-1/2 -translate-x-1/2 mb-2 w-64 p-2.5 bg-bg-card border border-border rounded-md shadow-lg text-xs text-fg-primary leading-relaxed pointer-events-none"
        >
          {resolved}
        </span>
      )}
    </span>
  );
}

export default InfoTooltip;
