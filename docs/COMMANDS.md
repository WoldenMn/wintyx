# Reference des commandes Tauri

**81 commandes** exposees au frontend via `invoke<T>(command, args)` : 62 `async` et 19 synchrones.

Comptage : lignes de `collect_commands![...]` dans `src-tauri/src/lib.rs::build_specta_builder`, et `#[tauri::command]` suivi de `async fn` ou `fn` dans `src-tauri/src`.

Tous les structs de retour utilisent `#[serde(rename_all = "camelCase")]`. Les commandes qui peuvent echouer retournent `Result<T, CommandError>` (9 variants semantiques) ; le frontend utilise `describeError(e)` pour obtenir un message utilisateur a partir de `{ code, details }`.

## Async et synchrone

Une commande synchrone s'execute sur le thread principal de Tauri : si elle dure, la fenetre gele. Les 62 commandes `async` delegent leur travail bloquant (WMI, PowerShell, fichiers) a `spawn_blocking` via `utils/blocking.rs`, en general avec un corps `*_sync` appelable depuis les tests et les autres commandes.

Les 19 commandes synchrones sont celles qui ne font qu'une lecture locale breve ou qui pilotent un etat en memoire :

| Module | Commandes synchrones |
|--------|----------------------|
| backup | `is_system_protection_enabled`, `get_backup_folder`, `open_backup_folder` |
| benchmark | `get_benchmark_history` |
| data_protection | `get_data_protection_items` |
| frontend_log | `log_frontend_error` |
| monitor | `start_monitoring`, `stop_monitoring`, `set_monitor_interval` |
| network | `get_firewall_status` |
| perf | `perf_dump_frontend` |
| profiles | `get_profiles`, `create_custom_profile`, `update_custom_profile`, `delete_custom_profile` |
| scores_history | `record_score_snapshot`, `get_score_history` |
| startup | `get_startup_items` |
| admin (utils) | `get_admin_status` |

Toutes les autres commandes du document sont `async`. Les delais d'`invoke` cote frontend sont alignes sur ceux du backend, commande par commande, dans `invokeTimeoutFor` (`src/lib/constants.ts`).

## system_info
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_system_info` | `SystemInfoData` | OS, CPU, RAM, disque, uptime, antivirus, GPU |

## data_protection
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_data_protection_items` | `Vec<DataProtectionItem>` | Items telemetrie et vie privee |
| `protect_item` | `()` | Protege un item par id |
| `protect_all` | `BatchResult` | Protege tous les items (`options` : dry-run, filtre) |

## services
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_services` | `Vec<ServiceItem>` | Services suivis ; statut `Running`, `Stopped`, `Absent` ou `Unknown` |
| `stop_service` | `()` | Arrete un service (liste blanche) et memorise son demarrage d'origine |
| `start_service` | `()` | Redemarre un service et restaure son demarrage d'origine |

## hardening
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_hardening_checks` | `Vec<HardeningCheck>` | Controles de securite ; un etat illisible est `isUnknown`, jamais sur |
| `apply_hardening_fix` | `()` | Applique un correctif par id (refuse si configuration manuelle) |
| `apply_hardening_essential` | `BatchResult` | Applique les essentiels |
| `apply_hardening_all` | `BatchResult` | Applique tous les controles |

## performance
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_performance_items` | `Vec<PerformanceItem>` | Optimisations disponibles |
| `optimize_item` | `()` | Applique une optimisation par id |
| `optimize_all` | `BatchResult` | Applique toutes les optimisations |

## cleanup
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_cleanup_targets` | `Vec<CleanupTarget>` | Cibles avec taille ; une cible illisible est `accessible: false` |
| `clean_target` | `CleanResult` | Nettoie une cible par id (octets liberes et erreurs) |
| `clean_all` | `CleanResult` | Nettoie tout (total et erreurs) |
| `flush_dns` | `()` | Vide le cache DNS |

## bloatware
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_bloatware_items` | `Vec<BloatwareItem>` | Apps UWP preinstallees |
| `remove_bloatware` | `()` | Supprime une app par `packageName` |
| `remove_all_bloatware` | `BatchResult` | Supprime toutes les apps de la liste |

## startup
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_startup_items` | `Vec<StartupItem>` | Programmes au demarrage |
| `disable_startup_item` | `()` | Desactive par nom et emplacement |
| `enable_startup_item` | `()` | Reactive par nom et emplacement |

