import { useId, type ButtonHTMLAttributes, type ReactNode } from "react";
import { Lock } from "lucide-react";
import { useAdminOptional } from "../../contexts/AdminContext";

const ADMIN_REQUIRED = "Droits administrateur requis";

export type ActionVariant = "accent" | "danger" | "warning" | "success" | "neutral";

interface ActionButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> {
  variant?: ActionVariant;
  loading?: boolean;
  requiresAdmin?: boolean;
  children: ReactNode;
}

const VARIANTS: Record<ActionVariant, string> = {
  accent: "bg-accent text-on-accent hover:bg-accent-hover",
  danger: "bg-danger text-on-semantic hover:bg-danger/80",
  warning: "bg-warning text-on-semantic hover:bg-warning/80",
  success: "bg-success text-on-semantic hover:bg-success/80",
  neutral: "bg-bg-card border border-border text-fg-primary hover:bg-bg-hover",
};

export function ActionButton({
  variant = "neutral",
  loading = false,
  requiresAdmin = false,
  disabled,
  className = "",
  title,
  children,
  ...rest
}: ActionButtonProps) {
  const { isAdmin, loaded } = useAdminOptional();
  const adminBlocked = requiresAdmin && loaded && !isAdmin;
  const isDisabled = disabled || loading || adminBlocked;
  const reasonId = useId();
  const resolvedTitle = adminBlocked ? ADMIN_REQUIRED : title;
  return (
    <>
      <button
        {...rest}
        disabled={isDisabled}
        aria-disabled={isDisabled ? true : undefined}
        aria-busy={loading ? true : undefined}
        title={resolvedTitle}
        aria-describedby={adminBlocked ? reasonId : rest["aria-describedby"]}
        className={`inline-flex items-center justify-center gap-2 px-4 py-2 rounded-md text-sm active:scale-[0.98] [transition:transform_80ms,background-color_120ms] focus:outline-none focus-visible:ring-2 focus-visible:ring-accent disabled:bg-bg-elevated disabled:hover:bg-bg-elevated disabled:border disabled:border-border disabled:text-fg-dim disabled:cursor-not-allowed disabled:active:scale-100 ${VARIANTS[variant]} ${className}`}
      >
        {loading && (
          <span aria-hidden="true" className="w-3.5 h-3.5 border-2 border-current/30 border-t-current rounded-full animate-spin" />
        )}
        {adminBlocked && <Lock size={14} aria-hidden="true" />}
        {children}
      </button>
      {adminBlocked && (
        <span id={reasonId} className="sr-only">
          {ADMIN_REQUIRED}
        </span>
      )}
    </>
  );
}

export default ActionButton;
