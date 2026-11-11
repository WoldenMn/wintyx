import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { PAGES, SIDEBAR_GROUPS, getPage, type PageDef } from "../../lib/pages";
import { ThemeSwitcher } from "../ui/ThemeSwitcher";
import { SUSPICIOUS_TASKS_CHANGED_EVENT } from "../../lib/suspicious-tasks";
import { useScores } from "../../contexts/ScoresContext";
import {
  SIDEBAR_BADGE_HARDENING_THRESHOLD,
  SIDEBAR_BADGE_SERVICES_THRESHOLD,
} from "../../lib/constants";

interface SidebarProps {
  currentPage: string;
  onNavigate: (page: string) => void;
}

interface SidebarBadge {
  label: string;
  variant: "danger" | "warning";
  /** Texte lecteur d'ecran quand le label visuel ("!") ne se suffit pas. */
  srLabel?: string;
}

const SUSPICIOUS_REFRESH_MS = 60000;

export default function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  const dashboard = getPage("dashboard")!;
  const diagnostic = getPage("diagnostic")!;
  const help = getPage("help")!;
  const about = getPage("about")!;
  const [suspiciousCount, setSuspiciousCount] = useState<number | null>(0);
  const { scores, loaded: scoresLoaded, scoresError } = useScores();
  // Pas de badge tant que les scores ne sont pas charges (ni en cas d'erreur :
  // des scores a 0 par defaut declencheraient de fausses alertes).
  const scoresReady = scoresLoaded && !scoresError;

  useEffect(() => {
    let mounted = true;
    let inFlight = false;
    let rerun = false;
    const load = (force: boolean) => {
      if (inFlight) {
        if (force) rerun = true;
        return;
      }
      inFlight = true;
      invoke<number>("get_suspicious_task_count")
        .then((n) => { if (mounted) setSuspiciousCount(n); })
        .catch(() => { if (mounted) setSuspiciousCount(null); })
        .finally(() => {
          inFlight = false;
          if (rerun && mounted) {
            rerun = false;
            load(false);
          }
        });
    };
    const onChanged = () => load(true);
    const onVisibility = () => {
      if (!document.hidden) load(false);
    };
    load(false);
    const id = window.setInterval(() => {
      if (!document.hidden) load(false);
    }, SUSPICIOUS_REFRESH_MS);
    window.addEventListener(SUSPICIOUS_TASKS_CHANGED_EVENT, onChanged);
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      mounted = false;
      window.clearInterval(id);
      window.removeEventListener(SUSPICIOUS_TASKS_CHANGED_EVENT, onChanged);
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, []);

  const badgeFor = (id: string): SidebarBadge | null => {
    if (id === "tasks" && suspiciousCount === null) {
      return { label: "?", variant: "warning", srLabel: "Tâches suspectes : vérification impossible" };
    }
    if (id === "tasks" && suspiciousCount !== null && suspiciousCount > 0) {
      return { label: String(suspiciousCount), variant: "danger" };
    }
    if (id === "hardening" && scoresReady && scores.hardening < SIDEBAR_BADGE_HARDENING_THRESHOLD) {
      return { label: "!", variant: "warning", srLabel: "Score durcissement faible" };
    }
    if (id === "services" && scoresReady && scores.services < SIDEBAR_BADGE_SERVICES_THRESHOLD) {
      return { label: "!", variant: "danger", srLabel: "Score services faible" };
    }
    return null;
  };

  return (
    <aside className="flex flex-col w-52 min-w-52 bg-bg-sidebar border-r border-border h-full overflow-y-auto">
      <nav aria-label="Navigation principale" className="flex flex-col flex-1 min-h-0">
        <div className="p-2">
          <NavButton
            item={dashboard}
            isActive={currentPage === dashboard.id}
            onClick={() => onNavigate(dashboard.id)}
            badge={badgeFor(dashboard.id)}
          />
        </div>

        <div className="sidebar-scroll flex-1 overflow-y-auto px-2 pb-2" data-testid="sidebar-scroll">
          {SIDEBAR_GROUPS.map((group) => (
            <div key={group} className="mt-2.5" role="group" aria-label={group}>
              <span className="px-3 text-xs font-medium tracking-wide text-fg-muted" aria-hidden="true">
                {group}
              </span>
              <div className="mt-1 flex flex-col gap-0.5">
                {PAGES.filter((p) => p.group === group).map((item) => (
                  <NavButton
                    key={item.id}
                    item={item}
                    isActive={currentPage === item.id}
                    onClick={() => onNavigate(item.id)}
                    badge={badgeFor(item.id)}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
      </nav>

      <div className="p-2 border-t border-divider">
        <ThemeSwitcher />
        {/* Groupe "Autres" (hors SIDEBAR_GROUPS) ancre en bas : rend Aide et A propos
            visibles et surlignables quand on y navigue, y compris via Ctrl+K (run-21). */}
        <NavButton
          item={diagnostic}
          isActive={currentPage === diagnostic.id}
          onClick={() => onNavigate(diagnostic.id)}
          badge={null}
        />
        <NavButton
          item={help}
          isActive={currentPage === help.id}
          onClick={() => onNavigate(help.id)}
          badge={null}
        />
        <NavButton
          item={about}
          isActive={currentPage === about.id}
          onClick={() => onNavigate(about.id)}
          badge={null}
        />
      </div>
    </aside>
  );
}

function NavButton({
  item,
  isActive,
  onClick,
  badge,
}: {
  item: PageDef;
  isActive: boolean;
  onClick: () => void;
  badge: SidebarBadge | null;
}) {
  const badgeClasses =
    badge?.variant === "danger"
      ? "bg-danger text-on-semantic"
      : "bg-warning text-on-semantic";
  return (
    <button
      onClick={onClick}
      aria-current={isActive ? "page" : undefined}
      className={`relative flex items-center gap-2.5 w-full px-3 py-1 rounded-md text-sm text-left transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent ${
        isActive
          ? "bg-bg-elevated text-fg-primary"
          : "text-fg-dim hover:bg-bg-hover hover:text-fg-primary"
      }`}
    >
      {isActive && (
        <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-5 rounded-r bg-accent" />
      )}
      <item.icon size={18} className="shrink-0" aria-hidden="true" />
      <span className="flex-1 truncate">{item.label}</span>
      {badge && (
        <span className={`ml-auto text-xs font-semibold px-1.5 py-0.5 rounded-full ${badgeClasses}`}>
          <span aria-hidden={badge.srLabel ? true : undefined}>{badge.label}</span>
          {badge.srLabel && <span className="sr-only">{badge.srLabel}</span>}
        </span>
      )}
    </button>
  );
}
