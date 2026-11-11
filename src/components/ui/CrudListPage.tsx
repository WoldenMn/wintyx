import { useEffect, useMemo, useState, type MutableRefObject, type ReactNode } from "react";
import { useInvoke } from "../../hooks/use-invoke";
import type { CommandName } from "../../lib/command-names";
import { PageHeader } from "./PageHeader";
import { PageSkeleton } from "./Skeleton";
import { EmptyState } from "./EmptyState";
import { ErrorState } from "./ErrorState";
import { ItemErrorBoundary } from "./ItemErrorBoundary";
import { VirtualList } from "./VirtualList";

export interface CrudApi<T> {
  refresh: () => void;
  items: T[];
}

export interface CrudListPageProps<T> {
  title: string;
  subtitle?: ReactNode;
  fetchCommand: CommandName;
  fetchArgs?: Record<string, unknown>;
  /** Si fourni, active la search bar. Retourne true si item match la query. */
  filter?: (item: T, query: string) => boolean;
  /** Si fourni, groupe les items par la cle retournee. */
  groupBy?: (item: T) => string;
  /** Ordre des groupes. */
  sortGroups?: (groups: string[]) => string[];
  /** Label affiche pour un groupe (defaut: nom brut de la cle). */
  groupLabel?: (groupKey: string, count: number) => ReactNode;
  /** Rendu d'un item — recoit aussi l'API (refresh, items). */
  renderItem: (item: T, api: CrudApi<T>) => ReactNode;
  /**
   * Actions dans le header (boutons batch par ex).
   * Peut etre un ReactNode statique, ou une fonction recevant l'API
   * (utile pour les boutons "Tout XXX" qui doivent declencher un refresh).
   */
  headerActions?: ReactNode | ((api: CrudApi<T>) => ReactNode);
  /** Etat vide personnalise. */
  emptyState?: { title: string; hint?: ReactNode };
  /** Virtualisation pour les listes longues (typiquement >80 items). */
  virtualize?: { itemHeight: number; threshold?: number };
  /** Variante de skeleton pendant le chargement. */
  pageSkeleton?: "list" | "grid" | "dashboard";
  /** Placeholder custom pour la search bar. */
  searchPlaceholder?: string;
  /**
   * Retourne un ID stable par item pour les keys React.
   * Si non fourni, fallback sur l'index (peut causer des bugs au filtrage).
   * Recommande pour les pages avec animations/halos (v2.1.2+).
   */
  getItemId?: (item: T) => string;
  /**
   * Si fournie, calcule un sous-titre dynamique depuis la liste filtree.
   * Override `subtitle` si les deux sont fournis (subtitleCompute prioritaire).
   * Utile pour afficher un total agrege (ex: "Total : 1.5 GB recuperables").
   * @v2.9.0
   */
  subtitleCompute?: (items: T[]) => ReactNode;
  /**
   * Recoit le `refresh` de la liste : permet a une action declaree hors de la
   * page liste (useToggleAction) de recharger les donnees apres une modification.
   */
  refreshRef?: MutableRefObject<(() => void) | null>;
}

/**
 * Composant generique pour les pages "liste d'items avec actions".
 *
 * Elimine la duplication du pattern `fetch + filter + group + render + empty state`
 * implemente manuellement dans 10 pages CRUD (Services, Startup, Bloatware, etc.).
 *
 * Compose avec `useToggleAction<T>` pour gerer les actions individuelles dans
 * `renderItem`.
 */
export function CrudListPage<T>(props: CrudListPageProps<T>) {
  const { data, isLoading, error, refresh } = useInvoke<T[]>(props.fetchCommand, props.fetchArgs);
  const [query, setQuery] = useState("");
  const { refreshRef } = props;

  useEffect(() => {
    if (!refreshRef) return;
    refreshRef.current = refresh;
    return () => {
      refreshRef.current = null;
    };
  }, [refreshRef, refresh]);

  const items = useMemo(() => data ?? [], [data]);

  // Destructure pour des deps useMemo precises et stables (perf-react-01) :
  // depend des callbacks specifiques, pas de l'objet props entier recree a chaque render.
  const { filter, groupBy, sortGroups } = props;

  const filtered = useMemo(() => {
    if (!filter || !query.trim()) return items;
    return items.filter((item) => filter(item, query));
  }, [items, query, filter]);

  const grouped = useMemo(() => {
    if (!groupBy) return null;
    const map = new Map<string, T[]>();
    for (const item of filtered) {
      const key = groupBy(item);
      const arr = map.get(key) ?? [];
      arr.push(item);
      map.set(key, arr);
    }
    let keys = Array.from(map.keys());
    if (sortGroups) keys = sortGroups(keys);
    return keys.map((k) => ({ key: k, items: map.get(k)! }));
  }, [filtered, groupBy, sortGroups]);

  const api: CrudApi<T> = { refresh, items: filtered };

  if (isLoading) {
    return <PageSkeleton kind={props.pageSkeleton ?? "list"} />;
  }

  const isEmpty = filtered.length === 0;
  const headerActionsNode = error
    ? undefined
    : typeof props.headerActions === "function"
      ? props.headerActions(api)
      : props.headerActions;

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title={props.title}
        subtitle={
          props.subtitleCompute ? props.subtitleCompute(filtered) : props.subtitle
        }
        actions={headerActionsNode}
      />

      {props.filter && !error && (
        <div className="mb-6">
          <input
            type="search"
            aria-label={props.searchPlaceholder ?? "Rechercher"}
            placeholder={props.searchPlaceholder ?? "Rechercher..."}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="w-full px-4 py-2 bg-bg-input border border-border rounded-md text-sm text-fg-primary placeholder:text-fg-muted focus:outline-none focus:border-accent focus-visible:ring-2 focus-visible:ring-accent"
          />
        </div>
      )}

      {error ? (
        <ErrorState message={error} onRetry={refresh} />
      ) : isEmpty ? (
        <EmptyState
          title={props.emptyState?.title ?? "Aucun élément"}
          hint={props.emptyState?.hint}
        />
      ) : grouped ? (
        grouped.map(({ key, items: groupItems }) => (
          <section key={key} className="mb-6">
            <h2 className="text-sm font-medium text-fg-muted uppercase tracking-wider mb-3">
              {props.groupLabel ? props.groupLabel(key, groupItems.length) : `${key} (${groupItems.length})`}
            </h2>
            <div className="space-y-3">
              {groupItems.map((item, i) => {
                const itemKey = props.getItemId ? `${key}-${props.getItemId(item)}` : `${key}-${i}`;
                return (
                  <div key={itemKey}>
                    <ItemErrorBoundary render={() => props.renderItem(item, api)} />
                  </div>
                );
              })}
            </div>
          </section>
        ))
      ) : props.virtualize ? (
        <VirtualList
          items={filtered}
          itemHeight={props.virtualize.itemHeight}
          threshold={props.virtualize.threshold}
          render={(item) => <ItemErrorBoundary render={() => props.renderItem(item, api)} />}
        />
      ) : (
        <div className="space-y-3">
          {filtered.map((item, i) => {
            const itemKey = props.getItemId ? props.getItemId(item) : String(i);
            return (
              <div key={itemKey}>
                <ItemErrorBoundary render={() => props.renderItem(item, api)} />
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

export default CrudListPage;
