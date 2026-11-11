interface UnavailableValueProps {
  message: string;
  onRetry?: () => void;
  retryAriaLabel?: string;
}

export function UnavailableValue({ message, onRetry, retryAriaLabel }: UnavailableValueProps) {
  return (
    <div role="alert">
      <p className="text-lg font-semibold text-danger">Indisponible</p>
      <p className="text-xs text-fg-dim mt-1 break-words">{message}</p>
      {onRetry && (
        <button
          type="button"
          onClick={onRetry}
          aria-label={retryAriaLabel}
          className="mt-2 text-xs text-accent hover:underline rounded px-1 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
        >
          Réessayer
        </button>
      )}
    </div>
  );
}

export default UnavailableValue;
