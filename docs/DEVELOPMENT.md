# Guide de developpement

## Prerequis

- **Rust** : [rustup.rs](https://rustup.rs)
- **Bun** : [bun.sh](https://bun.sh)
- **Visual Studio Build Tools** : workload C++ (pour compiler Tauri)
- **Windows 10/11** 64-bit

## Installation

```bash
git clone https://github.com/WoldenMn/wintyx.git
cd wintyx
bun install
```

## Commandes

| Commande | Description |
|----------|-------------|
| `bun tauri dev` | Dev mode avec hot reload (frontend + backend) |
| `bun tauri build` | Build release (exe + installeur NSIS + MSI) |
| `bun run build` | Build frontend seul (dist/) |
| `bun run dev` | Dev frontend seul (sans Tauri) |
| `cd src-tauri && cargo build` | Build backend seul |
| `bun run lint` | Lint TypeScript (ESLint) |
| `cd src-tauri && cargo clippy` | Lint Rust |
| `bun run test` | Tests frontend (Vitest) |
| `cd src-tauri && cargo test` | Tests Rust |

## Ajouter une nouvelle page

### 1. Backend (Rust)

Creer `src-tauri/src/commands/mon_module.rs` :
```rust
use serde::Serialize;
use crate::utils::errors::CommandError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonData {
    pub mon_champ: String,
    pub est_actif: bool,
}

pub fn get_mon_data_sync() -> Result<MonData, CommandError> {
    Ok(MonData { mon_champ: "valeur".into(), est_actif: true })
}
```

Ajouter dans `commands/mod.rs` :
```rust
pub mod mon_module;
```

Annoter la struct de retour ET la commande pour le codegen :
```rust
#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MonData { mon_champ: String, est_actif: bool }

#[tauri::command]
#[specta::specta]
pub async fn get_mon_data() -> Result<MonData, CommandError> {
    crate::utils::blocking::read_result(get_mon_data_sync).await
}
```

Une commande qui touche WMI, PowerShell, le registre ou le disque doit etre `async` et deleguer son corps a `utils/blocking.rs` (`read_*` pour une lecture, `write_*` pour une ecriture systeme) : une commande synchrone tourne sur le thread principal et gele la fenetre. Les ecritures prennent le verrou commun, ce qui evite qu'un snapshot capture un etat a moitie modifie.

Ajouter dans `lib.rs` -> `build_specta_builder` -> `collect_commands![...]` :
```rust
commands::mon_module::get_mon_data,
```

### 2. Types TypeScript (generes, ne pas editer a la main)

Les types sont **generes automatiquement** dans `src/lib/bindings.ts` via tauri-specta
(en mode debug au lancement, ou via `cargo run --bin wintyx -- --export-bindings`).
`bindings.ts` est versionne : apres tout changement de struct ou de commande, le regenerer puis le
committer avec le changement Rust. La CI (job `rust`, etape « Bindings a jour ») le regenere et
echoue s'il differe du fichier commite. `src/lib/types.ts` en re-exporte les types (+ alias retro-compat).
Ne PAS y ajouter d'interface manuelle (doublon/divergence avec le codegen) :
```typescript
// Genere automatiquement — exemple de sortie :
export type MonData = { monChamp: string; estActif: boolean };
```

### 3. Frontend (React)

Creer `src/components/pages/MonPage.tsx` :
```tsx
import { useInvoke } from "../../hooks/use-invoke";
import type { MonData } from "../../lib/types";

export function MonPage() {
  const { data, isLoading } = useInvoke<MonData>("get_mon_data");
  if (isLoading) return <div className="p-7 text-fg-dim">Chargement...</div>;
  return (
    <div className="p-7 overflow-y-auto h-full">
      <h1 className="text-2xl font-semibold text-fg-primary mb-1">Ma Page</h1>
      <p className="text-sm text-fg-dim">{data?.monChamp}</p>
    </div>
  );
}
```

Ajouter dans `App.tsx` (import paresseux et table `pages`) :
```tsx
const MonPage = lazy(() => import("./components/pages/MonPage"));
// dans pages: mapage: MonPage
```

Ajouter dans `src/lib/pages.ts`, qui alimente la sidebar, la palette Ctrl+K et les titres :
```tsx
{ id: "mapage", label: "Ma Page", group: "Outils", keywords: "mots cles de recherche", icon: Wrench },
```
`icon` est une icone `lucide-react`. Le groupe doit etre un `PageGroup` ; seuls les groupes de `SIDEBAR_GROUPS` apparaissent dans la sidebar.

## Conventions

### Backend Rust
- `#[serde(rename_all = "camelCase")]` obligatoire sur tous les structs serialises
- Magic numbers et timeouts dans `utils/constants.rs` (jamais de valeurs en dur)
- Inputs utilisateur sanitises via `registry::sanitize_name()` avant interpolation
- PowerShell : `Get-CimInstance` (pas `Get-WmiObject`), escaping via `process::escape_powershell()`
- Parsing sorties pipe-separated via `process::parse_pipe_fields()` (validation bornes)
- Chemins systeme via env vars (`SYSTEMDRIVE`, `LOCALAPPDATA`), jamais `C:\` en dur
- Commandes Tauri : `Result<T, CommandError>` des qu'elles peuvent echouer, propager les erreurs avec `?` ; jamais d'echec avale ni pris pour un resultat vide
- Batch operations : collect-errors (continuer + rapporter les echecs)
- `CREATE_NO_WINDOW` via `constants::CREATE_NO_WINDOW` sur tous les appels process

### Frontend TypeScript
- Types dans `lib/types.ts` uniquement (jamais de types locaux dans les pages)
- Helpers partages dans `lib/helpers.ts`
- Constantes dans `lib/constants.ts`
- Pas d'emojis ni d'icones Unicode dans l'interface

### Nommage
- Rust : `snake_case` pour les champs (serde convertit en camelCase)
- TypeScript : `camelCase` pour les proprietes (miroir de serde)
- Pages : `NomPage.tsx` (PascalCase)
- Commandes Tauri : `verbe_nom` (snake_case)

## Design system

### Couleurs (definies dans src/index.css @theme)

2 themes signature (Obsidian sombre / Daylight clair). Variables principales :
- `bg-bg-primary` : fond principal
- `bg-bg-card` : cartes
- `text-fg-primary` : texte principal
- `text-fg-dim` : texte secondaire
- `bg-accent` : couleur d'accent
- `bg-success/bg-warning/bg-danger` : statuts

### Classes Tailwind courantes

```
Card :     bg-bg-card border border-border rounded-lg p-5 theme-card
Badge :    px-3 py-1 rounded-full text-xs bg-success/10 text-success
Bouton :   px-4 py-2 bg-accent text-white rounded-md hover:bg-accent-hover text-sm
```

### Boutons par type d'action
- Rouge (`bg-danger`) : actions destructrices (supprimer, nettoyer)
- Bleu (`bg-accent`) : actions primaires (scanner, tester, appliquer)
- Gris (`bg-bg-card border`) : actions secondaires (rafraichir, rapport)

## Tests

### Frontend (Vitest)

```bash
bun run test
```

Les tests Vitest (le compte s'affiche en fin de run) couvrent : helpers, hooks (`useInvoke`, `useToggleAction`, `useConfirmBatchAction`),
contexts (`ScoresContext`, `MonitorContext`, `AdminContext`), composants UI (Toast, ConfirmDialog,
Sparkline, StatCard, Skeleton, ScoreBreakdown, CrudListPage, ErrorState, ItemErrorBoundary, etc.) et pages CRUD migrees.

Config dans `vite.config.ts` (section `test`), setup dans `src/test/setup.ts`.

### Backend (cargo test)

```bash
cd src-tauri && cargo test
```

Les tests Rust (modules `#[cfg(test)]` et un test d'integration sur les capabilities de fenetre, `src-tauri/tests/`) portent sur les helpers purs extraits depuis les commandes Tauri
(scoring par domaine, parsing reseau, formatting cleanup, classify_update, restore_type_label, etc.)
et utils (`sanitize_name`, `parse_pipe_fields`, `escape_powershell`, `kb_to_gb_rounded`,
`require_admin`, superviseur du monitoring).

## Mesurer les performances

L'instrumentation est coupee par defaut. Pour une session de mesure, dans PowerShell :

```powershell
$env:VITE_PERF = "1"
bun tauri dev --features perf
```

Parcourir les pages a mesurer, fermer l'application, puis :

```bash
bun run perf:report   # lit %LOCALAPPDATA%\Wintyx\Logs\perf-*, ecrit audits/perf-baseline-<date>.md
bun run perf:clear    # efface les mesures avant une nouvelle session
```

Le dossier `audits/` est ignore par git : les rapports restent sur la machine qui mesure.

## CI/CD

GitHub Actions (`.github/workflows/ci.yml`) sur push/PR vers `main` :
1. Lint TypeScript (`bun run lint`)
2. Build frontend (`bun run build`)
3. Tests frontend (`bun run test`)
4. Clippy Rust (`cargo clippy -- -D warnings`)
5. Tests Rust (`cargo test`)
6. Controle de derive de `src/lib/bindings.ts` et `cargo audit`

Build Tauri complet exclu du CI (trop lent).
