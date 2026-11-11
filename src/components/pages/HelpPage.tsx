import { useState, type ReactNode } from "react";
import { ChevronDown } from "lucide-react";
import { PageHeader } from "../../components/ui/PageHeader";

interface Section {
  id: string;
  title: string;
  content: ReactNode;
}

const SECTIONS: Section[] = [
  {
    id: "scores",
    title: "Comprendre les scores 0-100",
    content: (
      <>
        <p>
          Wintyx calcule 5 scores de santé et un score global qui est la moyenne
          équipondérée. Tous sont exprimés en pourcentage d'items au bon état.
        </p>
        <ul className="list-disc pl-6 space-y-2">
          <li>
            <strong>Protection des données</strong> : % d'items de vie privée activés
            (télémétrie, publicité, suivi, localisation). 21 items au total.
          </li>
          <li>
            <strong>Services</strong> : % de services Windows à risque arrêtés. Les
            services à risque élevé pèsent 3 points, moyens 2, faibles 1.
          </li>
          <li>
            <strong>Durcissement</strong> : % de corrections de sécurité appliquées
            (SMBv1, UAC, WDigest, LLMNR...). 16 contrôles au total, dont 8 essentiels.
          </li>
          <li>
            <strong>Performance</strong> : % d'optimisations appliquées (effets
            visuels, prefetch, plans d'énergie...). 10 items.
          </li>
          <li>
            <strong>Bloatware</strong> : % d'applications Microsoft pré-installées
            (Candy Crush, Xbox, Teams consumer...) désinstallées. 29 packages scannés.
          </li>
        </ul>
        <p className="mt-3">
          <strong>Astuce :</strong> cliquer sur n'importe quel score dans le Tableau
          de bord ouvre un détail qui liste les items avec leur statut.
        </p>
      </>
    ),
  },
  {
    id: "admin",
    title: "Mode administrateur",
    content: (
      <>
        <p>
          Certaines actions (arrêter/démarrer un service, appliquer un durcissement
          HKLM, restaurer un backup registre, créer un point de restauration)
          nécessitent des droits administrateur.
        </p>
        <p>
          Si Wintyx est lancé sans élévation, la StatusBar affiche{" "}
          <strong>"Mode non-admin"</strong> et les boutons concernés sont grisés avec
          un tooltip explicatif. Cliquer sur <strong>"Relancer en admin"</strong>{" "}
          dans la StatusBar relance Wintyx via le mécanisme UAC de Windows.
        </p>
        <p>
          Avant toute action batch destructive, Wintyx crée automatiquement un point
          de restauration système (si admin) et une sauvegarde des 7 clés registre
          critiques. Ces sauvegardes sont visibles dans la page{" "}
          <strong>Sauvegarde</strong>.
        </p>
      </>
    ),
  },
  {
    id: "shortcuts",
    title: "Raccourcis clavier",
    content: (
      <>
        <dl className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-2">
          <dt>
            <kbd className="px-2 py-0.5 bg-bg-elevated border border-border rounded text-xs font-mono">
              Ctrl+K
            </kbd>
          </dt>
          <dd>Ouvrir la palette de commandes (recherche + navigation rapide).</dd>

          <dt>
            <kbd className="px-2 py-0.5 bg-bg-elevated border border-border rounded text-xs font-mono">
              Tab
            </kbd>
          </dt>
          <dd>Naviguer entre éléments interactifs. Focus trap dans les dialogs.</dd>

          <dt>
            <kbd className="px-2 py-0.5 bg-bg-elevated border border-border rounded text-xs font-mono">
              Escape
            </kbd>
          </dt>
          <dd>Fermer dialog / annuler action en cours.</dd>

          <dt>
            <kbd className="px-2 py-0.5 bg-bg-elevated border border-border rounded text-xs font-mono">
              Enter
            </kbd>
          </dt>
          <dd>Activer le bouton qui a le focus dans un dialogue : Confirmer par défaut, Annuler pour une action dangereuse.</dd>
        </dl>
      </>
    ),
  },
  {
    id: "themes",
    title: "Thèmes visuels",
    content: (
      <>
        <p>2 thèmes disponibles, basculables en bas de la Sidebar :</p>
        <ul className="list-disc pl-6 space-y-1">
          <li>
            <strong>Obsidian</strong> : thème sombre par défaut, sobre, accent ion (cyan, #2EC4D6).
          </li>
          <li>
            <strong>Daylight</strong> : thème clair, net, accent ion (teal profond, #0B6E7C).
          </li>
        </ul>
        <p>
          Les deux thèmes respectent WCAG 2.1 AA (contraste du texte sur le fond d'au moins 4.5:1).
        </p>
      </>
    ),
  },
  {
    id: "confidence",
    title: "Actions destructives & confiance",
    content: (
      <>
        <p>Avant toute modification batch, Wintyx applique plusieurs garde-fous :</p>
        <ol className="list-decimal pl-6 space-y-1">
          <li>
            <strong>Point de restauration automatique</strong> (si admin) via{" "}
            <code className="text-xs bg-bg-elevated px-1 rounded">
              Checkpoint-Computer
            </code>
            .
          </li>
          <li>
            <strong>Export registre cible</strong> des 7 clés critiques dans{" "}
            <code className="text-xs bg-bg-elevated px-1 rounded">
              %LOCALAPPDATA%\Wintyx\Backups
            </code>
            .
          </li>
          <li>
            <strong>Confirmation obligatoire</strong> via un dialog avec description
            claire de ce qui sera modifié.
          </li>
          <li>
            <strong>Aperçu automatique</strong> sur les actions en lot compatibles :
            la confirmation liste les éléments qui seront touchés avant exécution.
          </li>
          <li>
            <strong>Bouton "Annuler" dans le toast</strong> pour les actions
            atomiques réversibles (Services, Startup) : inverse l'opération en un
            clic dans les 6 secondes.
          </li>
          <li>
            <strong>Historique tracé</strong> dans la page <strong>Logs</strong>,
            filtrable via le select "Portée &gt; Actions uniquement".
          </li>
        </ol>
      </>
    ),
  },
  {
    id: "troubleshooting",
    title: "Dépannage",
    content: (
      <>
        <dl className="space-y-3">
          <div>
            <dt className="font-medium text-fg-primary">
              Une section ne charge pas (ErrorBoundary affiche "Cette section n'a pas
              pu charger")
            </dt>
            <dd className="text-fg-dim text-sm mt-1">
              Cliquer sur "Réessayer". Si persistent, consulter la page Logs pour le
              détail technique. Le reste de l'app reste utilisable.
            </dd>
          </div>
          <div>
            <dt className="font-medium text-fg-primary">
              Une action échoue avec "nécessite des privilèges administrateur"
            </dt>
            <dd className="text-fg-dim text-sm mt-1">
              Cliquer sur "Relancer en admin" dans la StatusBar ou relancer Wintyx
              via clic droit &gt; "Exécuter en tant qu'administrateur".
            </dd>
          </div>
          <div>
            <dt className="font-medium text-fg-primary">
              WMI indisponible / les scans retournent vide
            </dt>
            <dd className="text-fg-dim text-sm mt-1">
              Le service "Windows Management Instrumentation" peut être désactivé.
              Dans les services Windows, vérifier que "Winmgmt" est démarré.
            </dd>
          </div>
          <div>
            <dt className="font-medium text-fg-primary">Restaurer après une action regrettable</dt>
            <dd className="text-fg-dim text-sm mt-1">
              Page <strong>Sauvegarde</strong> &gt; cliquer "Restaurer" sur le backup
              du jour. Ou ouvrir le dossier de sauvegarde (
              <code>%LOCALAPPDATA%\Wintyx\Backups</code>) et double-cliquer le
              fichier <code>.reg</code>.
            </dd>
          </div>
        </dl>
      </>
    ),
  },
];

export function HelpPage() {
  const [openId, setOpenId] = useState<string>(SECTIONS[0].id);

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Aide"
        subtitle="Guide utilisateur Wintyx : scores, raccourcis, actions destructives et dépannage"
      />
      <div className="grid grid-cols-2 gap-3 items-start">
        {SECTIONS.map((s) => {
          const expanded = openId === s.id;
          return (
            <section
              key={s.id}
              className="bg-bg-card border border-border rounded-lg theme-card overflow-hidden"
            >
              <button
                type="button"
                onClick={() => setOpenId(expanded ? "" : s.id)}
                aria-expanded={expanded}
                aria-controls={expanded ? `help-section-${s.id}` : undefined}
                className="w-full flex items-center justify-between px-5 py-4 text-left hover:bg-bg-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
              >
                <h2 className="text-base font-medium text-fg-primary">{s.title}</h2>
                <ChevronDown
                  size={16}
                  aria-hidden="true"
                  className={`text-fg-dim transition-transform ${expanded ? "rotate-180" : ""}`}
                />
              </button>
              {expanded && (
                <div
                  id={`help-section-${s.id}`}
                  className="px-5 pb-5 text-sm text-fg-dim leading-relaxed space-y-3 max-w-[75ch]"
                >
                  {s.content}
                </div>
              )}
            </section>
          );
        })}
      </div>
    </div>
  );
}

export default HelpPage;
