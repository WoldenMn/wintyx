# Contribuer a Wintyx

Merci de vouloir contribuer a Wintyx. Ce document decrit les conventions,
le flux de contribution et les controles de qualite attendus.

## Prerequis

- **Rust stable** (via [rustup.rs](https://rustup.rs))
- **Bun** (via [bun.sh](https://bun.sh))
- **Visual Studio Build Tools** avec workload "Developpement Desktop en C++"
- **Windows 10 ou 11** 64-bit

Consulter [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) pour une installation pas-a-pas
et la procedure d'ajout d'une nouvelle page.

## Flux de contribution

1. Creer une branche depuis `main` : `git checkout -b feat/nom-court`
2. Implementer la modification (code + tests)
3. Executer la boucle de verification (voir section `Verification`) jusqu'a zero erreur
4. Mettre a jour la documentation impactee (`README.md`, `CHANGELOG.md`, `docs/`, `CLAUDE.md`)
5. Committer avec un message conventionnel (voir section `Messages de commit`)
6. Pousser la branche et ouvrir une pull request

### Types de branche

| Prefixe | Usage |
|---------|-------|
| `feat/` | Nouvelle fonctionnalite |
| `fix/` | Correction de bug |
| `refactor/` | Refonte interne sans changement de comportement |
| `docs/` | Mise a jour documentation seulement |
| `test/` | Ajout ou correction de tests |
| `chore/` | Maintenance (deps, CI, build) |

## Messages de commit

Format Conventional Commits :

```
<type>(<scope>): <description courte>

<corps optionnel — pourquoi, pas comment>
```

Exemples :

```
feat(monitor): ajouter metrique cpu_percent instantane
fix(backup): remonter stderr et exit_code au frontend
refactor(benchmark): extraire compute_overall_score pour tests
docs(readme): mettre a jour compteurs de tests v2.0.3
test(stat-card): ajouter 11 tests composant StatCard
chore(ci): activer cargo clippy en mode strict
```

### Regles

- Titre < 72 caracteres, imperatif present ("ajouter" pas "ajoute")
- Un seul sujet par commit (un fix = un commit)
- Le corps explique le **pourquoi**, pas le **quoi** (le diff montre le quoi)
- Le corps parle du changement, pas de l'outil qui l'a ecrit ; un co-auteur (humain ou assistant) se credite par une ligne `Co-Authored-By:` en fin de message

## Conventions de code

### Backend Rust

- `#[serde(rename_all = "camelCase")]` obligatoire sur tous les structs serialises
- Magic numbers et timeouts centralises dans `utils/constants.rs`
- Inputs utilisateur sanitises via `registry::sanitize_name()` avant interpolation
- PowerShell : `Get-CimInstance` (jamais `Get-WmiObject`)
- Escaping PowerShell via `process::escape_powershell()`
- Parsing des sorties pipe-separated via `process::parse_pipe_fields()`
- Chemins systeme via `std::env::var()` (`SYSTEMDRIVE`, `LOCALAPPDATA`), jamais de `C:\` en dur
- Les commandes Tauri qui peuvent echouer renvoient `Result<T, CommandError>`, propagation d'erreur avec `?`
- Une commande qui touche WMI, PowerShell, le registre ou le disque est `async` et confie son corps a `utils/blocking.rs` (`spawn_blocking`) : une commande synchrone gele la fenetre
- Batch operations : collect-errors (ne pas s'arreter au premier echec, rapporter tout)
- `CREATE_NO_WINDOW` (`constants::CREATE_NO_WINDOW`) sur chaque `Command::creation_flags`
- Actions privilegiees (HKLM, sc stop/start, restore points) : ajouter `require_admin("feature")` en tete
- Tests unitaires dans un module `#[cfg(test)] mod tests` **a la fin** du fichier
  (clippy::items_after_test_module). Extraire la logique pure en `pub(crate) fn` pour la tester.
  Au-dela de 300 lignes de fichier, les tests vont dans un `tests.rs` voisin (`#[cfg(test)] mod tests;`).

### Frontend TypeScript

- Types dans `lib/types.ts` uniquement (miroir exact des structs Rust, camelCase)
- Helpers partages dans `lib/helpers.ts`
- Constantes dans `lib/constants.ts`
- Pas d'emojis ni d'icones Unicode dans l'interface (texte pur)
- React 19 : `useDeferredValue` pour le filtrage reactif sur listes >100 items
- Context values memoisees avec `useMemo` pour eviter les re-renders en cascade
- Cleanup des effets : `mountedRef` + guard sur `setState` apres unmount

### Accessibilite (WCAG 2.1 AA)

Chaque nouveau composant interactif doit respecter :

- Role ARIA semantique (`role="dialog"`, `role="region"`, `role="alert"`, etc.)
- `aria-label` ou `aria-labelledby` sur les regions sans texte visible
- Focus trap dans les dialogs (`Tab` cycle, focus restore sur fermeture)
- Contraste texte/fond >= 4.5:1 (utiliser les variables `--color-*-contrast`)
- Annonces lecteurs d'ecran : `role="alert" aria-live="assertive"` pour les erreurs
- Tables : `<caption className="sr-only">` + `scope="col"` sur les `<th>`
- Navigation clavier complete (Tab, Shift+Tab, Escape, Enter)
- Icones decoratives marquees `aria-hidden="true"`, icones signifiantes avec `role="img"` + `<title>`

### Nommage

- Rust : `snake_case` pour les champs (serde convertit en camelCase a la serialisation)
- TypeScript : `camelCase` pour les proprietes (miroir de serde)
- Pages : `NomPage.tsx` (PascalCase)
- Commandes Tauri : `verbe_nom` (snake_case)
- Hooks React : `use-nom-hook.ts` ou `use-nom-hook.tsx` (kebab-case)

## Verification

Apres **chaque modification** (pas seulement en fin de tache), executer la boucle
suivante jusqu'a zero erreur :

```bash
# Frontend
bun run lint          # ESLint
bun run test          # Vitest (le compte s'affiche en fin de run)
bun run build         # Build TypeScript + Vite

# Backend Rust
cd src-tauri
cargo clippy --all-targets -- -D warnings
cargo test            # tests Rust (le compte s'affiche en fin de run)
cargo build --release
```

### Construire l'application

Avant une PR qui touche le backend, la configuration Tauri ou les installeurs, verifier que l'application s'assemble :

```bash
bun tauri build
```

Les installeurs sortent dans `src-tauri/target/release/bundle/`. Les versions publiees sont construites par la CI (`.github/workflows/release.yml`).

### Ajout de tests

Tout nouveau code doit etre accompagne de tests :

- **Rust** : extraire la logique pure en `pub(crate) fn`, tester avec `#[test]`
- **TypeScript** : tester les composants avec `@testing-library/react`, les hooks avec
  `renderHook`, les helpers avec `describe/it` Vitest

Objectif de couverture : >= 60% sur le code non-IO (logique pure, helpers, scoring).

## Documentation

Mettre a jour la documentation impactee dans le meme commit :

| Fichier | Quand le mettre a jour |
|---------|------------------------|
| `README.md` | Version, stack, compteurs de tests, nouvelles fonctionnalites majeures |
| `CHANGELOG.md` | A chaque release ou changement notable |
| `CLAUDE.md` | Decisions techniques, pieges rencontres, historique |
| `docs/ARCHITECTURE.md` | Nouveau module backend, nouvelle integration |
| `docs/COMMANDS.md` | Nouvelle commande Tauri ajoutee a `generate_handler!` |
| `docs/DEVELOPMENT.md` | Nouvelle convention, nouvelle procedure dev |
| `docs/USER_GUIDE.md` | Nouvelle page utilisateur, changement UX notable |

## Revue de code

Une PR est prete a merger quand :

- [ ] CI verte (lint + build + tests frontend + clippy + tests Rust)
- [ ] Aucun warning `cargo clippy --all-targets`
- [ ] Aucun warning ESLint
- [ ] Aucune erreur TypeScript (`bun run build` passe)
- [ ] Tests ajoutes pour la nouvelle logique
- [ ] Documentation impactee mise a jour
- [ ] Message de commit conforme
- [ ] `bun tauri build` passe si la PR touche le backend, Tauri ou les installeurs

## Signalement de bugs

Ouvrir une issue avec le modele « Signaler un bug », qui demande :

1. Version de Wintyx (visible dans la page `A propos`)
2. Version de Windows (`winver`)
3. Etapes pour reproduire
4. Resultat attendu vs resultat observe
5. Extrait pertinent des logs (`%LOCALAPPDATA%\Wintyx\Logs\`), relu avant de le coller : il peut contenir des chemins et des noms propres a votre machine

## Contact

Questions, suggestions, retours : ouvrir une issue sur le depot.
