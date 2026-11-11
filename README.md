# Wintyx

[![CI](https://github.com/WoldenMn/wintyx/actions/workflows/ci.yml/badge.svg)](https://github.com/WoldenMn/wintyx/actions/workflows/ci.yml)
[![Licence MIT](https://img.shields.io/badge/licence-MIT-blue.svg)](LICENSE)
![Windows 10/11](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D4)

Application de bureau pour Windows 10 et 11 qui analyse, nettoie, optimise et durcit le système, en local, sans compte ni télémétrie. Écrite en Rust et React avec Tauri v2 : un exécutable de 12 Mo, un installeur de 3 Mo.

> **Wintyx modifie votre système** (registre, services, tâches planifiées, pilotes). Les actions en lot (durcissement, performance, protection des données, applications préinstallées, pilotes, profils) prennent d'abord un instantané du registre, et un point de restauration quand Windows l'accepte (un par 24 heures au plus) ; toute action destructrice demande confirmation. Le logiciel est fourni sans garantie (voir la [licence](LICENSE)) : lisez ce qu'une action va changer avant de la lancer.

## Fonctionnalités

| Catégorie | Pages |
|-----------|-------|
| **Principal** | Tableau de bord : score global, cinq domaines, tendance sur 7 et 30 jours, état de Windows Defender |
| **Optimisation** | Profils, Nettoyage, Performance, Démarrage, Applications préinstallées |
| **Sécurité** | Durcissement, Services, Protection des données, Scanner de menaces |
| **Système** | Monitoring en temps réel, Santé, Réseau, Disques, Pilotes (doublons, fantômes, obsolètes, non signés), Mémoire |
| **Outils** | Mises à jour, Tâches planifiées, Sauvegarde, Comparaison d'instantanés, Journaux, Benchmark |
| **Autres** | Diagnostic de l'application, Aide, À propos |

- **Aperçu avant d'agir** : les actions en lot compatibles listent dans leur confirmation ce qu'elles vont changer.
- **Revenir en arrière** : instantané du registre avant chaque lot qui l'écrit, comparaison d'un instantané avec l'état actuel et restauration sélective des valeurs.
- **Aucun échec affiché comme un succès** : un contrôle illisible s'affiche « Inconnu », une collecte ratée affiche une erreur avec un lien vers le diagnostic.
- **Benchmark avant et après** pour mesurer l'effet d'une optimisation.
- Palette de commandes (`Ctrl+K`), thèmes sombre et clair, interface entièrement en français, navigation complète au clavier (WCAG 2.1 AA).

## Installation

Téléchargez la dernière version dans les [Releases](https://github.com/WoldenMn/wintyx/releases) :

| Fichier | Usage |
|---------|-------|
| `Wintyx_<version>_x64-setup.exe` | Installeur recommandé, pour votre compte, sans droits administrateur |
| `Wintyx_<version>_x64_fr-FR.msi` | Windows Installer, pour un déploiement par GPO ou outil d'entreprise |

Prérequis : Windows 10 ou 11, 64 bits, avec WebView2 (présent d'office sur Windows 11 et sur un Windows 10 à jour).

Les binaires ne sont pas signés : au premier lancement, SmartScreen affiche « Windows a protégé votre ordinateur ». Cliquez sur **Informations complémentaires**, puis **Exécuter quand même**. Vous pouvez aussi compiler Wintyx vous-même (plus bas).

### Droits administrateur

Sans droits administrateur, Wintyx lit tout : scores, diagnostic, pilotes, tâches, réseau, benchmark. Les actions qui écrivent dans `HKLM` ou touchent un service affichent un cadenas ; le bouton **Relancer en admin**, en bas de la fenêtre, redémarre l'application avec ces droits par la demande UAC de Windows.

## Données et réseau

Tout reste sur la machine :

| Emplacement | Contenu |
|-------------|---------|
| `%LOCALAPPDATA%\Wintyx\Logs` | Journaux, conservés 7 jours |
| `%LOCALAPPDATA%\Wintyx\Backups` | Instantanés et exports du registre |
| `%LOCALAPPDATA%\Wintyx\scores_history.json`, `benchmarks.json` | Historique des scores (90 jours) et des benchmarks |
| `%APPDATA%\Wintyx` | Profils personnalisés |

Wintyx n'envoie aucune donnée. Ses seules sorties réseau sont des mesures : un `ping` vers `8.8.8.8`, `1.1.1.1` et `google.com` (test de connectivité, benchmark) et, sur la page Réseau, la lecture de votre adresse IP publique auprès d'`api.ipify.org`.

## Compiler depuis les sources

Prérequis : [Rust](https://rustup.rs) stable, [Bun](https://bun.sh), Visual Studio Build Tools avec la charge de travail « Développement Desktop en C++ ».

```bash
bun install
bun tauri dev        # application en développement, rechargement à chaud
bun tauri build      # installeurs NSIS et MSI dans src-tauri/target/release/bundle/
```

Vérifications, à zéro erreur avant toute contribution :

```bash
bun run lint && bun run test && bun run build
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test
```

## Structure

```
src/                 Interface React 19 + TypeScript (strict), Tailwind CSS v4
  components/pages/  25 pages
  components/ui/     composants partagés (dialogues, états d'erreur, graphiques)
  hooks/ contexts/   accès aux commandes, état partagé (admin, scores, monitoring)
  lib/               types, bindings générés depuis Rust, aides
src-tauri/           Backend Rust
  src/commands/      28 modules, 81 commandes Tauri
  src/utils/         processus, registre, WMI, journal, écriture atomique
  capabilities/      permissions de la fenêtre, réduites au strict nécessaire
docs/                architecture, commandes, guide utilisateur, charte
```

## Documentation

| Fichier | Contenu |
|---------|---------|
| [docs/USER_GUIDE.md](docs/USER_GUIDE.md) | Guide utilisateur, page par page |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Architecture Tauri v2, pont Rust et React |
| [docs/COMMANDS.md](docs/COMMANDS.md) | Référence des 81 commandes |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Mise en place et ajout d'une page |
| [docs/BRAND.md](docs/BRAND.md) | Identité visuelle |
| [CHANGELOG.md](CHANGELOG.md) | Historique des versions |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Conventions, flux de contribution, vérifications |
| [SECURITY.md](SECURITY.md) | Modèle de menace, défenses, signalement d'une faille |
| [CLAUDE.md](CLAUDE.md) | Décisions techniques et pièges connus, pour les assistants de code |

## Contribuer

Les issues et pull requests sont bienvenues : lisez [CONTRIBUTING.md](CONTRIBUTING.md). Une faille de sécurité se signale en privé, jamais dans une issue : voir [SECURITY.md](SECURITY.md).

## Licence

[MIT](LICENSE).
