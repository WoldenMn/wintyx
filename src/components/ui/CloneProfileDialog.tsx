import { useState, useRef, useEffect } from "react";
import { ActionButton } from "./ActionButton";
import type { Profile } from "../../lib/types";

interface Props {
  presets: Profile[];
  onClone: (sourceId: string, name: string) => void | Promise<void>;
  onCancel: () => void;
}

export function CloneProfileDialog({ presets, onClone, onCancel }: Props) {
  const [sourceId, setSourceId] = useState(presets[0]?.id ?? "");
  const [name, setName] = useState("");
  const [pending, setPending] = useState(false);
  const pendingRef = useRef(false);
  const dialogRef = useRef<HTMLDivElement>(null);

  const canClone = sourceId.length > 0 && name.trim().length > 0;

  async function submit() {
    if (pendingRef.current) return;
    pendingRef.current = true;
    setPending(true);
    try {
      await onClone(sourceId, name.trim());
    } finally {
      pendingRef.current = false;
      setPending(false);
    }
  }

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onCancel();
        return;
      }
      if (e.key === "Tab") {
        const focusables = dialogRef.current?.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
        );
        if (!focusables || focusables.length === 0) return;
        const first = focusables[0];
        const last = focusables[focusables.length - 1];
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [onCancel]);

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="clone-dialog-title"
      className="fixed inset-0 z-50 bg-black/60 flex items-center justify-center p-4"
    >
      <div
        ref={dialogRef}
        className="bg-bg-card border border-border rounded-lg theme-card p-5 w-full max-w-md"
      >
        <h2 id="clone-dialog-title" className="text-lg font-semibold text-fg-primary mb-4">
          Cloner un preset
        </h2>

        <div className="space-y-3 mb-5">
          <div>
            <label htmlFor="clone-source" className="block text-xs text-fg-muted mb-1">
              Source
            </label>
            <select
              id="clone-source"
              autoFocus
              value={sourceId}
              onChange={(e) => setSourceId(e.target.value)}
              className="w-full px-3 py-2 bg-bg-input border border-border rounded-md text-sm text-fg-primary focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
            >
              {presets.map((p) => (
                <option key={p.id} value={p.id}>{p.name}</option>
              ))}
            </select>
          </div>
          <div>
            <label htmlFor="clone-name" className="block text-xs text-fg-muted mb-1">
              Nom du nouveau profil
            </label>
            <input
              id="clone-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              maxLength={80}
              placeholder="Mon profil personnalisé"
              className="w-full px-3 py-2 bg-bg-input border border-border rounded-md text-sm text-fg-primary focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
            />
          </div>
        </div>

        <div className="flex gap-2 justify-end">
          <ActionButton variant="neutral" onClick={onCancel}>Annuler</ActionButton>
          <ActionButton
            variant="accent"
            onClick={() => void submit()}
            loading={pending}
            disabled={!canClone}
          >
            Cloner
          </ActionButton>
        </div>
      </div>
    </div>
  );
}
