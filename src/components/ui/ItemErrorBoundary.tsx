import { Component, type ReactNode } from "react";
import { reportFrontendError } from "../../lib/frontend-log";

interface Props {
  /** Fonction de rendu de l'item. Appelee dans le render du boundary pour que
   *  les erreurs levees DANS renderItem (pas seulement dans l'arbre rendu) soient
   *  capturees par getDerivedStateFromError. */
  render: () => ReactNode;
}

interface State {
  hasError: boolean;
}

// L'appel a render() se fait dans CE composant enfant (pas dans le render() du
// boundary lui-meme) : un error boundary ne capture que les erreurs de ses
// descendants, pas de son propre render. Sans cet intermediaire, un throw dans
// renderItem ne serait pas capture.
function ItemRenderer({ render }: { render: () => ReactNode }) {
  return <>{render()}</>;
}

/**
 * Error boundary compacte pour un seul item de liste : si un renderItem leve une
 * erreur (carte buggee, donnee inattendue), seule cette carte est remplacee par
 * un fallback discret au lieu de demonter toute la liste CrudListPage (fvol-12).
 */
export class ItemErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error) {
    console.error("[ItemErrorBoundary]", error);
    reportFrontendError("ItemErrorBoundary", error);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div
          role="alert"
          className="bg-bg-card border border-danger/30 rounded-lg p-4 text-xs text-danger theme-card"
        >
          Cet élément n'a pas pu s'afficher.
        </div>
      );
    }
    return <ItemRenderer render={this.props.render} />;
  }
}

export default ItemErrorBoundary;
