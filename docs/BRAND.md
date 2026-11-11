# Charte graphique Wintyx

Wintyx est un poste de pilotage : on l'ouvre souvent, on y lit un état, on agit, on vérifie. La charte sert cette boucle. Elle doit inspirer confiance sans faire de bruit.

Direction : **bento-dense**. Cartes de tailles inégales sur une grille régulière, une carte par question, la couleur réservée à l'état.

## Le symbole

Un W tracé d'un trait épais, dont le dernier jambage remonte plus haut que le premier et se lit comme une coche de validation. La moitié gauche est neutre, la moitié droite porte la couleur ion : l'état du système, puis la confirmation qu'il est sain.

| Fichier | Usage |
|---|---|
| `src-tauri/icons/source/wintyx.svg` | Master 1024 px sur tuile obsidienne. Source unique des icônes de l'application |
| `src-tauri/icons/source/mark-dark.svg`, `mark-light.svg` | Symbole sans tuile, pour fond sombre et fond clair |
| `src-tauri/icons/*.png`, `icon.ico`, `icon.icns` | Générés par `bun tauri icon src-tauri/icons/source/wintyx.svg`. L'ICO contient 16, 24, 32, 48, 64 et 256 px |
| `src-tauri/icons/installer/*.bmp` | Visuels NSIS et WiX, générés par `python scripts/brand/installer_art.py` (Pillow) |
| `public/favicon.svg` | Copie du master |
| `src/components/ui/BrandMark.tsx` | Symbole dans l'interface : barre de titre, écran d'analyse, À propos. Ses couleurs suivent le thème |
| `index.html` | Écran de démarrage, affiché avant que React ne monte |

Ne jamais retoucher une icône générée à la main : modifier le master, puis régénérer.

Règles :

- Zone de protection autour du symbole : un quart de sa largeur.
- Taille minimale : 16 px avec la tuile, 14 px sans.
- Ne pas recolorer la moitié gauche en ion, ne pas inverser les deux moitiés, ne pas ajouter d'ombre portée ni de contour.

## Le nom

« Wintyx » en Segoe UI Variable Display, graisse 600, interlettrage -0,01 em. Toujours une capitale initiale, jamais tout en capitales, jamais en italique.

## Couleurs

L'ion est la seule couleur de marque. Vert, ambre et rouge sont des couleurs d'état, jamais décoratives.

| Rôle | Token | Obsidian (sombre) | Daylight (clair) |
|---|---|---|---|
| Ion, accent | `--t-accent` | `#2EC4D6` | `#0B6E7C` |
| Accent au survol | `--t-accent-hover` | `#22AFC1` | `#095E6A` |
| Texte sur accent | `--t-on-accent` | `#0B0D11` | `#FFFFFF` |
| Texte accent sur fond teinté | `--t-accent-contrast` | `#7DE3EE` | `#095E6A` |
| Fond | `--t-bg-primary` | `#0E0F13` | `#FAFAFB` |
| Carte | `--t-bg-card` | `#16181F` | `#FFFFFF` |

Contrastes mesurés (WCAG) : accent sur fond 9,1:1 en sombre et 5,7:1 en clair ; texte sur accent 9,2:1 et 5,9:1 ; badge accent sur sa teinte 9,0:1 et 6,0:1.

Le blanc sur ion sombre tombe à 2,1:1. Un bouton sur accent porte donc `text-on-accent`, jamais `text-white`. Même règle pour les fonds d'état : `text-on-semantic`.

## Typographie

| Usage | Police |
|---|---|
| Interface | Segoe UI Variable Text, repli Segoe UI |
| Titres `h1` à `h3`, nom de marque | Segoe UI Variable Display (`font-display`) |
| Nombres | Même famille avec `tabular-nums`, posé sur `body` : un chiffre garde sa largeur quand il passe de 9 à 10 |
| Journal, chemins, code | Cascadia Mono, repli Consolas (`font-mono`) |

Polices du système uniquement : rien à embarquer, rien à télécharger, rendu natif de Windows 11.

## Mouvement

Le mouvement confirme une action, il ne décore pas. Courbe unique : `--ease-brand`, `cubic-bezier(0.2, 0.8, 0.2, 1)`.

| Interaction | Durée | Effet |
|---|---|---|
| Survol d'un bouton | 120 ms | Changement de fond ou de couleur, jamais d'ombre |
| Pression d'un bouton | 80 ms | Échelle 0,98 |
| Changement de page | 160 ms | Fondu et glissement de 4 px (`animate-page-in`) |
| Apparition d'un toast | 200 ms | Glissement depuis la droite |
| Anneau de score | 600 ms | Remplissage depuis zéro au premier affichage |
| Démarrage | en boucle | Barre de progression sous le symbole, jusqu'au montage de React |

Aucune animation permanente hors d'un traitement en cours. `prefers-reduced-motion` coupe tout.

## Installeurs

| Installeur | Visuel | Dimensions |
|---|---|---|
| NSIS | Bandeau d'accueil et de fin (`nsis-sidebar.bmp`) | 164 × 314 |
| NSIS | En-tête des pages (`nsis-header.bmp`) | 150 × 57 |
| WiX (MSI) | Bannière (`wix-banner.bmp`) | 493 × 58 |
| WiX (MSI) | Dialogue d'accueil (`wix-dialog.bmp`) | 493 × 312 |

La fenêtre de l'application démarre sur `#0E0F13` (`backgroundColor` dans `tauri.conf.json`) : pas d'éclair blanc avant le premier rendu.
