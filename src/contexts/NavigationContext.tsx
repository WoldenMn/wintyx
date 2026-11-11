import { createContext, useContext, type ReactNode } from "react";

type NavigateFn = (pageId: string) => void;

const NavigationContext = createContext<NavigateFn | null>(null);

export function NavigationProvider({ navigate, children }: { navigate: NavigateFn; children: ReactNode }) {
  return <NavigationContext.Provider value={navigate}>{children}</NavigationContext.Provider>;
}

const NOOP: NavigateFn = () => {};

/**
 * Navigation tolerante : retourne un no-op quand aucun NavigationProvider n'est
 * present (tests isoles, composants montes hors App). Permet a ErrorState de
 * proposer un lien "Diagnostiquer" sans prop-drilling sur 14 pages.
 */
export function useNavigateOptional(): NavigateFn {
  return useContext(NavigationContext) ?? NOOP;
}

/**
 * Indique si un NavigationProvider reel est present. Permet a ErrorState de
 * n'afficher le lien "Diagnostiquer" que quand la navigation est branchee
 * (evite un lien mort dans les tests isoles / montages hors App).
 */
export function useHasNavigation(): boolean {
  return useContext(NavigationContext) !== null;
}
