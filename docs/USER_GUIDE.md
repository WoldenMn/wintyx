# Guide utilisateur Wintyx

Wintyx est une application de maintenance et de protection pour Windows 10/11.
Ce guide presente chaque page et explique comment interpreter les resultats.

## Installation et premier lancement

### Version portable (recommandee)

1. Lancer `Install/Wintyx.exe`
2. Au premier lancement, un scan initial s'execute (5-10 secondes)
3. La page `Tableau de bord` s'affiche avec le score de sante

### Version installeur

Executer `Install/Wintyx_x.y.z_x64-setup.exe` puis suivre l'assistant NSIS.

### Droits administrateur

Certaines actions modifient le systeme et requierent des droits administrateur :

- Arret/demarrage de services Windows
- Application de regles de durcissement (HKLM)
- Creation de points de restauration
- Backup/restore de cles HKLM
- Desactivation de programmes au demarrage (HKLM)

Pour ces actions, lancer Wintyx via clic droit → `Executer en tant qu'administrateur`.
Si ce n'est pas le cas, un message d'erreur explicite indiquera que les privileges
administrateur sont requis.

## Navigation

| Raccourci | Action |
|-----------|--------|
| `Ctrl+K` | Palette de commandes (recherche rapide de page ou d'action, sans tenir compte de la casse ni des accents) |
| `Tab` | Naviguer entre les elements interactifs |
| `Shift+Tab` | Naviguer en arriere |
| `Escape` | Fermer une modale / annuler une action |
| `Enter` | Confirmer l'action par defaut dans une modale |

La sidebar gauche contient 25 pages. Le `Tableau de bord` est en haut, les quatre groupes suivants sont au milieu, et `Diagnostic`, `Aide` et `A propos` sont ancres en bas :

- **OPTIMISATION** : Profils, Nettoyage, Performance, Demarrage, Bloatware
- **SECURITE** : Durcissement, Services, Protection donnees, Scanner de menaces
- **SYSTEME** : Monitoring live, Sante systeme, Reseau, Disques, Pilotes, Memoire
- **OUTILS** : Mises a jour, Taches planifiees, Sauvegarde, Comparer snapshots, Logs, Benchmark

Un badge sur une entree de la sidebar signale un point a regarder : par exemple le nombre de taches suspectes, ou « ? » quand Wintyx n'a pas pu les verifier.

## Etats que vous pouvez rencontrer

Quand Wintyx n'arrive pas a mesurer quelque chose, il le dit au lieu d'afficher « securise » ou « 0 ». Voici les etats que vous pouvez voir :

| Etat affiche | Ou | Ce que cela veut dire |
|--------------|----|-----------------------|
| **Inconnu** | Durcissement, Services, Performance, detail des scores | Wintyx n'a pas pu lire l'etat (sortie illisible, commande en echec, droits manquants). Le controle est exclu du score et des lots « Appliquer », et n'a pas de bouton Corriger. Relancer en administrateur donne souvent la reponse |
| **Configuration manuelle** | Durcissement | Le controle ne peut pas etre corrige d'un clic (Credential Guard, par exemple : il demande la strategie de groupe, l'UEFI, Secure Boot et la virtualisation). Il est exclu de « Appliquer essentiels » et « Appliquer tout » |
| **Non installe** | Services | Le service n'existe pas sur ce Windows. Il compte comme satisfait dans le score et le bouton Demarrer est masque |
| **Inaccessible sans droits administrateur** | Nettoyage | Wintyx ne peut pas lire le dossier sans elevation, il n'affiche donc pas « 0 B ». Le bouton de la cible est desactive |
| **Non analyse** | Scanner de menaces | Une phase du scan a echoue (delai, erreur de commande). Un bandeau signale que l'analyse est incomplete |
| **Indisponible** / **Indeterminee** | Mises a jour, Reseau, Sauvegarde | La valeur n'a pas pu etre lue. La cause s'affiche avec un bouton « Reessayer » |

