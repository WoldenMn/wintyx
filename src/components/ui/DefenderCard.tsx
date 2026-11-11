import { ShieldCheck, ShieldAlert, Shield } from "lucide-react";
import type { DefenderStatus } from "../../lib/bindings";

export type DefenderDisplay = "active" | "passive-thirdparty" | "unprotected" | "unavailable";

/** Croise l'etat Defender avec l'antivirus actif pour eviter une fausse alarme. */
export function defenderState(
  status: DefenderStatus | undefined,
  activeAntivirus: string | undefined,
): DefenderDisplay {
  if (!status) return "unavailable";
  if (status.realTimeProtection) return "active";
  const av = (activeAntivirus ?? "").toLowerCase().trim();
  const thirdParty = av !== "" && av !== "inconnu" && av !== "unknown" && !av.includes("defender");
  return thirdParty ? "passive-thirdparty" : "unprotected";
}

export function DefenderCard({
  status,
  activeAntivirus,
  loading,
  error,
}: {
  status: DefenderStatus | undefined;
  activeAntivirus: string | undefined;
  loading: boolean;
  error: string | null;
}) {
  const wrap = "bg-bg-card border border-border rounded-lg p-5 mb-6 theme-card";

  if (loading) {
    return (
      <div className={wrap}>
        <p className="text-sm font-medium text-fg-primary mb-1">Windows Defender</p>
        <p className="text-xs text-fg-muted">Vérification de l'état…</p>
      </div>
    );
  }

  const state = error ? "unavailable" : defenderState(status, activeAntivirus);

  const header: Record<DefenderDisplay, { icon: typeof Shield; color: string; label: string }> = {
    active: { icon: ShieldCheck, color: "text-success", label: "Defender : protection active" },
    "passive-thirdparty": { icon: ShieldCheck, color: "text-fg-dim", label: `Defender en veille : ${activeAntivirus ?? "un antivirus tiers"} assure la protection` },
    unprotected: { icon: ShieldAlert, color: "text-danger", label: "Aucune protection temps réel active" },
    unavailable: { icon: Shield, color: "text-fg-muted", label: "État Defender indisponible" },
  };
  const h = header[state];
  const Icon = h.icon;

  return (
    <div className={wrap}>
      <div className="flex items-center gap-2">
        <Icon size={18} className={h.color} aria-hidden="true" />
        <p className={`text-sm font-medium ${h.color}`}>{h.label}</p>
      </div>
      {state === "active" && status && (
        <div className="mt-2 flex flex-wrap gap-x-6 gap-y-1 text-xs text-fg-dim">
          <span>
            Signatures {status.signatureVersion ?? "?"}
            {status.signatureAgeDays !== null && (
              <span className={status.signatureAgeDays > 7 ? "text-warning" : ""}>
                , il y a {status.signatureAgeDays}j
              </span>
            )}
          </span>
          <span>Dernière analyse : {status.lastScan ?? "—"}</span>
          {status.tamperProtected && <span>Protection anti-altération active</span>}
        </div>
      )}
    </div>
  );
}

export default DefenderCard;
