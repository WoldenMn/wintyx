import { useState, useEffect, useRef, useMemo, useDeferredValue } from "react";
import { Search } from "lucide-react";
import { PAGES } from "../../lib/pages";
import { normalizeSearch } from "../../lib/search";

interface Props {
  open: boolean;
  onClose: () => void;
  onNavigate: (page: string) => void;
}

export function CommandPalette({ open, onClose, onNavigate }: Props) {
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  const filtered = useMemo(() => {
    if (!deferredQuery.trim()) return PAGES;
    const q = normalizeSearch(deferredQuery.trim());
    return PAGES.filter(
      (c) =>
        normalizeSearch(c.label).includes(q) ||
        normalizeSearch(c.group).includes(q) ||
        normalizeSearch(c.keywords).includes(q)
    );
  }, [deferredQuery]);

  useEffect(() => {
    if (!open) return;
    previousFocusRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    return () => {
      previousFocusRef.current?.focus();
      previousFocusRef.current = null;
    };
  }, [open]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setSelectedIndex(0);
      // Fix v2.7.9 C1 : cleanup setTimeout pour eviter setState apres unmount
      // si le dialog est ferme dans les 50ms suivant l'ouverture.
      const timer = setTimeout(() => inputRef.current?.focus(), 50);
      return () => clearTimeout(timer);
    }
  }, [open]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, filtered.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter" && filtered[selectedIndex]) {
      onNavigate(filtered[selectedIndex].id);
      onClose();
    } else if (e.key === "Escape") {
      onClose();
    } else if (e.key === "Tab") {
      e.preventDefault();
      inputRef.current?.focus();
    }
  };

  const activeOption = filtered[selectedIndex];

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]"
      role="dialog"
      aria-modal="true"
      aria-label="Palette de commandes"
    >
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" onClick={onClose} />
      <div className="relative bg-bg-card border border-border rounded-xl w-full max-w-lg mx-4 shadow-2xl overflow-hidden">
        <div className="flex items-center gap-3 px-4 py-3 border-b border-border">
          <Search size={14} className="text-fg-muted shrink-0" aria-hidden="true" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Rechercher une page..."
            aria-label="Rechercher une page"
            role="combobox"
            aria-expanded="true"
            aria-autocomplete="list"
            aria-controls="command-palette-list"
            aria-activedescendant={activeOption ? `command-palette-${activeOption.id}` : undefined}
            className="flex-1 bg-transparent text-fg-primary text-sm outline-none placeholder:text-fg-muted focus-visible:ring-2 focus-visible:ring-accent rounded"
          />
          <kbd className="text-xs text-fg-muted bg-bg-elevated px-1.5 py-0.5 rounded">ESC</kbd>
        </div>
        <div id="command-palette-list" role="listbox" aria-label="Pages" className="max-h-[300px] overflow-y-auto py-2">
          {filtered.length === 0 && (
            <p className="text-center text-fg-muted text-sm py-6">Aucun résultat</p>
          )}
          {filtered.map((item, i) => (
            <button
              key={item.id}
              id={`command-palette-${item.id}`}
              type="button"
              role="option"
              aria-selected={i === selectedIndex}
              tabIndex={-1}
              className={`w-full text-left px-4 py-2.5 flex items-center justify-between text-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-accent ${
                i === selectedIndex ? "bg-accent/10 text-fg-primary" : "text-fg-dim hover:bg-bg-hover"
              }`}
              onClick={() => {
                onNavigate(item.id);
                onClose();
              }}
              onMouseEnter={() => setSelectedIndex(i)}
            >
              <span>{item.label}</span>
              <span className="text-xs text-fg-muted">{item.group}</span>
            </button>
          ))}
        </div>
        <div className="flex items-center justify-between gap-3 px-4 py-2 border-t border-border bg-bg-elevated text-xs text-fg-muted">
          <div className="flex items-center gap-3">
            <span className="flex items-center gap-1">
              <kbd className="bg-bg-card border border-border px-1.5 py-0.5 rounded">↑↓</kbd>
              naviguer
            </span>
            <span className="flex items-center gap-1">
              <kbd className="bg-bg-card border border-border px-1.5 py-0.5 rounded">↵</kbd>
              ouvrir
            </span>
            <span className="flex items-center gap-1">
              <kbd className="bg-bg-card border border-border px-1.5 py-0.5 rounded">ESC</kbd>
              fermer
            </span>
          </div>
          <span>{filtered.length} {filtered.length > 1 ? "pages" : "page"}</span>
        </div>
      </div>
    </div>
  );
}