## network
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_network_adapters` | `Vec<NetworkAdapterInfo>` | Adaptateurs reseau |
| `get_firewall_status` | `Vec<FirewallProfile>` | Etat des 3 profils |
| `ping_host` | `PingResult` | Ping d'un hote (nom valide, ne commence pas par `-`) |
| `test_connectivity` | `Vec<PingResult>` | Test de connectivite multi-hotes |
| `get_public_ip` | `String` | IP publique lue aupres d'`api.ipify.org`, appelee seulement sur clic ; un echec est une erreur, pas une chaine vide |

## drivers
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_driver_audit` | `Vec<DriverIssue>` | 5 categories : doublons, fantomes, obsoletes, OEM tiers, non signes |
| `get_driver_audit_summary` | `DriverAuditSummary` | Compteurs par categorie |
| `uninstall_driver` | `BatchResult` | `pnputil /delete-driver <oem*.inf>`, sans `/uninstall` ni `/force` : refuse un paquet en service. Le paquet doit figurer dans l'audit (admin, `options` : dry-run) |
| `remove_phantom_device` | `BatchResult` | `pnputil /remove-device <instanceId>`, refuse une instance que la detection ne classe pas fantome (admin, `options` : dry-run) |
| `remove_all_phantom_devices` | `BatchResult` | Retire tous les fantomes avec snapshot et apercu |

## memory
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_memory_sticks` | `Vec<MemoryStick>` | Barrettes RAM |
| `get_memory_summary` | `MemorySummary` | Total, utilise, emplacements |
| `get_memory_consumers` | `Vec<MemoryConsumer>` | Top 15 des processus par RAM |

## disks
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_disks` | `Vec<DiskInfo>` | Tous les disques logiques |

## updates
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_update_history` | `Vec<WindowsUpdate>` | Historique des mises a jour |
| `get_windows_build` | `String` | Version et build de Windows |
| `get_last_update_date` | `String` | Date de la derniere mise a jour ; une lecture ratee est une erreur |

## tasks
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_scheduled_tasks` | `Vec<ScheduledTaskInfo>` | Toutes les taches planifiees actives |
| `get_suspicious_tasks` | `Vec<ScheduledTaskInfo>` | Taches suspectes seulement |
| `disable_scheduled_task` | `()` | Desactive une tache par chemin |
| `get_suspicious_task_count` | `u32` | Nombre de taches suspectes |

## backup
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_restore_points` | `Vec<RestorePointInfo>` | Points de restauration (administrateur ; liste affichee par la page Sauvegarde) |
| `create_restore_point` | `RestorePointOutcome` | Cree un point et verifie sa creation |
| `backup_critical_keys` | `BatchResult` | Exporte les 7 cles critiques |
| `get_registry_backups` | `Vec<RegistryBackupInfo>` | Liste des sauvegardes |
| `restore_registry_backup` | `bool` | Restaure une sauvegarde par chemin (dans le dossier de backup) |
| `is_system_protection_enabled` | `bool` | Protection du systeme active ou non ; une lecture ratee est une erreur |
| `get_backup_folder` | `String` | Chemin du dossier de sauvegarde |
| `open_backup_folder` | `()` | Ouvre le dossier dans l'Explorateur |
| `create_pre_batch_snapshot` | `SnapshotInfo` | Point de restauration et export registre avant une action destructive |

## report
| Commande | Retour | Description |
|----------|--------|-------------|
| `generate_report` | `String` | Genere le rapport HTML et retourne son chemin |

## monitor
| Commande | Retour | Description |
|----------|--------|-------------|
| `start_monitoring` | `()` | Demarre le streaming (event `system-metrics`, 1,5 s par defaut) |
| `stop_monitoring` | `()` | Arrete le streaming : jeton d'arret propre au thread, jamais d'attente bloquante |
| `set_monitor_interval` | `u64` | Change l'intervalle (1 s a 10 s : une collecte dure ~400 ms) et retourne la valeur retenue |

## diagnostics
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_app_diagnostics` | `AppDiagnostics` | 4 prerequis runtime (WMI, PowerShell, admin, stockage) avec statut OK, Attention ou Echec et pages concernees. `hasCriticalFailure` = echec dur WMI, PowerShell ou stockage ; admin non eleve = Attention, jamais critique |

