import { Component, type ReactNode } from "react";
import { reportFrontendError } from "../../lib/frontend-log";

interface Props {
  pageName: string;
  children: ReactNode;
  onReset?: () => void;
}

interface State {
  error: Error | null;
}

export class PageErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error) {
    // Les ErrorBoundary React ne capturent pas les erreurs asynchrones (invoke) :
    // celles-ci passent par try/catch dans chaque page. Console + journal Logs.
    console.error(`[PageErrorBoundary:${this.props.pageName}]`, error);
    reportFrontendError(`PageErrorBoundary:${this.props.pageName}`, error);
  }

  // Note : `componentDidUpdate` pour reset le state sur changement de pageName a
  // ete retire car App.tsx utilise `key={currentPage}` sur PageErrorBoundary, ce
  // qui force React a unmount/remount le composant a chaque changement de page.
  // Le reset du state est donc naturel via le lifecycle React.

  handleReset = () => {
    this.setState({ error: null });
    this.props.onReset?.();
  };

  render() {
    if (this.state.error) {
      return (
        <div
          role="alert"
          className="flex flex-col items-center justify-center h-full p-12 text-center"
        >
          <div className="max-w-md">
            <h2 className="text-lg font-semibold text-fg-primary mb-2">
              Cette section n'a pas pu charger
            </h2>
            <p className="text-sm text-fg-dim mb-4">
              Une erreur est survenue en affichant la page{" "}
              <span className="font-mono text-fg-muted">
                {this.props.pageName}
              </span>
              . Le reste de l'application reste utilisable.
            </p>
            <pre className="bg-bg-secondary border border-border rounded-md text-xs text-fg-dim font-mono p-3 mb-4 text-left whitespace-pre-wrap break-words max-h-40 overflow-auto">
              {this.state.error.message}
            </pre>
            <div className="flex gap-3 justify-center">
              <button
                type="button"
                onClick={this.handleReset}
                className="px-4 py-2 text-sm bg-accent text-on-accent rounded-md hover:bg-accent-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
              >
                Réessayer
              </button>
              <span className="text-xs text-fg-dim self-center">
                ou consulter les logs dans la page Logs
              </span>
            </div>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

export default PageErrorBoundary;
