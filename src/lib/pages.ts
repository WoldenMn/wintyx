import type { LucideIcon } from "lucide-react";
import {
  LayoutDashboard, SlidersHorizontal, Trash2, Gauge, Power, PackageX,
  ShieldHalf, Server, ShieldCheck, Radar, Activity, HeartPulse, Wifi,
  HardDrive, Cpu, MemoryStick, Download, ListChecks, Archive, GitCompare,
  ScrollText, Timer, Stethoscope, HelpCircle, Info,
} from "lucide-react";

export type PageGroup =
  | "Principal"
  | "Optimisation"
  | "Sécurité"
  | "Système"
  | "Outils"
  | "Autres";

export interface PageDef {
  id: string;
  label: string;
  group: PageGroup;
  keywords: string;
  icon: LucideIcon;
}

export const PAGES: PageDef[] = [
  { id: "dashboard", label: "Tableau de bord", group: "Principal", keywords: "dashboard accueil home", icon: LayoutDashboard },

  { id: "profiles", label: "Profils", group: "Optimisation", keywords: "profil preset bureau gaming performance discretion", icon: SlidersHorizontal },
  { id: "cleanup", label: "Nettoyage", group: "Optimisation", keywords: "temp cache corbeille espace", icon: Trash2 },
  { id: "performance", label: "Performance", group: "Optimisation", keywords: "optimiser vitesse", icon: Gauge },
  { id: "startup", label: "Démarrage", group: "Optimisation", keywords: "boot programmes lancement", icon: Power },
  { id: "bloatware", label: "Bloatware", group: "Optimisation", keywords: "desinstaller apps preinstallees", icon: PackageX },

  { id: "hardening", label: "Durcissement", group: "Sécurité", keywords: "hardening securite renforcement", icon: ShieldHalf },
  { id: "services", label: "Services", group: "Sécurité", keywords: "windows services arreter", icon: Server },
  { id: "data-protection", label: "Protection données", group: "Sécurité", keywords: "telemetrie vie privee privacy", icon: ShieldCheck },
  { id: "threats", label: "Scanner de menaces", group: "Sécurité", keywords: "scan security risque threat analyse", icon: Radar },

  { id: "monitor", label: "Monitoring live", group: "Système", keywords: "cpu ram live temps reel streaming continu graphe", icon: Activity },
  { id: "health", label: "Santé système", group: "Système", keywords: "cpu ram processus snapshot ponctuel etat sante score instantane", icon: HeartPulse },
  { id: "network", label: "Réseau", group: "Système", keywords: "adaptateurs firewall ping ip", icon: Wifi },
  { id: "disks", label: "Disques", group: "Système", keywords: "stockage espace ssd hdd", icon: HardDrive },
  { id: "drivers", label: "Pilotes", group: "Système", keywords: "drivers signed unsigned", icon: Cpu },
  { id: "memory", label: "Mémoire", group: "Système", keywords: "ram barrettes ddr", icon: MemoryStick },

  { id: "updates", label: "Mises à jour", group: "Outils", keywords: "windows update kb", icon: Download },
  { id: "tasks", label: "Tâches planifiées", group: "Outils", keywords: "scheduler suspicious", icon: ListChecks },
  { id: "backup", label: "Sauvegarde", group: "Outils", keywords: "restauration point registre", icon: Archive },
  { id: "snapshots-compare", label: "Comparer snapshots", group: "Outils", keywords: "snapshot diff comparaison registre changements", icon: GitCompare },
  { id: "logs", label: "Logs", group: "Outils", keywords: "journal evenements debug erreurs", icon: ScrollText },
  { id: "benchmark", label: "Benchmark", group: "Outils", keywords: "performance test score vitesse", icon: Timer },

  { id: "diagnostic", label: "Diagnostic", group: "Autres", keywords: "diagnostic sante wmi powershell prerequis probleme", icon: Stethoscope },
  { id: "help", label: "Aide", group: "Autres", keywords: "aide help documentation guide scores raccourcis", icon: HelpCircle },
  { id: "about", label: "À propos", group: "Autres", keywords: "version info", icon: Info },
];

export const SIDEBAR_GROUPS: PageGroup[] = ["Optimisation", "Sécurité", "Système", "Outils"];

export const PAGE_LABELS: Record<string, string> = Object.fromEntries(
  PAGES.map((p) => [p.id, p.label]),
);

export function getPage(id: string): PageDef | undefined {
  return PAGES.find((p) => p.id === id);
}
