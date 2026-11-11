import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../../hooks/use-invoke";
import { useToast } from "../../components/ui/Toast";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageLoader } from "../../components/ui/PageLoader";
import { ErrorState } from "../../components/ui/ErrorState";
import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { EmptyState } from "../../components/ui/EmptyState";
import type { NetworkAdapter, FirewallProfile, PingResult } from "../../lib/types";
import { describeError } from "../../lib/helpers";
import { UnavailableValue } from "../../components/ui/UnavailableValue";
import { CircleCheck, CircleHelp, CircleX } from "lucide-react";
import { getAdapterStatusLabel } from "../../lib/labels";

const FIREWALL_STATES: Record<string, { label: string; className: string; icon: typeof CircleCheck }> = {
  true: { label: "activé", className: "text-success", icon: CircleCheck },
  false: { label: "désactivé", className: "text-danger", icon: CircleX },
  null: { label: "état inconnu", className: "text-fg-muted", icon: CircleHelp },
};

function PingIcon({ success, host }: { success: boolean; host: string }) {
  const Icon = success ? CircleCheck : CircleX;
  return (
    <>
      <Icon size={14} className={success ? "text-success" : "text-danger"} aria-hidden="true" />
      <span className="sr-only">{`${host} : ${success ? "joignable" : "injoignable"}`}</span>
    </>
  );
}

function summarizeFirewall(profiles: FirewallProfile[]): { label: string; className: string } {
  if (profiles.length === 0) return { label: "Indéterminé", className: "text-fg-dim" };
  if (profiles.some((p) => p.isEnabled === false)) return { label: "Partiel", className: "text-warning" };
  if (profiles.some((p) => p.isEnabled === null)) return { label: "Indéterminé", className: "text-fg-dim" };
  return { label: "Actif", className: "text-success" };
}

