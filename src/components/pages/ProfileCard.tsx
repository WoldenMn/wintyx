import { useConfirmBatchAction } from "../../hooks/use-confirm-batch-action";
import { ActionButton } from "../../components/ui/ActionButton";
import { Badge } from "../../components/ui/Badge";
import { InfoTooltip } from "../../components/ui/InfoTooltip";
import { useScores } from "../../contexts/ScoresContext";
import type { Profile } from "../../lib/types";

interface ProfileCardProps {
  profile: Profile;
  onApplied: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
}

export function ProfileCard({ profile, onApplied, onEdit, onDelete }: ProfileCardProps) {
  const { refreshScores } = useScores();
  const totalItems =
    (profile.dataProtectionIds?.length ?? 0) +
    (profile.hardeningIds?.length ?? 0) +
    (profile.performanceIds?.length ?? 0) +
    (profile.servicesIds?.length ?? 0) +
    (profile.bloatwareIds?.length ?? 0) +
    (profile.startupIds?.length ?? 0);

  const batch = useConfirmBatchAction({
    command: "apply_profile",
    confirmTitle: `Appliquer "${profile.name}"`,
    confirmMessage: profile.description,
    confirmVariant: "warning",
    noun: { singular: "modification", plural: "modifications", feminine: true },
    args: { id: profile.id },
    snapshotActionName: `Profil ${profile.name}`,
    enablePreview: true,
    onSettled: () => {
      onApplied();
      void refreshScores();
    },
  });

  return (
    <div className="bg-bg-card border border-border rounded-lg p-5 theme-card flex flex-col gap-3">
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <h2 className="text-lg font-semibold text-fg-primary">{profile.name}</h2>
            {profile.isCustom && <Badge variant="accent">Personnalisé</Badge>}
          </div>
          <p className="text-xs text-fg-dim">{profile.description}</p>
        </div>
      </div>

      <div className="flex flex-wrap gap-2">
        <span className="inline-flex items-center gap-1">
          <Badge variant="info">{profile.dataProtectionIds?.length ?? 0} protections</Badge>
          <InfoTooltip textId="profile_data_protection" ariaLabel="Aide sur les protections de données" />
        </span>
        <span className="inline-flex items-center gap-1">
          <Badge variant="warning">{profile.hardeningIds?.length ?? 0} durcissements</Badge>
          <InfoTooltip textId="profile_hardening" ariaLabel="Aide sur les durcissements" />
        </span>
        <span className="inline-flex items-center gap-1">
          <Badge variant="accent">{profile.performanceIds?.length ?? 0} optimisations</Badge>
          <InfoTooltip textId="profile_performance" ariaLabel="Aide sur les optimisations" />
        </span>
        <span className="inline-flex items-center gap-1">
          <Badge variant="info">{profile.servicesIds?.length ?? 0} services</Badge>
          <InfoTooltip textId="profile_services" ariaLabel="Aide sur les services" />
        </span>
        <span className="inline-flex items-center gap-1">
          <Badge variant="warning">{profile.bloatwareIds?.length ?? 0} bloatwares</Badge>
          <InfoTooltip textId="profile_bloatware" ariaLabel="Aide sur les bloatwares" />
        </span>
        <span className="inline-flex items-center gap-1">
          <Badge variant="accent">{profile.startupIds?.length ?? 0} démarrage</Badge>
          <InfoTooltip textId="profile_startup" ariaLabel="Aide sur le démarrage" />
        </span>
      </div>

      <div className="flex justify-between items-center mt-auto pt-2 gap-2 flex-wrap">
        <span className="text-xs text-fg-muted">{totalItems} actions au total</span>
        <div className="flex gap-2 flex-wrap">
          {onEdit && (
            <ActionButton variant="neutral" onClick={onEdit}>Éditer</ActionButton>
          )}
          {onDelete && (
            <ActionButton variant="neutral" onClick={onDelete}>Supprimer</ActionButton>
          )}
          <ActionButton
            variant="accent"
            loading={batch.isRunning || batch.isPreviewing}
            onClick={batch.trigger}
          >
            Appliquer
          </ActionButton>
        </div>
      </div>
    </div>
  );
}
