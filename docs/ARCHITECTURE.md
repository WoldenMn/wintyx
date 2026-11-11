# Architecture Wintyx v2

## Vue d'ensemble

```
+----------------------------------------------+
|                  Tauri v2                     |
|  +-----------------+  +------------------+   |
|  |   Frontend       |  |   Backend Rust   |   |
|  |   React 19 + TS  |  |                  |   |
|  |                  |  |  commands/       |   |
|  |  pages/          |<-|  + system_info   |   |
|  |  + Dashboard     |  |  + services      |   |
|  |  + Health        |  |  + hardening     |   |
|  |  + Monitor       |  |  + data_prot     |   |
|  |  + Benchmark     |  |  + performance   |   |
|  |  + Scanner       |  |  + cleanup       |   |
|  |  + ... (20)      |  |  + monitor       |   |
|  |                  |  |  + benchmark     |   |
|  |  hooks/          |  |  + threat_scan   |   |
|  |  + useInvoke<T>  |  |  + ... (19)      |   |
|  |                  |  |                  |   |
|  |  lib/            |  |  utils/          |   |
|  |  + types.ts      |  |  + registry      |   |
|  |  + helpers.ts    |  |  + process       |   |
|  |  + constants.ts  |  |  + wmi_helpers   |   |
|  |                  |  |  + constants     |   |
|  |  __tests__/      |  |  + logger        |   |
|  |  + helpers       |  |                  |   |
|  |  + use-invoke    |  |  #[cfg(test)]    |   |
|  |  ui/             |  |                  |   |
|  |  + Toast         |  |                  |   |
|  |  + Confirm       |  |                  |   |
|  |  + CmdPalette    |  |                  |   |
|  |  + Sparkline     |  |                  |   |
|  |  + EmptyState    |  |                  |   |
|  |  + ErrorState    |  |                  |   |
|  |  + ItemErrorBdry |  |                  |   |
|  |  + ScoreRing     |  |                  |   |
|  |  + StatCard      |  |                  |   |
|  |  + Badge         |  |                  |   |
|  |  + ThemeProvider |  |                  |   |
|  |  + ThemeSwitcher |  |                  |   |
|  +-----------------+  +------------------+   |
|         invoke()           Windows APIs       |
|         listen()       (WMI, Registry, PS)    |
+----------------------------------------------+
```

## Communication Frontend - Backend

### Commandes (request/response)
```
Frontend                    Backend
invoke("get_system_info") -> #[tauri::command] async fn get_system_info() -> Result<SystemInfoData, CommandError>
invoke("protect_item", {id}) -> #[tauri::command] async fn protect_item(id: String) -> Result<(), CommandError>
```

Les commandes qui touchent WMI, PowerShell ou le disque sont `async` et executent leur corps sur `spawn_blocking` (`utils/blocking.rs`) : une commande synchrone tourne sur le thread principal et gele la fenetre. La liste complete, avec le mode de chaque commande, est dans [COMMANDS.md](COMMANDS.md).

Tous les structs Rust serialises utilisent `#[serde(rename_all = "camelCase")]`.
Les champs Rust `snake_case` deviennent `camelCase` cote TypeScript.

### Events (streaming)
```
Backend                          Frontend
app.emit("system-metrics", data) -> listen("system-metrics", callback)
```
Utilise pour le monitoring temps reel (CPU/RAM toutes les 1.5s).

## APIs Windows utilisees

| API | Crate/Methode | Usage |
|-----|---------------|-------|
| WMI | `wmi::WMIConnection::raw_query` | Infos systeme, drivers, memoire, disques |
| Registry | `windows_registry::LOCAL_MACHINE/CURRENT_USER` | Data protection, hardening, performance, services |
| PowerShell | `std::process::Command` + `Get-CimInstance` | Bloatware, taches, updates, backup, network |
| sc.exe | `std::process::Command` | Arret/demarrage services |
| ping.exe | `std::process::Command` | Test connectivite |
| powercfg.exe | `std::process::Command` | Plans d'energie |
| reg.exe | `std::process::Command` | Backup/restore registre |

## Modules de commandes (28 modules)

15 modules a plat dans `src-tauri/src/commands/` :

