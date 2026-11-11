import { useState, useEffect } from "react";
import { PageHeader } from "../../components/ui/PageHeader";
import { ActionButton } from "../../components/ui/ActionButton";
import { BrandMark } from "../../components/ui/BrandMark";

interface AboutPageProps {
  onNavigate?: (page: string) => void;
}

export function AboutPage({ onNavigate }: AboutPageProps) {
  // performance.timeOrigin = demarrage de la webview (≈ lancement de l'app), pas
  // l'ouverture de cette page : la duree de session reflete la vraie session (run-23).
  const [startTime] = useState(() => new Date(performance.timeOrigin));
  const [elapsed, setElapsed] = useState("0s");

  useEffect(() => {
    const interval = setInterval(() => {
      const diff = Math.floor((Date.now() - startTime.getTime()) / 1000);
      const h = Math.floor(diff / 3600);
      const m = Math.floor((diff % 3600) / 60);
      const s = diff % 60;
      if (h > 0) {
        setElapsed(`${h}h ${m}m ${s}s`);
      } else if (m > 0) {
        setElapsed(`${m}m ${s}s`);
      } else {
        setElapsed(`${s}s`);
      }
    }, 1000);
    return () => clearInterval(interval);
  }, [startTime]);

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader title="À propos" subtitle="Informations sur Wintyx" />

      <div className="bg-bg-card border border-border rounded-lg p-8 text-center mb-6 theme-card">
        <BrandMark size={72} className="mx-auto mb-5" />
        <h2 className="text-2xl font-semibold text-fg-primary mb-1">Wintyx</h2>
        <p className="text-xs text-fg-muted mb-3">Version {__APP_VERSION__}</p>
        <p className="text-sm text-fg-dim max-w-md mx-auto">
          Maintenance, sécurité et optimisation de Windows.
          Construit avec Tauri, React et Rust.
        </p>
      </div>

      <div className="bg-bg-card border border-border rounded-lg p-5 mb-6 theme-card">
        <h2 className="text-lg font-medium text-fg-primary mb-3">Session</h2>
        <div className="flex items-center justify-between mb-2">
          <p className="text-xs text-fg-dim">Démarrage</p>
          <p className="text-sm text-fg-primary">
            {startTime.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit", second: "2-digit" })}
          </p>
        </div>
        <div className="flex items-center justify-between">
          <p className="text-xs text-fg-dim">Durée de la session</p>
          <p className="text-sm text-fg-primary">{elapsed}</p>
        </div>
      </div>

      <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
        <h2 className="text-lg font-medium text-fg-primary mb-3">Accès rapide</h2>
        <div className="flex items-center gap-3 flex-wrap">
          <ActionButton variant="neutral" onClick={() => onNavigate?.("logs")}>
            Logs
          </ActionButton>
          <ActionButton variant="neutral" onClick={() => onNavigate?.("backup")}>
            Sauvegardes
          </ActionButton>
        </div>
      </div>
    </div>
  );
}

export default AboutPage;
