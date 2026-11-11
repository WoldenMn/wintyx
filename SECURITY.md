# Securite Wintyx

Politique de securite, findings connus et bonnes pratiques.

## Signaler une vulnerabilite

Pour signaler une vulnerabilite de securite dans Wintyx, ne pas ouvrir
d'issue publique. Utiliser le signalement prive de GitHub : onglet
**Security** du depot, puis **Report a vulnerability**
(<https://github.com/WoldenMn/wintyx/security/advisories/new>).

Decrire la version de Wintyx, la version de Windows, les etapes pour
reproduire et l'effet obtenu. Une premiere reponse est visee sous 7 jours.
Le correctif sort en version corrective (x.y.Z) et figure dans le
[CHANGELOG](CHANGELOG.md).

## Versions prises en charge

Seule la derniere version publiee recoit des correctifs de securite.

## Modele de menace

Wintyx est une application **desktop locale** qui s'execute dans le
contexte utilisateur Windows. Elle modifie la configuration systeme
(registre, services, scheduled tasks, processus).

**Attaquants hors-scope** :
- Attaquant distant sans acces machine : N/A (aucun service en ecoute ; seules sorties : `ping` et, sur clic, `api.ipify.org` en HTTPS)
- Attaquant avec acces physique : N/A (la machine est deja compromise)
- Attaquant via reseau sur meme LAN : N/A (pas de listener reseau)

**Attaquants dans le scope** :
- Processus malveillant deja en execution sur la machine
  (tente d'utiliser Wintyx comme vecteur d'escalation)
- Input utilisateur malveillant (via UI, fichiers, registre)
- Injection dans les commandes externes (sc.exe, PowerShell, reg.exe)
- Path traversal (backups, logs, exports)

## Historique des audits

| Version | Date | Type | Resultat |
|---|---|---|---|
| v2.0.7 | 2026-04-24 | Audit complet (3 agents : backend, frontend, hygiene) | 4 fixes HIGH/MEDIUM appliques + 6 tests securite |
| v2.0.7 | 2026-04-24 | 2e passe verification + cohrence docs | Fixes confirmes + 5 incoherences docs corrigees |
| v2.1.0 | 2026-04-25 | Audit delta v2.0.7 → v2.1.0 (1 agent cible) | **CLEARED** : 0 nouvelle vulnerabilite, tous les fixes v2.0.7 intacts, 0 nouvelle commande IPC |
| v2.1.2 | 2026-05-04 | Audit delta v2.1.0 → v2.1.2 (migration CRUD + animations feedback) | **CLEARED** : 0 nouvelle vulnerabilite, 0 nouvelle commande Tauri, animations CSS strictement additives |
| v2.7.1 | 2026-05-11 | Audit chirurgical delta v2.0.7 → v2.7.0 (3 agents cibles : snapshot_diff, profiles, tauri-specta RC) | 6 fixes appliques (3 HIGH + 3 MEDIUM) + 5 nouveaux tests Rust ; regressions v2.0.7 INTACTES |
| v2.7.2 | 2026-05-11 | Suite hygiene v2.7.1 (refactor modulaire + L1/L3) | Split snapshot_diff.rs 990L → module 4 fichiers (mod 500L, parser 250L, diff 250L, restore 145L) + 3 tests UTF-16 (L1 ferme) + doc sanitize_* clarifiee (L3 ferme) |
| v2.7.3-v2.7.5 | 2026-05-11 | Suite hygiene refactor (profiles, threat_scanner, SnapshotComparePage) | Refactor pur sans nouveaute attaque ; tous les fixes v2.0.7 et v2.7.1 INTACTES |
| v2.7.6 | 2026-05-11 | Audit a11y WCAG 2.1 AA | 6 fixes (focus-visible inputs, caption sr-only, scope=col, aria-pressed, aria-busy) ; pas de vulnerabilite de securite |
| v2.7.9 | 2026-05-15 | Audit complet 4 agents Explore | 4 fixes (C1 CommandPalette cleanup, H1 decouplage apply_profile, H3+H4 perf with_wmi_conn) ; pas de nouvelle vulnerabilite |
| v2.8.0 | 2026-05-17 | Audit drivers (validation inputs PowerShell) | `validate_inf_name` regex stricte `^oem\d{1,4}\.inf$`, `validate_instance_id` pattern PNP ASCII strict, `escape_powershell` + `require_admin` sur 3 actions destructives drivers |
| v2.9.1-v2.9.3 | 2026-05 | Audits perf-infra + CVE (`cargo audit`) + audit complet 3 agents Opus | 0 CVE active backend, 0 bloquant securite ; baseline v2.0.7/v2.7.1/v2.8.0 confirmee intacte |
| v2.10.0 | 2026-06-03 | Release final : audit exhaustif 8 axes + verification adversariale (0 faux positif) | 43 findings traites (0 critical) ; 4 fixes securite : allowlist `sanitize_task_name` + `require_admin` sur `disable_scheduled_task`, `sanitize_key_path` centralise, `escape()` HTML systematique du rapport, permission `shell:allow-open` retiree |
| v2.18.0 | 2026-09-25 au 2026-10-02 | Audit backend et frontend de la pile v2.18 (runs 2 a 9b), correctifs verifies sur une machine reelle avec les vraies sorties PowerShell, WMI et pnputil | 5 correctifs de securite : injection PowerShell par apostrophes typographiques (U+2018 a U+201B) fermee dans `escape_powershell`, `/force` retire de `pnputil /delete-driver`, hote de ping commencant par `-` refuse, detection administrateur par jeton d'elevation au lieu de `net session`, noms et descriptions de profil sans caracteres de controle (creation et chargement). Aucune nouvelle permission Tauri |
| v2.19.0 | 2026-10-23 | Audit complet avant publication (backend et frontend, lecture integrale par deux agents, correctifs verifies par tests) | Executables lances par chemin absolu sous `%SystemRoot%` (detournement par un faux `powershell.exe` a cote de l'exe), fichiers temporaires aleatoires et exclusifs, contenu des `.reg` controle avant import, valeur restauree relue dans les instantanes, `pnputil` sans `/uninstall`, SmartScreen sorti des lots, `tauri-plugin-shell` retire. Aucune nouvelle permission Tauri |

### Audit v2.7.1 — 11 mai 2026

**Delta v2.0.7 → v2.7.0** (10 releases : v2.0.8 a v2.7.0). 3 cibles a risque max selectionnees :

1. **`snapshot_diff` + `restore_selected_changes`** (v2.4.0 + v2.5.0) — ecriture registre privilegiee, parser .reg manuel UTF-16
2. **`profiles` + customs** (v2.3.0 + v2.6.0 + v2.7.0) — JSON local atomique + cascade 6 domaines
3. **`tauri-specta` + 3 deps RC** (v2.2.0) — codegen + CLI flag `--export-bindings`

**Fixes appliques** :

- **H1** — `--export-bindings` desormais gated par `#[cfg(debug_assertions)]` dans `main.rs` ET `lib.rs::export_bindings`. Plus d'introspection API depuis binaire release.
- **H2** — `is_allowed_key_path` rendu case-insensitive (`to_ascii_uppercase`) + nouveau helper public `is_hklm_path` (`eq_ignore_ascii_case` sur les 4 premiers octets) reutilise dans `snapshot_diff::restore_selected_changes` et `backup::backup_registry_key`. Bloque le bypass de l'allowlist via une variation de casse (`hklm` vs `HKLM`). Note : `startup.rs:113,158` conserve `starts_with("HKLM")` car `location` est un discriminator d'enum (`HKLM_Run`/`HKCU_Run`), pas un chemin registre.
- **H3** — `save_custom_profiles_to` (profils customs) abandonne le pattern TOCTOU `remove_file` + `rename` au profit de `fs::rename` direct (atomique sur NTFS). Cleanup du tempfile en cas d'erreur.
- **M1** — `decode_reg_file` remplace `String::from_utf16_lossy` par `String::from_utf16` strict + log warn. Plus de decodage silencieux de surrogate pairs orphelins (sequence invalide → fichier rejete).
- **M2** — helper privé `is_symlink` propage dans `list_snapshot_groups`, `diff_snapshot_vs_snapshot` et `diff_snapshot_vs_live`. Coherence avec `backup::restore_registry_backup` qui rejette deja les symlinks.
- **M3** — `load_custom_profiles_from` applique `.take(MAX_CUSTOM_PROFILES)` apres deserialisation. Un fichier JSON edite a la main ne peut plus depasser la limite 20 customs.

**Findings non-fixes** :

- **5 LOW notes** (statut mis a jour en v2.7.2) :
  - **L1 — FERME en v2.7.2** : tests UTF-16 manquants pour `decode_reg_file` (sans BOM, surrogate orphelins). 3 tests ajoutes : `decode_reg_file_with_bom_le`, `decode_reg_file_without_bom_le`, `decode_reg_file_invalid_surrogate` validant le fix M1.
  - **L3 — FERME en v2.7.2** : divergence sémantique documentee `sanitize_name` vs `sanitize_value_name`. Docstrings enrichies avec cross-references mutuelles et warning explicite "ne jamais utiliser X pour Y".
  - **L2 — ACCEPTE** : pas de rate-limiting sur `restore_selected_changes` (50 changements max + atomicite Windows = acceptable).
  - **L4 — ACCEPTE** : `parse_startup_id` sans `|` retourne location vide, rejetee en aval (comportement defensif).
  - **L5 — ACCEPTE** : pas de mutex global sur `custom_profiles.json` ; Tauri est single-instance par defaut.

- **4 INFO acceptes** (modele de menace desktop local) :
  - **Dependances RC en production** (specta 2.0.0-rc.20, tauri-specta 2.0.0-rc.20, specta-typescript 0.0.7) : pas de CVE/RUSTSEC connue, mainteneurs actifs (oscartbeaumont + Brendonovich), upgrade vers stable planifie quand 2.0.0 sort.
  - **`RegistryBackupInfo.filePath` expose le username** via path `C:\Users\<username>\AppData\Local\Wintyx\Backups\...` : acceptable car commande Tauri (pas d'API publique), modele de menace desktop local.
  - **`bindings.ts` contenu audite** : aucune fuite (paths internes, env vars, debug info). Versionne depuis 2026-09 (la CI en a besoin) ; contenu reverifie a ce moment, toujours sans chemin ni variable d'environnement.
  - **u64 → JS number CLEARED** : tous les champs u64 du backend (uptimeSeconds, sizeBytes, freedBytes, durationMs, fileSizeBytes, timeoutMs) restent dans la safe range JS (`<2^53 - 1`).

**Verrou environnemental** : `cargo check`/`cargo build` bloque pendant l'audit par Windows Defender qui verrouille `target/debug/build/typeid-*/build-script-build.exe` a la creation (os error 32 reproductible). Validation deleguee a la 2e passe Explore (lecture statique du code) et au smoke test utilisateur final.

### Audit v2.8.0 — 17 mai 2026

**Cible** : nouvelles 5 commandes drivers (`get_driver_audit`, `get_driver_audit_summary`, `uninstall_driver`, `remove_phantom_device`, `remove_all_phantom_devices`) qui invoquent `pnputil` + `Get-PnpDevice` avec des inputs utilisateur (oem*.inf, PnP instance IDs).

**Defenses appliquees** :

- **`validate_inf_name`** : regex stricte `^oem\d{1,4}\.inf$` (rejette path traversal et injection caracteres). Test : 4 cas de validation TDD.
- **`validate_instance_id`** : pattern ASCII strict (charset `[A-Z0-9\\&_]`, min 2 backslashes, max 200 chars). Rejette Unicode confusables et injection shell. Test : 2 cas de validation TDD.
- **`escape_powershell`** applique systematiquement sur les inputs interpoles dans `Get-PnpDevice -InstanceId '...'`.
- **`require_admin`** sur `uninstall_driver`, `remove_phantom_device`, `remove_all_phantom_devices` (manipulent le DriverStore + Device Manager, equivalent operations HKLM).
- **`remove_all_phantom_devices`** : limite implicite via `collect_phantoms()` (pas de boucle infinie possible, le scan WMI est borne).

**Findings non-fixes** : aucun. Code review TDD-first a couvert tous les chemins d'input.

### Audit v2.1.2 — 4 mai 2026

**Delta v2.1.0 → v2.1.2** (3 releases : v2.1.1 migration CRUD + v2.1.2 feedback animations).

**Changements analyses** : extension API `useToggleAction<T>` (champ `recentId`), animations CSS Tailwind (halo, Badge crossfade, count-up, ActionButton scale), 4 pages CRUD migrées vers les abstractions v2.1.0.

**Findings** :
- **0 nouvelle vulnerabilite**
- 0 nouvelle commande Tauri
- Tous les fixes v2.0.7 confirmes presents (escape_powershell, sanitize_name, MonitorFlagGuard, CSP durcie)
- Animations strictement additives (CSS uniquement, pas de logique)
- Pattern `data-attr={cond ? "true" : undefined}` evite les fuites string

**Findings INFO (non bloquants)** :
- `prefers-reduced-motion` non respecte → corrige en v2.1.3 (CSS global media query)
- `useToggleAction` deps `options` dans `useCallback` : pre-existant, deja documente

### Audit delta v2.1.0 → v2.5.0 (2026-05-07)

**Verdict** : aucune nouvelle vulnérabilité.

3 releases auditées en chaîne (v2.5.0 snapshot vs snapshot + restauration sélective) :
- 2 nouvelles commandes Tauri (`diff_snapshot_vs_snapshot`, `restore_selected_changes`)
- Toutes les défenses v2.0.7 préservées (escape_powershell, sanitize_name, MonitorFlagGuard, CSP durcie)
- Nouveau helper `is_allowed_key_path` (allowlist 4 préfixes registre) — réutilisé par `backup_registry_key` (DRY)
- Nouveau helper `sanitize_value_name` (autorise espaces ASCII, contrairement à `sanitize_name` strict)
- Limite 50 changements par batch de restauration sélective (defense-in-depth)
- `require_admin` propagé sur HKLM uniquement
- WCAG AA : `aria-pressed` sur ModeButton, `aria-label` sur checkboxes, focus trap sur les modals

### Audit v2.5.0 → v2.6.0 (2026-05-07)

**Verdict** : approve avec concerns mineurs (non bloquants).

Nouvelle infrastructure profils customs :
- 4 nouvelles commandes Tauri (`get_available_items`, `create_custom_profile`, `update_custom_profile`, `delete_custom_profile`)
- Stockage JSON local atomique dans `%APPDATA%/Wintyx/custom_profiles.json` (tempfile + remove + rename Windows-compatible)
- Limite 20 customs (defense-in-depth)
- Validation : nom 1-80 chars, description ≤ 200 chars, ≥ 1 item, ID préfixé `custom_` requis pour update/delete
- IDs items invalides ignorés silencieusement à l'application (capturés dans `BatchResult.errors`)
- WCAG AA : focus trap + Escape sur `CloneProfileDialog` (aligné avec ConfirmDialog v2.0.3)
- Aucune nouvelle surface d'attaque (lecture/écriture JSON dans `%APPDATA%`, pas d'admin gate spécifique)

Concerns mineurs (à traiter en hygiène future) :
- `create_custom_profile` valide `items_count=1` hardcodé (impact nul aujourd'hui car tous les presets ont ≥ 1 item)
- `CloneProfileDialog` utilise `autoFocus` plutôt que `useEffect + ref.focus()` (équivalent fonctionnel)

### Audit v2.6.0 → v2.7.0 (2026-05-07)

**Verdict** : approve après fix d'1 bloquant fonctionnel identifié par le code-quality reviewer final.

Extension structurelle des profils customs aux 3 domaines reportés (services, bloatware, startup) :
- **Aucune nouvelle commande Tauri** (compat ascendante v2.6.0 garantie via `#[serde(default)]`)
- 3 nouveaux `Vec<String>` sur `Profile` (`servicesIds`, `bloatwareIds`, `startupIds`)
- Helper `parse_startup_id` pour décoder l'id composite `${location}|${name}` (split sur premier `|`)
- 4 presets curatés avec items pré-sélectionnés
- Items absents ignorés silencieusement (collectés dans `BatchResult.errors`, affichés en toast)

**Bug critique fixé en cours d'audit** :
- Les 3 presets contenant `OneDrive` dans `startup_ids` utilisaient initialement le chemin registre complet (`HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run|OneDrive`) au lieu du code court attendu par `disable_startup_item` (`HKCU_Run|OneDrive`). L'action OneDrive aurait été silencieusement ignorée à l'apply.
- Fix : alignement sur le format produit par `get_startup_items` (codes courts).
- Test de garde ajouté (`presets_startup_ids_use_short_location_codes`) pour empêcher la régression.

Lacune systémique notée : les tests d'intégration cross-modules (profiles ↔ startup) sont une zone de risque — candidate pour une release hygiène future.

### Dette technique connue (état au 2026-10-02)

La liste de l'audit v2.7.1 (profiles.rs, snapshot_diff.rs, SnapshotComparePage) est soldee : `profiles/` et `snapshot_diff/` sont des modules decoupes, `SnapshotComparePage.tsx` fait 214 lignes. Ce qui reste :

- Aucun fichier de code au-dessus de 300 lignes, Rust comme TypeScript (le plus long : `report/mod.rs`, 300 lignes pile, et `BenchmarkPage.tsx`, 270 lignes). Seuil de `REVIEW.md`, a tenir a chaque PR.
- `HardeningPage` et `TasksPage` ne passent pas par `CrudListPage` : 6 pages sur 8 sont migrees, Nettoyage compris.
- 20 commandes restent synchrones (lectures de registre ou de JSON local brèves, pilotage du monitoring) : elles tournent sur le thread principal. Liste dans [docs/COMMANDS.md](docs/COMMANDS.md). A rendre `async` si une mesure montre un gel.
- Le statut administrateur est lu une fois par processus (voir « Findings connus »).

## Defenses en place (v2.0.7+)

### Command injection

- **`utils/process/text.rs::escape_powershell`** double `'` et les
  apostrophes typographiques U+2018 a U+201B (PowerShell les prend toutes
  pour des delimiteurs de chaine) et supprime les caracteres de controle et
  les separateurs U+2028/U+2029, avant toute interpolation dans une chaine
  PowerShell **entre quotes simples**. Elle ne protege pas une chaine entre
  guillemets doubles, ou `$` et le backtick sont interpretes
- **`pnputil`** : l'identifiant d'un peripherique fantome passe par
  `validate_instance_id` puis par `/remove-device` sans shell, et la
  desinstallation d'un pilote n'utilise ni `/force` ni `/uninstall` : pnputil
  refuse de retirer un paquet encore utilise par un peripherique present.
  Une instance ou un paquet absent de la detection de l'audit est refuse
- **Executables par chemin absolu** (`utils/process/system_path.rs`) :
  PowerShell, `sc`, `reg`, `pnputil`, `schtasks`, `secedit` et les autres outils
  sont lances depuis `%SystemRoot%\System32`, jamais par nom nu. Sans cela,
  Windows chercherait d'abord dans le dossier de `Wintyx.exe`, inscriptible
  par l'utilisateur pour une installation NSIS ou portable, et un faux
  `powershell.exe` depose la tournerait avec les droits administrateur
- **Fichiers temporaires** (`utils/temp_file.rs`) : nom aleatoire et creation
  exclusive, pour qu'un lien pre-place ne detourne pas une ecriture faite en
  administrateur (export `secedit`, fichier du benchmark disque)
- **`ping_host`** refuse un hote qui commence par `-` (injection d'option)
- **`utils/registry.rs::sanitize_name`** valide tous les noms passes
  aux outils shell : ASCII-only, alphanumerique + `[- _ .]` uniquement,
  rejette les espaces et Unicode (defense-in-depth contre confusables
  et argument splitting)

### Path traversal

- **`backup::restore_registry_backup`** : `canonicalize()` + verification
  `starts_with(backup_folder())` pour empecher `../` et symlinks ; avant
  `reg import`, le contenu est lu (`backup/reg_guard.rs`) : aucune ligne de
  suppression, aucune cle hors des cles critiques exportees par Wintyx, fichier
  verrouille en lecture pendant l'import
- **`restore_selected_changes`** : la valeur a restaurer est relue dans les
  `.reg` des snapshots (`snapshot_diff/provenance.rs`) ; une valeur envoyee par
  l'interface et absente des snapshots est refusee. Les prefixes autorises
  respectent la frontiere `\` (`HKLM\SOFTWAREX` ne passe pas)
- **`backup::backup_registry_key`** : whitelist stricte des prefixes
  registre autorises (`HKLM\SOFTWARE`, `HKLM\SYSTEM`, `HKCU\SOFTWARE`,
  `HKCU\Control Panel`)

### Privilege escalation

- **`utils/admin::require_admin`** sur toutes les operations HKLM,
  services (sc.exe stop/start), taches planifiees (schtasks),
  sauvegardes registre
- **Mode admin detecte** : `get_admin_status` expose l'etat au frontend
  qui grise les boutons concernes. L'elevation est lue sur le jeton du
  processus (`TokenElevation`), pas par `net session`, qui depend du
  service Serveur
- **Elevation controlee** : `relaunch_as_admin` via `Start-Process -Verb
  RunAs` (mecanisme UAC Windows)

### Content Security Policy (Tauri)

```
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';  (necessaire pour Tailwind)
img-src 'self' data:;
frame-ancestors 'none';             (anti-clickjacking)
base-uri 'self';                    (anti-base-injection)
form-action 'self';                 (anti-form-hijacking)
object-src 'none';                  (bloque plugins/embeds)
```

### Capabilities Tauri (minimales)

- `core:default` : actions basiques Tauri
- cinq permissions `core:window:*` : reduire, agrandir ou restaurer, fermer,
  lire l'etat agrandi, glisser la fenetre (barre de titre personnalisee)
- `shell:allow-open` a ete **retiree** en v2.10.0 : l'ouverture d'un dossier
  passe par `explorer.exe` lance depuis le backend, sans permission frontend
- **Aucune** `shell:allow-execute`, `fs:*` ou autre permission large
- Un test d'integration (`src-tauri/tests/window_capabilities.rs`) verifie que ces cinq permissions sont accordees

### Race conditions

- **Monitoring** : chaque thread a son jeton d'arret ; un nouveau thread
  attend la fin du precedent, `stop` ne bloque jamais sur un `join`, et une
  requete WMI bloquee est abandonnee puis relancee (`monitor/supervisor.rs`)
- **`BenchmarkGuard`** RAII : le flag `BENCHMARKING` est remis a `false` via
  `Drop` meme en cas de panic du thread
- Toutes les operations atomiques via `AtomicBool` + `Ordering::SeqCst`
- **`SYSTEM_WRITE_LOCK`** (`utils/blocking.rs`) : les commandes d'ecriture
  systeme passent par `write_*` et se serialisent ; un snapshot ne capture
  jamais un etat a moitie modifie. Les lectures restent paralleles. Le verrou
  n'est jamais pris dans les corps synchrones, donc une commande qui en appelle
  une autre ne peut pas s'auto-bloquer

### Snapshots automatiques

- **`create_pre_batch_snapshot`** : point de restauration systeme +
  export des 7 cles registre critiques, pris par `useConfirmBatchAction`
  avant les lots qui ecrivent dans le registre ou la configuration :
  `protect_all`, `apply_hardening_*`, `optimize_all`, `remove_all_bloatware`,
  retrait des peripheriques fantomes, application d'un profil, restauration
  depuis un snapshot, Benchmark. Le point de restauration n'est pas cree
  quand Windows applique sa limite de 24 h : l'interface l'indique
- **Pas de snapshot avant `clean_all`** : le nettoyage supprime des
  fichiers temporaires et des caches, qu'un export de registre ne restaure
  pas. Il est protege par la confirmation de la page Nettoyage, pas par un snapshot
- **Un snapshot rate ne passe plus sous silence** : si l'export echoue ou ne
  produit aucune cle, la boite de dialogue demande de confirmer la poursuite
  sans possibilite de retour en arriere

### Logging

- Logs dans `%LOCALAPPDATA%\Wintyx\Logs\` (ACL utilisateur courant)
- Rotation automatique au-dela de 10 Mo
- Retention 7 jours
- **Aucun secret ou token** n'est journalise

### Aucun secret hardcode

- Pas de token, API key, password dans le code source
- Pas de fichier `.env` en production
- Credentials jamais commits dans git (verification `.gitignore`)

## Findings connus (acceptes)

Les findings suivants ont ete identifies par les audits v2.0.7 mais
non-fixes car acceptes dans le contexte d'une app desktop locale :

### MEDIUM - Error message leakage

**`lib/helpers.ts::describeError`** et **`CommandError::ProcessFailed`**
exposent les details techniques (noms de commandes, paths, messages
WMI) a l'utilisateur final via les toasts.

**Raison d'acceptation** : utile pour le debug. Ces details ne s'affichent
qu'a l'utilisateur de la machine, dans sa propre session ; rien ne sort
du poste. Un attaquant qui peut lire les toasts a deja acces au processus.

### LOW - Path exposure dans BackupPage

Le chemin complet `C:\Users\<username>\AppData\Local\Wintyx\Backups`
est visible dans l'UI.

**Raison d'acceptation** : accepte pour app desktop. L'utilisateur sait
ou sont ses donnees. Un attaquant avec acces processus a deja lu le
chemin via les env vars.

### LOW - Dry-run bypass de require_admin

Les 6 batch actions en mode `dry_run=true` ne verifient pas `require_admin`
avant de lister les items qui seraient modifies.

**Raison d'acceptation** : voulue. Un utilisateur non-admin peut voir
ce qui serait modifie en elevation (incitation a elevat, pas un risque
de securite). Aucune modification n'est effectuee en dry-run. Portee
bornee : idem pour `restore_selected_changes` (comparateur de snapshots),
dont les previews dry-run n'exposent que des descriptions key_path /
value_name de cles deja validees par l'allowlist
`registry::is_allowed_key_path`, sans lecture ni ecriture registre.

### LOW - Statut admin mis en cache pour toute la session

`utils::admin::is_admin()` lit l'elevation du jeton du processus une seule
fois (`OnceLock`) et ne la reevalue jamais avant redemarrage de l'application.

**Raison d'acceptation** : la revocation d'un token Windows a chaud (retrait
du groupe Administrators pendant que Wintyx tourne) est un scenario
rarissime, et une verification live a chaque commande admin aurait un cout
disproportionne. Le cache est invalide de fait par `relaunch_as_admin`
(nouveau processus). Reevaluation planifiable si un cas reel apparait.

### LOW - Logger sans redaction des paths

Les paths systemes et usernames apparaissent dans les logs.

**Raison d'acceptation** : logs locaux ACL-protected. Un attaquant avec
acces fichier sur la machine peut lire tous les paths directement.

### INFO - wmi crate 0.14 (0.18 disponible)

Pas de CVE connue sur 0.14 (`cargo audit` du 2026-10-23). La 0.18 change
l'API : migration en cycle separe, avec verification sur une machine reelle.

### INFO - useToggleAction options dans deps useCallback

Le `trigger` du hook `useToggleAction<T>` (introduit en v2.1.0) inclut `options`
dans les dependances de `useCallback`. Si la page recree l'objet `options` a
chaque render, `trigger` est recree aussi (impact perf negligeable, zero impact
securite). Optimisable en futur cycle si besoin.

### INFO - Pas de schema validation runtime

Pas de Zod / Yup sur les retours `invoke()` frontend. Les types TypeScript
sont declaratifs uniquement.

**Raison d'acceptation** : low risk car backend sous notre controle.
Migration possible si IPC surface elargie au public.

## Bonnes pratiques pour contributeurs

### Avant d'ajouter une commande Tauri

1. La commande necessite-t-elle des privileges admin ? Ajouter
   `admin::require_admin("Description action")?` en debut de fonction.
2. Input utilisateur (chemins, noms) ? Passer par `sanitize_name`
   ou `escape_powershell` avant interpolation.
3. Erreurs ? Retourner `Result<T, CommandError>` avec le variant
   semantique (`NotFound`, `InvalidInput`, `ProcessFailed`, etc.).
4. Tests ? Au minimum : 1 test de cas nominal + 1 test de validation
   d'input (refus d'input malveillant).

### Avant de modifier les logs

- Ne jamais inclure de tokens, cookies, hashs de mots de passe
- Ne jamais loguer le contenu de fichiers utilisateur
- Privilegier les messages structures (`logger::info("component", "event")`)
- Pas de PII (Personally Identifiable Information)

### Avant de modifier la CSP

Toute modification de `tauri.conf.json::security.csp` doit etre revue
dans un PR dedie avec justification. La CSP actuelle est durcie et ne
doit pas etre relachee sans raison majeure.

## Ressources

- [Tauri Security](https://v2.tauri.app/concept/security/)
- [Microsoft Security Baseline (Windows 10/11)](https://learn.microsoft.com/en-us/windows/security/threat-protection/windows-security-baselines)
- [OWASP Desktop App Security](https://owasp.org/www-project-desktop-app-security-top-10/)