export function NetworkPage() {
  const { data: adapters, isLoading: adaptersLoading, error: adaptersError, refresh: refreshAdapters } = useInvoke<NetworkAdapter[]>("get_network_adapters");
  const { data: firewall, isLoading: fwLoading, error: fwError, refresh: refreshFw } = useInvoke<FirewallProfile[]>("get_firewall_status");
  const [publicIp, setPublicIp] = useState<string | null>(null);
  const [ipError, setIpError] = useState<string | null>(null);
  const [ipLoading, setIpLoading] = useState(false);
  const [pingResults, setPingResults] = useState<PingResult[] | null>(null);
  const [testing, setTesting] = useState(false);
  const [flushing, setFlushing] = useState(false);
  const [pingHostInput, setPingHostInput] = useState("");
  const [pingingHost, setPingingHost] = useState(false);
  const [customPing, setCustomPing] = useState<PingResult | null>(null);
  const toast = useToast();

  const isLoading = adaptersLoading || fwLoading;
  const error = adaptersError || fwError;

  async function handleShowPublicIp() {
    setIpLoading(true);
    setIpError(null);
    try {
      setPublicIp(await invoke<string>("get_public_ip"));
    } catch (e) {
      setIpError(describeError(e));
    }
    setIpLoading(false);
  }

  async function handleTestConnectivity() {
    setTesting(true);
    try {
      const results = await invoke<PingResult[]>("test_connectivity");
      setPingResults(results);
    } catch (e) {
      toast.error(`Erreur connectivité : ${describeError(e)}`);
    }
    setTesting(false);
  }

  async function handleFlushDns() {
    setFlushing(true);
    try {
      await invoke("flush_dns");
      toast.success("Cache DNS vidé");
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    }
    setFlushing(false);
  }

  async function handlePingHost() {
    const host = pingHostInput.trim();
    if (!host) return;
    setPingingHost(true);
    try {
      const result = await invoke<PingResult>("ping_host", { host });
      setCustomPing(result);
      if (result.success) {
        toast.success(`${result.host} : ${result.roundtripMs}ms`);
      } else {
        toast.warning(`${result.host} : ${result.status}`);
      }
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    }
    setPingingHost(false);
  }

  if (isLoading) return <PageLoader />;
  if (error)
    return (
      <div className="p-7">
        <ErrorState
          message={error}
          onRetry={() => {
            refreshAdapters();
            refreshFw();
          }}
        />
      </div>
    );

  const fwSummary = summarizeFirewall(firewall ?? []);
  const connectOk = pingResults ? pingResults.some((r) => r.success) : null;

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader title="Réseau" subtitle="Adaptateurs, pare-feu et connectivité" />

      <div className="grid grid-cols-2 gap-4 mb-6">
        <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
          <p className="text-xs text-fg-muted mb-1">Pare-feu</p>
          <p className={`text-2xl font-bold ${fwSummary.className}`}>{fwSummary.label}</p>
          <ul className="mt-2 space-y-1">
            {firewall?.map((p) => {
              const state = FIREWALL_STATES[String(p.isEnabled)];
              const Icon = state.icon;
              return (
                <li key={p.name} className="flex items-center gap-2 text-xs">
                  <Icon size={14} className={state.className} aria-hidden="true" />
                  <span className="sr-only">{`${p.name} : ${state.label}`}</span>
                  <span className="text-fg-dim" aria-hidden="true">{p.name}</span>
                </li>
              );
            })}
          </ul>
        </div>

        <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
          <p className="text-xs text-fg-muted mb-1">Connectivité</p>
          {connectOk === null ? (
            <p className="text-2xl font-bold text-fg-dim">--</p>
          ) : connectOk ? (
            <p className="text-2xl font-bold text-success">OK</p>
          ) : (
            <p className="text-2xl font-bold text-danger">Échec</p>
          )}
          <ActionButton
            variant="accent"
            loading={testing}
            onClick={handleTestConnectivity}
            className="mt-3 px-3 py-1.5 text-xs"
          >
            {testing ? "Test..." : "Tester"}
          </ActionButton>
          {pingResults && (
            <div className="mt-2 space-y-1">
              {pingResults.map((r) => (
                <div key={r.host} className="flex items-center gap-2 text-xs">
                  <PingIcon success={r.success} host={r.host} />
                  <span className="text-fg-dim">{r.host}</span>
                  {r.success && <span className="text-fg-muted">{r.roundtripMs}ms</span>}
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
          <p className="text-xs text-fg-muted mb-1">IP publique</p>
          {ipError ? (
            <UnavailableValue message={ipError} onRetry={handleShowPublicIp} retryAriaLabel="Réessayer l'IP publique" />
          ) : publicIp ? (
            <p className="text-2xl font-bold text-fg-primary">{publicIp}</p>
          ) : (
            <>
              <p className="text-2xl font-bold text-fg-dim">--</p>
              <ActionButton
                variant="accent"
                loading={ipLoading}
                onClick={handleShowPublicIp}
                aria-label="Afficher l'IP publique"
                className="mt-3 px-3 py-1.5 text-xs"
              >
                {ipLoading ? "Lecture..." : "Afficher"}
              </ActionButton>
              <p className="mt-2 text-xs text-fg-muted">Demandée à api.ipify.org</p>
            </>
          )}
        </div>

        <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
          <p className="text-xs text-fg-muted mb-1">DNS</p>
          <p className="text-sm text-fg-dim mb-3">Vider le cache de résolution DNS</p>
          <ActionButton
            variant="accent"
            loading={flushing}
            onClick={handleFlushDns}
            className="px-3 py-1.5 text-xs"
          >
            {flushing ? "Vidage..." : "Vider le cache DNS"}
          </ActionButton>
        </div>
      </div>

      <div className="bg-bg-card border border-border rounded-lg p-5 mb-6 theme-card">
        <p className="text-xs text-fg-muted mb-2">Test ping personnalisé</p>
        <div className="flex gap-2 flex-wrap">
          <input
            type="text"
            aria-label="Hôte ou adresse IP à pinguer"
            placeholder="Hôte ou adresse IP (ex. : google.com)"
            value={pingHostInput}
            onChange={(e) => setPingHostInput(e.target.value)}
            onKeyDown={(e) => { if (e.key === "Enter") void handlePingHost(); }}
            className="flex-1 min-w-[200px] px-4 py-2 bg-bg-input border border-border rounded-md text-sm text-fg-primary placeholder:text-fg-muted focus:outline-none focus:border-accent focus-visible:ring-2 focus-visible:ring-accent"
          />
          <ActionButton
            variant="accent"
            loading={pingingHost}
            onClick={handlePingHost}
            disabled={!pingHostInput.trim()}
          >
            {pingingHost ? "Ping..." : "Ping"}
          </ActionButton>
        </div>
        {customPing && (
          <div className="mt-3 flex items-center gap-2 text-xs">
            <PingIcon success={customPing.success} host={customPing.host} />
            <span className="text-fg-dim">{customPing.host}</span>
            {customPing.success ? (
              <span className="text-fg-muted">{customPing.roundtripMs}ms</span>
            ) : (
              <span className="text-fg-muted">{customPing.status}</span>
            )}
          </div>
        )}
      </div>

      <h2 className="text-lg font-medium text-fg-primary mb-3">Adaptateurs réseau</h2>

      <div className="space-y-3">
        {adapters?.map((adapter) => (
          <div key={`${adapter.name}-${adapter.macAddress || adapter.ipAddress}`} className="bg-bg-card border border-border rounded-lg p-5 theme-card">
            <div className="flex items-center gap-3 mb-2 flex-wrap">
              <p className="text-sm font-medium text-fg-primary">{adapter.name}</p>
              <Badge variant={adapter.status === "Up" || adapter.status === "Connected" ? "success" : "danger"}>
                {getAdapterStatusLabel(adapter.status)}
              </Badge>
              <Badge>{adapter.adapterType}</Badge>
            </div>
            <div className="grid grid-cols-2 gap-x-6 gap-y-1 text-xs">
              <div>
                <span className="text-fg-muted">IP : </span><span className="text-fg-dim">{adapter.ipAddress || "—"}</span>
                {adapter.subnetMask && <span className="text-fg-dim"> / {adapter.subnetMask}</span>}
              </div>
              <div><span className="text-fg-muted">MAC : </span><span className="text-fg-dim">{adapter.macAddress || "—"}</span></div>
              <div><span className="text-fg-muted">Passerelle : </span><span className="text-fg-dim">{adapter.gateway || "—"}</span></div>
              <div><span className="text-fg-muted">Vitesse : </span><span className="text-fg-dim">{adapter.speed || "—"}</span></div>
              <div className="col-span-2"><span className="text-fg-muted">DNS : </span><span className="text-fg-dim">{adapter.dnsServers || "—"}</span></div>
            </div>
          </div>
        ))}

        {(!adapters || adapters.length === 0) && (
          <EmptyState
            title="Aucun adaptateur réseau détecté"
            hint="Aucun adaptateur physique ou virtuel actif. Vérifier le gestionnaire de périphériques Windows."
          />
        )}
      </div>
    </div>
  );
}

export default NetworkPage;
