/**
 * Dictionnaire centralise des descriptions pedagogiques pour items techniques.
 *
 * Utilisation : `<InfoTooltip textId="smbv1" />` resout automatiquement le texte.
 * Fallback sur la cle si l'ID n'existe pas.
 */
export const HELP_TEXTS: Record<string, string> = {
  // Hardening
  smbv1:
    "Protocole de partage fichier ancien (1983). Vulnérable (WannaCry utilisait SMBv1). Désactivé, Windows utilise SMBv2/v3.",
  rdp:
    "Remote Desktop Protocol : permet la connexion à distance. À désactiver si non utilisé (port 3389, cible fréquente d'attaques).",
  uac:
    "User Account Control : demande confirmation avant toute action admin. Doit rester activé.",
  wdigest:
    "Ancien protocole d'authentification stockant les mots de passe en clair en mémoire. Désactivation recommandée.",
  llmnr:
    "Link-Local Multicast Name Resolution : résolution de nom sur réseau local. Désactivé sur réseau d'entreprise (attaques MITM/NBT-NS poisoning).",
  netbios:
    "Protocole réseau ancien. Vulnérabilité NBT-NS poisoning sur réseau local. Désactivé hors environnements legacy.",
  autorun:
    "Autorun/AutoPlay sur clés USB : désactivation recommandée pour éviter les malwares auto-exécutés.",
  spooler:
    "Print Spooler : vulnérable à PrintNightmare. À désactiver si aucune imprimante n'est utilisée.",

  // Data protection
  smartscreen:
    "Filtre URL/fichiers Microsoft. Envoie à Microsoft un hash des fichiers exécutés. Tradeoff télémétrie vs sécurité : Wintyx le classe comme télémétrie désactivable.",
  diagtrack:
    "Service de télémétrie Windows qui envoie des données d'utilisation à Microsoft. Désactivation recommandée pour la vie privée.",
  cortana:
    "Assistant vocal Microsoft. Envoie les requêtes à Microsoft. Désactivation n'affecte pas les fonctions système.",
  cortana_web_search:
    "Recherche Bing dans le menu Démarrer. Envoie chaque requête de recherche locale à Microsoft.",
  telemetry:
    "Collecte de données d'utilisation envoyées à Microsoft. Niveau minimal ne peut pas être totalement désactivé (Windows Pro/Home).",
  advertising_id:
    "Identifiant unique par utilisateur pour le suivi publicitaire cross-app. À désactiver.",
  location:
    "Accès à la localisation géographique pour les applications. À désactiver si non utilisé.",

  // Performance
  prefetch:
    "Windows précharge les exécutables fréquents pour accélérer le lancement. À désactiver sur SSD (bénéfice nul, I/O inutile).",
  superfetch:
    "Service qui précharge des données en RAM en anticipant l'usage. Sur SSD moderne, gain marginal voire contre-productif.",
  visual_effects:
    "Animations et effets visuels Windows. Désactivation libère quelques %CPU/GPU.",
  game_mode:
    "Priorité CPU pour les jeux. Peut causer des micro-freezes hors jeu.",
  nagle:
    "Algorithme réseau qui groupe les petits paquets TCP. Désactivation réduit la latence (gaming, streaming).",
  power_throttling:
    "Réduit la vitesse CPU des apps en arrière-plan. Désactivation = plus de perf mais plus de consommation.",

  // Scores
  score_global:
    "Mesure le % de recommandations Wintyx appliquées (moyenne équipondérée des 5 domaines : Protection données, Services, Durcissement, Performance, Bloatware), PAS un niveau de risque. 0% = configuration Windows par défaut, ce n'est pas un danger. Cliquer sur un domaine pour voir le détail.",
  score_data_protection:
    "% d'items de protection de la vie privée activés (télémétrie, publicité, suivi).",
  score_services:
    "% de services à risque Windows arrêtés. Pondéré par risque (High=3 points, Medium=2, Low=1).",
  score_hardening:
    "% de corrections de durcissement appliquées (SMBv1, UAC, WDigest, etc.).",
  score_performance:
    "% d'optimisations de performance appliquées.",
  score_bloatware:
    "% d'applications Microsoft préinstallées (jeux, Xbox, Teams consumer, etc.) désinstallées.",

  // Driver audit (v2.8.0)
  driver_duplicate:
    "Plusieurs versions du même pilote sont installées dans DriverStore. Une seule est active, les autres consomment de l'espace disque (~50-200 Mo chacune) et ralentissent l'inventaire au démarrage.",
  driver_phantom:
    "Périphérique déconnecté mais dont les pilotes et clés de registre persistent. Sans risque immédiat mais alourdit le système. Suppression sûre tant que le périphérique n'est pas rebranché.",
  driver_obsolete:
    "Pilote tiers (non-Microsoft) dont la date dépasse 2 ans. Peut contenir des failles de sécurité connues ou des incompatibilités avec les versions récentes de Windows.",
  driver_third_party:
    "Pilote fourni par le fabricant (OEM) plutôt que Microsoft. Souvent nécessaire et meilleur que les pilotes génériques, mais à surveiller pour la sécurité et les mises à jour.",
  driver_unsigned:
    "Pilote sans signature numérique valide. Peut être un pilote ancien, modifié ou malveillant. À investiguer en priorité.",

  // Profils (v2.9.0)
  profile_data_protection:
    "Désactive télémétrie, publicité ciblée, recherche web, Cortana. Réduit le partage de données personnelles avec Microsoft et les tiers.",
  profile_hardening:
    "Désactive SMBv1, durcit UAC, LLMNR, NetBIOS, WDigest. Réduit la surface d'attaque sans casser les usages courants.",
  profile_performance:
    "Désactive effets visuels superflus, Superfetch, indexation, plan d'énergie. Gain réactivité ressentie surtout sur machines anciennes.",
  profile_services:
    "Arrête les services Windows à risque élevé (Spooler si pas d'imprimante, Remote Registry, etc.). Audit à faire si erreurs.",
  profile_bloatware:
    "Désinstalle apps UWP préinstallées inutiles (Xbox, Météo, Cartes, Solitaire, etc.). Réinstallables via le Store.",
  profile_startup:
    "Désactive les programmes Microsoft+tiers lancés automatiquement au démarrage. Accélération du temps de boot.",
};

export function getHelpText(id: string): string | undefined {
  return HELP_TEXTS[id];
}
