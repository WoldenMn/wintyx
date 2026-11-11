import { useState, useEffect, useMemo, lazy, Suspense } from "react";
import { prefetchCommand } from "./hooks/use-invoke";
import type { CommandName } from "./lib/command-names";
import { PERF_ENABLED } from "./lib/perf-collector";
import TitleBar from "./components/layout/TitleBar";
import Sidebar from "./components/layout/Sidebar";
import StatusBar from "./components/layout/StatusBar";
import { ToastProvider } from "./components/ui/Toast";
import { ConfirmProvider } from "./components/ui/ConfirmDialog";
import { CommandPalette } from "./components/ui/CommandPalette";
import { PageLoader } from "./components/ui/PageLoader";
import { PageErrorBoundary } from "./components/ui/PageErrorBoundary";
import { ThemeProvider } from "./components/ui/ThemeProvider";
import { MonitorProvider } from "./contexts/MonitorContext";
import { ScoresProvider, useScores } from "./contexts/ScoresContext";
import { AdminProvider } from "./contexts/AdminContext";
import { DiagnosticProvider, useDiagnosticsOptional } from "./contexts/DiagnosticContext";
import { NavigationProvider } from "./contexts/NavigationContext";
import { DiagnosticBanner } from "./components/ui/DiagnosticBanner";
import { PAGE_LABELS } from "./lib/pages";

const DashboardPage = lazy(() => import("./components/pages/DashboardPage"));
const HealthPage = lazy(() => import("./components/pages/HealthPage"));
const DataProtectionPage = lazy(() => import("./components/pages/DataProtectionPage"));
const HardeningPage = lazy(() => import("./components/pages/HardeningPage"));
const ServicesPage = lazy(() => import("./components/pages/ServicesPage"));
const PerformancePage = lazy(() => import("./components/pages/PerformancePage"));
const CleanupPage = lazy(() => import("./components/pages/CleanupPage"));
const BloatwarePage = lazy(() => import("./components/pages/BloatwarePage"));
const StartupPage = lazy(() => import("./components/pages/StartupPage"));
const NetworkPage = lazy(() => import("./components/pages/NetworkPage"));
const DisksPage = lazy(() => import("./components/pages/DisksPage"));
const DriversPage = lazy(() => import("./components/pages/DriversPage"));
const MemoryPage = lazy(() => import("./components/pages/MemoryPage"));
const UpdatesPage = lazy(() => import("./components/pages/UpdatesPage"));
const TasksPage = lazy(() => import("./components/pages/TasksPage"));
const BackupPage = lazy(() => import("./components/pages/BackupPage"));
const AboutPage = lazy(() => import("./components/pages/AboutPage"));
const HelpPage = lazy(() => import("./components/pages/HelpPage"));
const LogsPage = lazy(() => import("./components/pages/LogsPage"));
const MonitorPage = lazy(() => import("./components/pages/MonitorPage"));
const ThreatScanPage = lazy(() => import("./components/pages/ThreatScanPage"));
const BenchmarkPage = lazy(() => import("./components/pages/BenchmarkPage"));
const ProfilesPage = lazy(() => import("./components/pages/ProfilesPage"));
const SnapshotComparePage = lazy(() => import("./components/pages/SnapshotComparePage"));
const DiagnosticPage = lazy(() => import("./components/pages/DiagnosticPage"));

const pages: Record<string, React.FC> = {
  dashboard: DashboardPage,
  health: HealthPage,
  "data-protection": DataProtectionPage,
  hardening: HardeningPage,
  services: ServicesPage,
  performance: PerformancePage,
  cleanup: CleanupPage,
  bloatware: BloatwarePage,
  startup: StartupPage,
  network: NetworkPage,
  disks: DisksPage,
  drivers: DriversPage,
  memory: MemoryPage,
  updates: UpdatesPage,
  tasks: TasksPage,
  backup: BackupPage,
  logs: LogsPage,
  monitor: MonitorPage,
  threats: ThreatScanPage,
  benchmark: BenchmarkPage,
  profiles: ProfilesPage,
  "snapshots-compare": SnapshotComparePage,
  diagnostic: DiagnosticPage,
  help: HelpPage,
  about: AboutPage,
};

const PREFETCH_COMMANDS: Array<{ command: CommandName; args?: Record<string, unknown> }> = [
  { command: "get_data_protection_items" },
  { command: "get_hardening_checks" },
  { command: "get_services" },
];
const PREFETCH_DELAY_MS = 800;