## scores
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_all_scores` | `AllScores` | Les 5 scores de domaines calcules en parallele (1 IPC au lieu de 5) |

## scores_history
| Commande | Retour | Description |
|----------|--------|-------------|
| `record_score_snapshot` | `()` | Persiste un instantane journalier (retention 90 j, ecriture atomique) |
| `get_score_history` | `Vec<ScoreSnapshot>` | Historique pour la carte Evolution |

## defender
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_defender_status` | `DefenderStatus` | Protection temps reel, signatures, derniere analyse (`Get-MpComputerStatus`) |

## threat_scanner
| Commande | Retour | Description |
|----------|--------|-------------|
| `run_threat_scan` | `ThreatScanResult` | Scan complet (6 categories), progression par l'event `threat-scan-progress` |

## benchmark
| Commande | Retour | Description |
|----------|--------|-------------|
| `run_benchmark` | `BenchmarkResult` | Benchmark CPU, RAM, disque, reseau |
| `get_benchmark_history` | `Vec<BenchmarkEntry>` | Historique persiste (20 dernieres entrees), chaque entree porte la `version` de l'application qui a mesure |

## logs
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_logs` | `Vec<LogEntry>` | Journal de l'application (`levelFilter`, `limit` optionnels) |

## frontend_log
| Commande | Retour | Description |
|----------|--------|-------------|
| `log_frontend_error` | `()` | Ecrit une erreur ou un avertissement du frontend dans le journal (categorie `frontend`, 500 caracteres au plus, retours a la ligne neutralises) |

## profiles
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_profiles` | `Vec<Profile>` | 4 presets et profils personnalises (JSON local) |
| `apply_profile` | `BatchResult` | Applique un profil sur 6 domaines (protection des donnees, durcissement, performance, services, bloatware, demarrage) |
| `get_available_items` | `AvailableItems` | Items disponibles par domaine pour l'editeur |
| `create_custom_profile` | `Profile` | Clone un preset, retourne le profil `custom_<timestamp>` (nom valide) |
| `update_custom_profile` | `Profile` | Modifie un profil personnalise (nom, description, items) |
| `delete_custom_profile` | `()` | Supprime un profil personnalise (id prefixe `custom_` requis) |

## snapshot_diff
| Commande | Retour | Description |
|----------|--------|-------------|
| `list_snapshot_groups` | `Vec<SnapshotGroup>` | Snapshots disponibles (timestamps et fichiers) |
| `diff_snapshot_vs_live` | `SnapshotDiff` | Diff d'un snapshot contre l'etat actuel du registre |
| `diff_snapshot_vs_snapshot` | `SnapshotDiff` | Diff entre 2 snapshots |
| `restore_selected_changes` | `BatchResult` | Restauration selective avec snapshot, apercu et limite de 50 changements |

## admin (utils)
| Commande | Retour | Description |
|----------|--------|-------------|
| `get_admin_status` | `AdminStatus` | `{ isAdmin, canElevate }` ; l'administrateur est detecte par le jeton du processus |
| `relaunch_as_admin` | `()` | Relance Wintyx via UAC (`Start-Process -Verb RunAs`) |

## perf
| Commande | Retour | Description |
|----------|--------|-------------|
| `perf_dump_frontend` | `()` | Recoit les samples perf du frontend (plafond `MAX_SAMPLES_PER_BATCH`). Actif avec la feature cargo `perf` seulement, sans effet sinon |

## Parametre `options` des commandes batch

Les commandes qui retournent un `BatchResult` acceptent `options: Option<BatchOptions>` (`dryRun` : apercu sans ecriture) :
`protect_all`, `apply_hardening_essential`, `apply_hardening_all`, `optimize_all`, `remove_all_bloatware`, `clean_all`, `apply_profile`, `restore_selected_changes`, `uninstall_driver`, `remove_phantom_device`, `remove_all_phantom_devices`.

En dry-run, `BatchResult.dryRun` vaut `true` et `previewItems` liste les elements qui seraient modifies, sans ecriture registre ni fichier.

## Events Tauri

| Event | Payload | Emetteur |
|-------|---------|----------|
| `system-metrics` | `SystemMetrics` | `start_monitoring` (1,5 s par defaut) |
| `threat-scan-progress` | `ThreatScanProgress { phaseName, phaseIndex, phaseCount, scannedSoFar }` | `run_threat_scan` (a chaque phase) |
