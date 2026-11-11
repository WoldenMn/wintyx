import { useEffect, useState, useMemo } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, Copy, X } from "lucide-react";
import { useToast } from "../ui/Toast";
import { describeError } from "../../lib/helpers";
import { reportFrontendError } from "../../lib/frontend-log";
import { BrandMark } from "../ui/BrandMark";

interface TitleBarProps {
  currentPage: string;
}

export default function TitleBar({ currentPage }: TitleBarProps) {
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindow = useMemo(() => getCurrentWindow(), []);
  const toast = useToast();

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | null = null;
    const sync = () => {
      appWindow
        .isMaximized()
        .then((maximized) => { if (!cancelled) setIsMaximized(maximized); })
        .catch((e) => reportFrontendError("TitleBar", e, "warn"));
    };
    sync();
    appWindow
      .onResized(sync)
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      })
      .catch((e) => reportFrontendError("TitleBar", e, "warn"));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [appWindow]);

  async function handleMinimize() {
    try {
      await appWindow.minimize();
    } catch (e) {
      toast.error(`Réduction impossible : ${describeError(e)}`);
    }
  }

  async function handleToggleMaximize() {
    try {
      await appWindow.toggleMaximize();
    } catch (e) {
      toast.error(`Redimensionnement impossible : ${describeError(e)}`);
    }
  }

  async function handleClose() {
    try {
      await appWindow.close();
    } catch (e) {
      toast.error(`Fermeture impossible : ${describeError(e)}`);
    }
  }

  return (
    <div className="flex h-10 items-center bg-bg-sidebar border-b border-border select-none">
      <div
        className="flex flex-1 items-center gap-2 px-4 h-full"
        data-tauri-drag-region
      >
        <BrandMark size={20} className="pointer-events-none" />
        <span className="pointer-events-none font-display text-fg-primary text-sm font-semibold tracking-tight">Wintyx</span>
        <span className="pointer-events-none text-fg-muted text-sm" aria-hidden="true">/</span>
        <span className="pointer-events-none text-fg-dim text-sm">{currentPage}</span>
      </div>

      <div className="flex h-full">
        <button
          onClick={handleMinimize}
          aria-label="Réduire la fenêtre"
          className="flex items-center justify-center w-12 h-full text-fg-dim hover:bg-bg-hover transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset"
        >
          <Minus size={15} aria-hidden="true" />
        </button>
        <button
          onClick={handleToggleMaximize}
          aria-label={isMaximized ? "Restaurer la fenêtre" : "Agrandir la fenêtre"}
          className="flex items-center justify-center w-12 h-full text-fg-dim hover:bg-bg-hover transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset"
        >
          {isMaximized ? <Copy size={13} aria-hidden="true" /> : <Square size={13} aria-hidden="true" />}
        </button>
        <button
          onClick={handleClose}
          aria-label="Fermer la fenêtre"
          className="flex items-center justify-center w-12 h-full text-fg-dim hover:bg-danger hover:text-on-semantic transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset"
        >
          <X size={15} aria-hidden="true" />
        </button>
      </div>
    </div>
  );
}
