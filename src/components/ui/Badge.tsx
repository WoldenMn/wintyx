import type { ReactNode } from "react";

export type BadgeVariant = "success" | "danger" | "warning" | "info" | "accent" | "neutral";

interface BadgeProps {
  variant?: BadgeVariant;
  children: ReactNode;
  className?: string;
  icon?: ReactNode;
}

const VARIANTS: Record<BadgeVariant, string> = {
  success: "bg-success/15 text-success-contrast border border-success/30",
  danger: "bg-danger/15 text-danger-contrast border border-danger/30",
  warning: "bg-warning/15 text-warning-contrast border border-warning/30",
  info: "bg-info/15 text-info-contrast border border-info/30",
  accent: "bg-accent/15 text-accent-contrast border border-accent/30",
  neutral: "bg-bg-hover text-fg-primary border border-border",
};

export function Badge({ variant = "neutral", children, className = "", icon }: BadgeProps) {
  return (
    <span className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs transition-colors duration-200 ${VARIANTS[variant]} ${className}`}>
      {icon}
      {children}
    </span>
  );
}

export default Badge;
