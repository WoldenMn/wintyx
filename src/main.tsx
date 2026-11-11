import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { RootErrorBoundary } from "./components/ui/RootErrorBoundary";
import { PERF_ENABLED } from "./lib/perf-collector";
import "./index.css";

// Perf instrumentation : T0 = main.tsx exec, jalon de reference pour les
// mesures du temps de demarrage (cf. logs [perf] dans App + DashboardPage +
// ScoresContext). Zero cout perf (1 mark + 1 console.info).
performance.mark("wintyx-app-init");
if (PERF_ENABLED) {
  console.info("[perf] T0 main.tsx exec");
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <RootErrorBoundary>
      <App />
    </RootErrorBoundary>
  </React.StrictMode>,
);
