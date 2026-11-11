import type { ReactNode } from "react";

interface EmptyStateProps {
  title: string;
  description?: string;
  hint?: ReactNode;
  icon?: ReactNode;
}

export function EmptyState({ title, description, hint, icon }: EmptyStateProps) {
  return (
    <div
      role="status"
      aria-live="polite"
      className="bg-bg-card border border-border rounded-lg p-12 flex flex-col items-center justify-center text-center theme-card"
    >
      {icon && <div className="mb-3 text-fg-dim">{icon}</div>}
      <p className="text-fg-muted text-sm font-medium">{title}</p>
      {description && <p className="text-fg-dim text-xs mt-1">{description}</p>}
      {hint && (
        <p className="text-fg-dim text-xs mt-3 max-w-md leading-relaxed">{hint}</p>
      )}
    </div>
  );
}