Une erreur de chargement affiche un bloc rouge avec « Reessayer » et, quand la cause est un prerequis de Wintyx, un lien « Diagnostiquer » vers la page Diagnostic. Les erreurs de l'interface sont aussi ecrites dans le journal (page `Logs`, categorie `frontend`).

## Comprendre les scores

Wintyx calcule des scores de 0 a 100 sur plusieurs axes :

| Score | Signification |
|-------|---------------|
| 75-100 | Bon (vert) — aucune action urgente |
| 40-74 | Moyen (orange) — ameliorations possibles |
| 0-39 | Faible (rouge) — actions recommandees |

Le score global est affiche dans la `StatusBar` (bas d'ecran) et sur le `Tableau de bord`.

## Pages

### Tableau de bord

Vue d'ensemble avec :
- Score global de sante
- Score par categorie (Securite, Performance, Nettoyage)
- Informations systeme essentielles (OS, CPU, RAM)
- Bouton `Generer un rapport HTML` (sauvegarde dans `%APPDATA%\Wintyx\Reports\`)

### Sante systeme (Health)

Anneaux CPU et RAM avec sparkline, uptime et informations systeme. C'est un etat a l'ouverture ; pour un suivi continu, voir Monitoring live. Si le monitoring ne demarre pas, la page affiche l'erreur avec « Reessayer » au lieu d'un faux 0 %.

### Monitoring live

Suivi temps reel avec streaming Tauri Events :
- Usage CPU global + par coeur
- RAM utilisee / totale (GB, pourcentage)
- Top 10 processus (CPU %, memoire Mo)

Intervalle configurable (1 s a 5 s au curseur, defaut 1,5 s : une collecte dure environ 0,4 s, un intervalle plus court ne serait pas tenu). La page affiche l'intervalle reel mesure entre deux mesures recues. Bouton `Arreter` / `Demarrer`. Si le monitoring echoue, un bloc d'erreur propose « Reessayer ».

### Profils

Un profil est un ensemble coherent de reglages appliques en un clic sur six domaines : protections de donnees, durcissements, optimisations de performance, services, bloatware et demarrage.

- **Presets** : 4 profils fournis, non modifiables. Chaque carte indique combien d'elements de chaque domaine elle touche.
- **Mes profils** : jusqu'a 20 profils personnalises. `+ Cloner un preset` en cree un a partir d'un preset ; `Modifier` ouvre l'editeur (nom, description, elements cochables par domaine) ; `Supprimer` retire le profil apres confirmation.
- **Appliquer** : demande confirmation, prend un snapshot du registre avant d'ecrire, puis rapporte le resultat element par element. Un element absent ou refuse est liste dans le compte rendu, il n'interrompt pas les autres.
- Le nom et la description d'un profil n'acceptent pas les caracteres de controle.

### Nettoyage

Identifie les fichiers temporaires, caches, logs, corbeille.
- **Scanner** : calcule la taille recuperable
- **Nettoyer** : supprime une categorie individuelle (avec confirmation)
- **Tout nettoyer** : batch operation sur toutes les categories
- **Vider le cache DNS** : `ipconfig /flushdns`

### Performance

Optimisations registre et powercfg pour les performances :
- Effets visuels desactivables
- Plan d'alimentation haute performance
- Services d'indexation et Superfetch
- Bouton `Optimiser tout` pour appliquer l'ensemble

### Demarrage (Startup)

Liste des programmes lances au demarrage (HKLM/HKCU + dossier Startup).
- `Desactiver` : deplace la cle dans une zone de backup
- `Activer` : restaure depuis le backup

### Bloatware

Applications UWP pre-installees (AppX). Suggestions de suppression par categorie
(Xbox, Weather, Maps, etc.). Suppression via `Remove-AppxPackage` PowerShell.

### Durcissement (Hardening)

Regles de securite Windows applicables :
- Desactivation SMBv1
- Protection contre les macros Office
- Durcissement des ports d'ecoute
- Politiques UAC strictes

Chaque regle affiche son etat courant + bouton `Appliquer`. Mode `Essentiel`
applique un sous-ensemble recommande.

### Services

Services Windows avec leur niveau de risque.
- Informations : nom, etat (Actif / Arrete / Non installe / Inconnu), type de demarrage, risque (Eleve / Moyen / Faible)
- Actions : `Stop` / `Start` (requiert administrateur)
- Score base sur le ratio de services a risque eleve arretes

### Protection donnees (Data Protection)

Reglages de confidentialite (telemetrie, publicite, recherche web, etc.) regroupes
par categorie. Bouton `Proteger` par item + `Tout proteger`.

### Scanner de menaces

Scan multi-source :
- Taches planifiees suspectes (meme analyse que la page Taches : seuls les indices forts, pas les taches « A verifier »)
- Ports en ecoute dont l'executable n'est pas signe (gravite selon son dossier), ou ports de portes derobees connues ; un port en ecoute n'est pas une menace en soi
- Services non signes
- Entrees registre suspectes
- Regles firewall ouvertes

Score de risque 0-100 + details par item.

### Reseau

- Adaptateurs reseau (IP, masque, passerelle)
- Regles firewall (entrantes / sortantes)
- Test de ping (Google, Cloudflare, Microsoft)
- Connectivite internet (bouton Tester) et IP publique, lue a la demande aupres d'api.ipify.org (bouton Afficher) : la page ne contacte aucun service a l'ouverture

### Disques

Volumes logiques : lettre, systeme de fichiers, taille totale / libre, usage %.

### Pilotes (Drivers)

Audit complet des pilotes installes avec 5 categories detectees :

- **Doublons DriverStore** : versions multiples du meme pilote presentes dans `pnputil /enum-drivers`. Wintyx groupe par fournisseur/classe/inf et identifie la version la plus recente.
- **Peripheriques fantomes** : peripheriques deja desinstalles mais dont les pilotes restent en cache (`Get-PnpDevice -PresentOnly:$false`). Action batch `Nettoyer fantomes` disponible (requiert administrateur).
- **Pilotes obsoletes** : DriverDate > 2 ans ET fournisseur non-Microsoft.
- **Pilotes OEM tiers** : fournisseur et signataire non-Microsoft.
- **Pilotes non signes** : signature numerique manquante (`Win32_PnPSignedDriver.IsSigned = false`).

6 StatCards cliquables filtrent la liste sur une categorie. Recherche texte disponible. Tooltips pedagogiques (icone `i`) expliquent chaque categorie.

**Actions destructives** :

- `Desinstaller` (par pilote) : `pnputil /delete-driver <oem*.inf>` avec snapshot auto + confirmation.
- `Supprimer fantome` (par peripherique) : `pnputil /remove-device <instanceId>`.
- `Nettoyer fantomes` (batch en header) : iterer sur tous les peripheriques fantomes detectes.

Toutes les actions destructives requierent les droits administrateur. Si Wintyx n'est pas lance en admin, les boutons concernes sont grises avec un tooltip explicatif.

### Memoire

- Barrettes RAM (fabricant, capacite, vitesse, slot)
- Top consommateurs RAM (processus)
- Resume : total, utilise, libre, cache

### Mises a jour (Updates)

Historique Windows Update (hotfix, cumulatif, securite) via `Get-HotFix`.

### Taches (Tasks)

Taches planifiees. Vue separee `Suspectes` (non-Microsoft + chemins inhabituels).
Action `Desactiver` disponible.

### Sauvegarde (Backup)

- Points de restauration systeme existants, listes sur la page (administrateur requis : en session standard la page l'indique au lieu de rester vide)
- Creer un point de restauration (requiert administrateur). Windows n'en cree pas plus d'un toutes les 24 h : dans ce cas Wintyx affiche « non cree : limite Windows de 24 h » au lieu d'un succes
- Export des cles critiques et import de sauvegardes de registre (`.reg`). L'import refuse un fichier qui supprime des cles ou des valeurs, qui touche une cle hors des cles critiques exportees par Wintyx, ou qui n'est ni en UTF-16 avec BOM ni en UTF-8
- Etat de la protection du systeme lu dans la strategie : « Non desactivee », « Desactivee par strategie » ou « Indeterminee » si la lecture a echoue. L'etat reel par volume se lit dans Parametres > Systeme > Protection du systeme

### Comparer snapshots

Un snapshot est un lot d'exports `.reg` pris automatiquement avant une action de lot (durcissement, optimisation, profil, restauration, etc.). Cette page montre ce qui a change depuis.

1. Choisir le mode : `Vs etat actuel` (le snapshot contre le registre d'aujourd'hui) ou `Vs autre snapshot` (deux snapshots entre eux).
2. Choisir le ou les snapshots, puis `Comparer`.
3. Filtrer les differences : tout, modifie, supprime, ajoute.
4. Cocher les valeurs a remettre comme dans le snapshot, puis `Restaurer`. Wintyx demande confirmation, prend un nouveau snapshot, applique au plus 50 changements par lot et rapporte le resultat.

Sans snapshot, la page l'explique : lancez une action de lot pour en creer un. Restaurer des valeurs HKLM demande les droits administrateur.

### Diagnostic

Verifie les quatre prerequis dont dependent les pages de Wintyx :

| Controle | Echec ou alerte | Pages touchees |
|----------|-----------------|----------------|
| WMI joignable | Echec | Systeme, Memoire, Disques, Pilotes, Sante systeme, Monitoring live |
| PowerShell disponible | Echec | Reseau, Scanner de menaces, Taches planifiees, Mises a jour, Bloatware |
| Droits administrateur | Attention (jamais critique) | Durcissement, Services, Protection donnees |
| Stockage `%LOCALAPPDATA%` | Echec (dossier inaccessible ou non inscriptible) ou attention (peu d'espace) | Sauvegarde, Logs, historique des scores |

Chaque ligne affiche OK, Attention ou Echec. `Relancer` refait l'analyse. Une banniere apparait en haut de l'application quand un prerequis critique tombe, et les etats d'erreur des pages proposent « Diagnostiquer ».

### Aide

Guide integre, sections depliables : comprendre les scores 0-100, mode administrateur, raccourcis clavier, themes, actions destructives et confiance, depannage.

### Logs

Affiche les logs Wintyx (`%LOCALAPPDATA%\Wintyx\Logs\`).
Filtres par niveau (All / Info / Warning / Error) + recherche texte.

### Benchmark

Test synthetique de performance :
- CPU : calculs flottants sur N iterations
- RAM : allocation/ecriture memoire
- Disk : lecture/ecriture sequentielle
- Reseau : latence vers serveurs de reference

Sauvegarde des resultats dans un historique JSON (`%APPDATA%\Wintyx\benchmarks.json`).
L'historique reste visible quand un resultat est affiche. Chaque entree porte la version de Wintyx qui a mesure ; une mesure d'une version anterieure a 2.18.0 (metrique CPU differente) est signalee « non comparable ». L'estimation affichee pendant le test reprend la duree de la derniere mesure.
Permet de comparer avant/apres une optimisation.

### Animations de feedback (v2.1.2+)

Quand vous appliquez une action sur un item (Proteger, Optimiser, Supprimer un bloatware, etc.), 4 indicateurs visuels coordonnes vous confirment que l'action a ete prise en compte :

1. **Bouton qui se contracte** au clic (100ms) — retour tactile immediat.
2. **Halo vert subtil** autour de l'item modifie (800ms) — confirmation visuelle.
3. **Badge qui crossfade** entre les variantes (rouge → vert, etc.) (200ms) — transition d'etat fluide.
4. **Score qui grimpe en compte** (200ms) — feedback du score domaine apres l'action.

Ces animations respectent la preference systeme `prefers-reduced-motion` (v2.1.3+) : si vous l'avez activee, les transitions sont reduites a 0.01ms (visuellement instantanees).

Pages concernees : Services, Demarrage, Protection des donnees, Performance, Bloatware.

## Themes

2 themes signature disponibles via la bascule en bas de la barre laterale :

- **Obsidian** (defaut) : sombre, sobre, accent ion (turquoise)
- **Daylight** : clair, net, accent ion (turquoise fonce)

Le theme est persiste dans `localStorage`.

## Accessibilite

Wintyx respecte les recommandations WCAG 2.1 niveau AA :

- Navigation clavier complete (sidebar, pages, dialogs, palette de commandes)
- Lien `Aller au contenu principal` au premier focus
- Focus visible sur tous les elements interactifs
- Contraste texte/fond >= 4.5:1 sur les 2 themes
- Toasts d'erreur annonces aux lecteurs d'ecran (`aria-live="assertive"`)
- Tables avec legendes pour lecteurs d'ecran
- Focus trap dans les modales + focus restore sur fermeture

## Stockage local

| Chemin | Contenu |
|--------|---------|
| `%LOCALAPPDATA%\Wintyx\Logs\` | Logs applicatifs (rotation 7 jours, >10 Mo) |
| `%LOCALAPPDATA%\Wintyx\Backups\` | Instantanes et exports du registre (`.reg`) |
| `%LOCALAPPDATA%\Wintyx\RegistryBackups\` | Trace texte de l'ancienne valeur avant chaque ecriture ou suppression dans le registre |
| `%LOCALAPPDATA%\Wintyx\scores_history.json`, `benchmarks.json` | Historique des scores (90 jours) et des benchmarks |
| `%APPDATA%\Wintyx\custom_profiles.json` | Profils personnalises |
| Bureau | Rapports HTML generes depuis le Tableau de bord (`Wintyx_Report_<date>.html`) : ils contiennent le nom de la machine, l'utilisateur et les adresses reseau, a relire avant de les partager |

Aucune donnee n'est envoyee a l'exterieur. Seules sorties reseau : un `ping` (test de connectivite, benchmark) et, sur clic, la lecture de l'IP publique aupres d'`api.ipify.org` (page Reseau).

## Depannage

### L'application refuse de faire une action

Cause probable : droits administrateur manquants. Relancer Wintyx via
`Executer en tant qu'administrateur`.

### Les metriques Monitor restent a zero

Verifier que le service `Windows Management Instrumentation` (Winmgmt) tourne :
```
services.msc → Windows Management Instrumentation → Demarrer
```

### Le benchmark bloque sur le test reseau

Le test ping peut etre bloque par un firewall ou un VPN. Le benchmark passe
a une latence maximale par defaut apres timeout.

### Un nettoyage a efface un fichier que je voulais garder

Le nettoyage ne supprime que des fichiers temporaires, des caches et le contenu de la corbeille, et il demande confirmation. Wintyx ne prend pas de snapshot avant un nettoyage : un export de registre ne restaure pas des fichiers. Un fichier vide de la corbeille ou d'un dossier temporaire ne revient pas par la page `Sauvegarde`. Les autres actions de lot (durcissement, optimisation, bloatware, profils) prennent un snapshot et restent annulables depuis `Comparer snapshots`.

### Je veux reinitialiser les reglages Wintyx

Fermer Wintyx et supprimer :
- `%APPDATA%\Wintyx\`
- `%LOCALAPPDATA%\Wintyx\`

Les backups de registre y restent accessibles tant que ces dossiers existent.

## Support

Issues et suggestions : voir le depot GitHub du projet.
Logs utiles : `%LOCALAPPDATA%\Wintyx\Logs\` (joindre a tout rapport de bug).
