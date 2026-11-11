import { Component, type ErrorInfo, type ReactNode } from "react";
import { reportFrontendError } from "../../lib/frontend-log";

interface State {
  error: Error | null;
}

export class RootErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("[RootErrorBoundary]", error, info.componentStack);
    reportFrontendError("RootErrorBoundary", error);
  }

  render() {
    if (this.state.error) {
      return (
        <div style={{ padding: 40, color: "#ef4444", fontFamily: "monospace", background: "#1a1d25", height: "100vh" }}>
          <h1 style={{ color: "#e8eaed", marginBottom: 12 }}>Erreur Wintyx</h1>
          <pre style={{ whiteSpace: "pre-wrap", fontSize: 13 }}>{this.state.error.message}</pre>
          <pre style={{ whiteSpace: "pre-wrap", fontSize: 11, color: "#9aa0a6", marginTop: 8 }}>{this.state.error.stack}</pre>
        </div>
      );
    }
    return this.props.children;
  }
}

export default RootErrorBoundary;