| Module | Responsabilite |
|---|---|
| `benchmark` | Test synthetique CPU/RAM/Disk/Network avec historique |
| `bloatware` | Apps UWP pre-installees |
| `defender` | Etat Windows Defender (`Get-MpComputerStatus`) |
| `diagnostics` | Auto-diagnostic des 4 prerequis runtime |
| `disks` | Volumes logiques |
| `frontend_log` | `log_frontend_error` : les erreurs du frontend dans le journal de l'app |
| `logs` | Lecture logs applicatifs Wintyx |
| `memory` | Barrettes RAM + top consommateurs |
| `perf` | Instrumentation perf (feature `perf`, `perf_dump_frontend`) |
| `performance` | Effets visuels, Superfetch, indexation, plan d'energie |
| `scores` | Les 5 scores de domaines en un appel |
| `scores_history` | Historique journalier des scores (90 jours) |
| `startup` | Programmes au demarrage (HKLM/HKCU/Startup folder) |
| `tasks` | Taches planifiees + analyse suspicion |
| `updates` | Historique Windows Update via Get-HotFix |

13 modules avec sous-fichiers (decomposition par responsabilite ; un `tests.rs` voisin porte les tests des fichiers de plus de 300 lignes) :

| Module | Sous-fichiers |
|---|---|
| `backup/` | `mod.rs` (commandes) + `restore_point.rs` + `registry_backup.rs` + `registry_export.rs` + `snapshot.rs` + `system_protection.rs` + `reg_guard.rs` (controle d'un `.reg` avant import) |
| `cleanup/` | `sizes.rs` (mesure des cibles) + `tests.rs`, avec `cleanup.rs` pour les commandes |
| `data_protection/` | `mod.rs` (logique + scoring) + `items.rs` (table des items) |
| `drivers/` | `mod.rs` + `types.rs` + `audit.rs` + `phantoms.rs` + `pnputil_cmd.rs` + `pnputil_parser.rs` + `sources.rs` + `validation.rs` |
| `hardening/` | `states.rs` (lecture des etats) + `tests.rs`, avec `hardening.rs` |
| `monitor/` | `collect.rs` (compteurs WMI bruts) + `sample.rs` (taux entre deux echantillons) + `controller.rs` + `worker.rs` (thread de prelevement WMI) + `supervisor.rs` (relance apres un blocage WMI), avec `monitor.rs` |
| `network/` | `parse.rs` + `tests.rs`, avec `network.rs` |
| `profiles/` | `mod.rs` + `presets.rs` + `storage.rs` + `applicator.rs` |
| `report/` | `mod.rs` (generate_report) + `html.rs` (helpers + CSS + escape) + `labels.rs` (libelles francais des valeurs backend) + `sections.rs` (une collecte en echec s'ecrit « non collectee ») |
| `services/` | `sc_state.rs` (classification des sorties `sc.exe`) + `tests.rs`, avec `services.rs` |
| `snapshot_diff/` | `mod.rs` + `types.rs` + `parser.rs` + `diff_engine.rs` + `restore_engine.rs` + `provenance.rs` (valeur relue dans les snapshots avant restauration) |
| `system_info/` | `antivirus.rs` + `assemble.rs` (assemblage et cache des seuls champs lus) + `tests.rs`, avec `system_info.rs` |
| `threat_scanner/` | `mod.rs` (orchestration + ScanPhase) + `scanners/` (`tasks.rs` via l'analyse de la page Taches, `ports.rs`, `process_based.rs`, `registry_based.rs`) |

Les utilitaires partages vivent dans `src-tauri/src/utils/` (17 modules) :

| Module | Role |
|---|---|
| `admin` | Detection administrateur par le jeton du processus, relance UAC |
| `batch` | `BatchOptions` et `BatchResult` des commandes de lot |
| `blocking` | `read_value`, `read_result`, `write_value`, `write_result` : executent le corps synchrone sur `spawn_blocking` ; les ecritures systeme prennent un verrou commun |
| `cleanup_helpers` | Taille et correspondance de motifs pour le nettoyage |
| `constants` | Delais, seuils et chemins registre |
| `errors` | `CommandError` (9 variants) |
| `fs_atomic` | Ecriture atomique d'un fichier |
| `json_store` | Lecture JSON avec mise en quarantaine d'un fichier corrompu, ecriture atomique |
| `known_folders` | Dossiers connus de Windows (Bureau redirige) |
| `logger` | Journal dans `%LOCALAPPDATA%/Wintyx/Logs/` |
| `paths` | Dossiers de donnees et de sauvegarde |
| `perf` | Mesure des commandes (`PerfGuard`, feature `perf`) |
| `process` | Lancement de processus par chemin absolu sous `%SystemRoot%` (`system_path`), delai, decodage UTF-8 ou page de codes OEM |
| `registry` | Lecture, ecriture, suppression, trace de l'ancienne valeur et validation des chemins (frontiere `\` respectee, `within`) |
| `temp_file` | Fichier temporaire au nom aleatoire, cree en exclusif (`create_new`) |
| `snapshot_cache` | Resultat recent d'une collecte couteuse partage entre commandes (verrou tenu pendant la collecte, invalidation apres ecriture) |
| `wmi_helpers` | Connexion WMI et conversions de `Variant` |

## Securite

- Content Security Policy (CSP) restrictive dans tauri.conf.json
- Sanitisation des inputs utilisateur via `registry::sanitize_name()` avant interpolation
- Escaping PowerShell centralise via `process::escape_powershell()`
- Parsing securise des sorties pipe-separated via `process::parse_pipe_fields()`
- Chemins systeme dynamiques via variables d'environnement (pas de `C:\` en dur)
- Backup automatique du registre avant toute modification (dans %LOCALAPPDATA%/Wintyx/RegistryBackups/)
- Logging structure dans %LOCALAPPDATA%/Wintyx/Logs/ avec rotation 7 jours
- CREATE_NO_WINDOW via constante dans tous les appels process
- Modales de confirmation avant actions destructrices
- ErrorBoundary React pour capturer les crashes frontend

## Abstractions frontend (v2.1.0+)

- **`useToggleAction<T>`** (`src/hooks/use-toggle-action.ts`) : hook generique pour le pattern `confirm + invoke + toast + undo`. Supporte `buildArgs` pour les commandes backend qui n'utilisent pas `id`. Expose `recentId` pour piloter les animations (v2.1.2+).
- **`<CrudListPage<T>>`** (`src/components/ui/CrudListPage.tsx`) : composant generique pour les pages liste (fetch + filter + group + render + empty + virtualize). Supporte `getItemId` pour des keys React stables (v2.1.3+).
- **`useConfirmBatchAction`** (`src/hooks/use-confirm-batch-action.ts`) : hook pour les actions batch avec snapshot auto + dry-run preview.

**Pages migrees vers ces abstractions** : Services, Demarrage, Performance, Protection des donnees, Bloatware (6 pages CRUD, dont Cleanup migree depuis). Hardening et Tasks restent des pages sur mesure.

## Tests et CI

- Tests frontend (Vitest + @testing-library/react), comptes par `bun run test` : helpers, hooks (`useInvoke`, `useToggleAction`, `useConfirmBatchAction`), contexts, composants UI (dont ErrorState, ItemErrorBoundary, ScoreRing, StatCard, ThemeProvider), pages CRUD migrees, mappings FR (getCategoryLabel/getHardeningLevelLabel)
- Tests Rust, comptes par `cargo test` (unitaires `#[cfg(test)]` et test d'integration sur les capabilities de fenetre, `src-tauri/tests/`) : sanitize_name, parse_pipe_fields, escape_powershell, kb_to_gb_rounded, scoring helpers extraits par domaine, classify_update, restore_type_label, is_trusted_author + analyze_suspicious (patterns -enc), backup_folder, etc.
- CI/CD GitHub Actions (.github/workflows/ci.yml) : lint, build, test frontend, clippy, test Rust

## Themes

2 themes signature via CSS custom properties sur `[data-theme]` :
- Obsidian (defaut) : theme sombre, sobre, accent indigo
- Daylight : theme clair, net, accent indigo

Persistence dans `localStorage` via `ThemeProvider`.