function App() {
  const [currentPage, setCurrentPage] = useState("dashboard");
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);

  useEffect(() => {
    // Perf : App mont apres React render. Mesure depuis main.tsx T0.
    try {
      performance.mark("wintyx-app-mounted");
      performance.measure("wintyx-react-mount", "wintyx-app-init", "wintyx-app-mounted");
      const entries = performance.getEntriesByName("wintyx-react-mount");
      if (entries[0] && PERF_ENABLED) {
        console.info(`[perf] T1 React mount = ${Math.round(entries[0].duration)}ms (App.tsx useEffect)`);
      }
    } catch {
      // Marks indisponibles (rare), ignore silencieusement.
    }

    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        setCommandPaletteOpen((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  useEffect(() => {
    // Prefetch apres un court delai pour laisser la page courante finir de charger
    const timer = setTimeout(() => {
      for (const { command, args } of PREFETCH_COMMANDS) {
        prefetchCommand(command, args);
      }
    }, PREFETCH_DELAY_MS);
    return () => clearTimeout(timer);
  }, []);

  const PageComponent = pages[currentPage];

  return (
    <ThemeProvider>
      <ToastProvider>
        <ConfirmProvider>
          <AdminProvider>
            <DiagnosticProvider>
              <ScoresProvider>
                <MonitorProvider>
                  <AppShell
                    currentPage={currentPage}
                    setCurrentPage={setCurrentPage}
                    commandPaletteOpen={commandPaletteOpen}
                    setCommandPaletteOpen={setCommandPaletteOpen}
                    PageComponent={PageComponent}
                  />
                </MonitorProvider>
              </ScoresProvider>
            </DiagnosticProvider>
          </AdminProvider>
        </ConfirmProvider>
      </ToastProvider>
    </ThemeProvider>
  );
}

function AppShell({
  currentPage, setCurrentPage,
  commandPaletteOpen, setCommandPaletteOpen, PageComponent,
}: {
  currentPage: string; setCurrentPage: (p: string) => void;
  commandPaletteOpen: boolean; setCommandPaletteOpen: (v: boolean) => void;
  PageComponent?: React.FC<{ onNavigate?: (page: string) => void }>;
}) {
  const { globalScore, loaded: scoresLoaded, scoresError } = useScores();
  const { diagnostics } = useDiagnosticsOptional();

  const statusMessage = useMemo(() => {
    if (!scoresLoaded) return "Scan initial...";
    return PAGE_LABELS[currentPage] ?? currentPage;
  }, [currentPage, scoresLoaded]);

  return (
    <NavigationProvider navigate={setCurrentPage}>
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:top-2 focus:left-2 z-50 bg-accent text-on-accent rounded shadow-lg font-medium"
      >
        Aller au contenu principal
      </a>
      <div className="flex flex-col h-screen w-screen bg-bg-primary text-fg-primary overflow-hidden"
           style={{ fontFamily: "var(--t-font)" }}>
        <TitleBar currentPage={PAGE_LABELS[currentPage] ?? currentPage} />
        <DiagnosticBanner diagnostics={diagnostics} onDiagnose={() => setCurrentPage("diagnostic")} />
        <div className="flex flex-1 overflow-hidden">
          <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
          <main
            id="main-content"
            role="main"
            aria-label={PAGE_LABELS[currentPage] ?? currentPage}
            tabIndex={-1}
            className="flex-1 overflow-y-auto focus:outline-none"
          >
            <PageErrorBoundary
              key={currentPage}
              pageName={PAGE_LABELS[currentPage] ?? currentPage}
            >
              <div className="h-full animate-page-in">
                <Suspense fallback={<PageLoader />}>
                  {PageComponent ? <PageComponent onNavigate={setCurrentPage} /> : <FallbackPage />}
                </Suspense>
              </div>
            </PageErrorBoundary>
          </main>
        </div>
        <StatusBar healthScore={scoresLoaded ? globalScore : null} scoreError={Boolean(scoresError)} statusMessage={statusMessage} />
      </div>
      <CommandPalette
        open={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        onNavigate={setCurrentPage}
      />
    </NavigationProvider>
  );
}

function FallbackPage() {
  return (
    <div className="flex items-center justify-center h-full">
      <p className="text-fg-dim text-sm">Page non trouvée</p>
    </div>
  );
}

export default App;
