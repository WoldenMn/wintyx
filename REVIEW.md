# Review instructions — Wintyx

Application Tauri : frontend TypeScript/React, backend Rust dans `src-tauri/`. La CI est complète et sévère — `bun run lint`, `bun run build`, `bun run test`, `cargo build`, `cargo clippy -- -D warnings`, `cargo test`. Elle attrape le style, les types et les lints Rust. **La revue ne doit donc pas refaire son travail** : elle vise ce qu'aucun lint ne voit.

## Ce que « Important » veut dire ici

### La frontière Tauri

- Une commande `#[tauri::command]` qui prend un **chemin de fichier venant du frontend** et l'utilise sans le contraindre à un dossier attendu.
- Une commande qui exécute un binaire, ouvre une URL ou écrit hors du dossier de données de l'app.
- Une permission élargie dans les capabilities Tauri sans justification dans la PR.
- Un secret rendu au frontend alors qu'il pouvait rester côté Rust.

### Erreurs avalées

`clippy -D warnings` attrape beaucoup, mais pas tout : un `let _ = …` sur un `Result` porteur passe le lint, et c'est exactement le motif à signaler. De même côté TypeScript pour `catch {}` vide et `.catch(() => {})`. Un faux vert coûte plus cher qu'un crash.

### Concurrence

C'est là que la CI est le plus aveugle et que le coût d'un défaut est le plus élevé.

- Un `Mutex` ou `RwLock` verrouillé de part et d'autre d'un `.await`.
- Deux verrous pris dans des ordres différents selon le chemin d'appel.
- Un état partagé entre commandes Tauri modifié sans synchronisation.
- Un `spawn` dont le résultat n'est jamais joint ni observé.

### Le reste

- Fichier au-delà de 300 lignes sans motif dit dans la PR.
- Chemin en dur avec séparateur POSIX : ce projet cible Windows desktop.
- Une régression de performance sur un chemin mesuré — les scripts `perf:report` et `perf:clear` existent, donc les chiffres sont attendus quand la PR touche un chemin chaud.

## Plafond de nits

Trois. Volontairement plus bas qu'ailleurs, parce que la CI couvre déjà le style et les lints : un nit qui survit à `clippy -D warnings` et à `bun run lint` est soit réel, soit du bruit. Au-delà de trois, une ligne de décompte.

## Ne pas signaler

| Chemin | Motif |
|---|---|
| `src-tauri/target/**`, `node_modules/**`, `dist/**` | Générés |
| `src-tauri/gen/**` | Généré par Tauri |
| `Install/**` | Artefacts d'installateur |
| `bun.lock`, `Cargo.lock` | Lockfiles |

**Règle** : ne rien signaler que `cargo clippy -- -D warnings`, `bun run lint` ou `bun run build` remontent déjà.** La CI bloque la PR dessus : le redire coûte un aller-retour et fait passer la revue pour du bruit. En cas de doute sur la couverture d'un lint, dire « peut-être déjà couvert par clippy » plutôt que de poster.

## Toujours vérifier

- **Chaque `#[tauri::command]` ajoutée ou modifiée** : ce qu'elle accepte du frontend et ce qu'elle en fait.
- **Chaque `unsafe`** : justification écrite au-dessus du bloc, ou c'est Important.
- **Chaque dépendance ajoutée** : ce qu'elle apporte que le code existant ne faisait pas.
- **Plateforme** : Windows desktop uniquement. Pas de responsive, pas de media queries sous `lg`, Segoe UI, raccourcis `Ctrl`.

## Barre de preuve

Toute affirmation de comportement se cite en `fichier:ligne`. Ne jamais déduire d'un nom de fonction ce qu'elle fait.

Pour un défaut de concurrence, citer les deux points : le verrou et le point de suspension, ou les deux ordres de prise. Un interblocage annoncé sans les deux lignes est dégradé en avertissement — c'est la famille de faux positifs la plus coûteuse à instruire.

## Re-review

Deuxième passe et suivantes : Important uniquement. Les nits de la première passe ne sont pas re-postés.

## Forme du résumé

Ouvrir par un décompte d'une ligne, du type `1 concurrence, 0 frontière Tauri, 2 nits`. Quand rien de sérieux ne sort, le dire d'abord, sans liste de consolation.
