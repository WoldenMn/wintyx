import { AlertTriangle } from "lucide-react";
import { ActionButton } from "./ActionButton";
import { useNavigateOptional, useHasNavigation } from "../../contexts/NavigationContext";

interface ErrorStateProps {
  /** Message d'erreur deja mis en forme (via describeError). */
  message: string;
  /** Si fourni, affiche un bouton Reessayer qui relance la commande. */
  onRetry?: () => void;
  title?: string;
}

/**
 * Etat d'erreur de chargement, pendant a EmptyState mais semantiquement distinct :
 * un echec de commande ne doit jamais etre confondu avec un resultat vide
 * (politique de mode degrade v2.12.0). Affiche le message + un bouton Reessayer.
 */
export function ErrorState({
  message,
  onRetry,
  title = "Échec du chargement",
}: ErrorStateProps) {
  const navigate = useNavigateOptional();
  const hasNav = useHasNavigation();
  return (
    <div
      role="alert"
      className="bg-bg-card border border-danger/30 rounded-lg p-12 flex flex-col items-center justify-center text-center theme-card"
    >
      <AlertTriangle size={28} className="text-danger mb-3" aria-hidden="true" />
      <p className="text-danger text-sm font-medium">{title}</p>
      <p className="text-fg-dim text-xs mt-1 max-w-md leading-relaxed">{message}</p>
      {(onRetry || hasNav) && (
        <div className="flex items-center gap-2 mt-4">
          {onRetry && (
            <ActionButton variant="neutral" onClick={onRetry}>
              Réessayer
            </ActionButton>
          )}
          {hasNav && (
            <ActionButton variant="neutral" onClick={() => navigate("diagnostic")}>
              Diagnostiquer
            </ActionButton>
          )}
        </div>
      )}
    </div>
  );
}

export default ErrorState;
