import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../../hooks/use-invoke";
import { useConfirm } from "../../components/ui/ConfirmDialog";
import { useToast } from "../../components/ui/Toast";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageSkeleton } from "../../components/ui/Skeleton";
import { ActionButton } from "../../components/ui/ActionButton";
import { EmptyState } from "../../components/ui/EmptyState";
import { ErrorState } from "../../components/ui/ErrorState";
import { CloneProfileDialog } from "../../components/ui/CloneProfileDialog";
import { describeError } from "../../lib/helpers";
import { ProfileEditView } from "./ProfileEditView";
import { ProfileCard } from "./ProfileCard";
import type { Profile } from "../../lib/types";

const MAX_CUSTOMS = 20;

export function ProfilesPage() {
  const { data: profiles, isLoading, error, refresh } = useInvoke<Profile[]>("get_profiles");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [showCloneDialog, setShowCloneDialog] = useState(false);
  const confirm = useConfirm();
  const toast = useToast();

  if (isLoading) return <PageSkeleton kind="list" count={4} />;
  if (error) return <div className="p-7"><ErrorState message={error} onRetry={refresh} /></div>;
  if (!profiles) return <PageSkeleton kind="list" count={4} />;

  if (editingId !== null) {
    return (
      <ProfileEditView
        profileId={editingId}
        onSave={async () => {
          await refresh();
          setEditingId(null);
        }}
        onCancel={() => setEditingId(null)}
      />
    );
  }

  const presets = profiles.filter((p) => !p.isCustom);
  const customs = profiles.filter((p) => p.isCustom);
  const customsLimitReached = customs.length >= MAX_CUSTOMS;

  async function handleClone(sourceId: string, name: string) {
    try {
      await invoke<Profile>("create_custom_profile", { sourcePresetId: sourceId, name });
      toast.success("Profil cloné");
      await refresh();
      setShowCloneDialog(false);
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    }
  }

  async function handleDelete(id: string, name: string) {
    const ok = await confirm({
      title: `Supprimer "${name}" ?`,
      message: "Cette action est définitive. Le profil sera retiré de la liste.",
      variant: "danger",
    });
    if (!ok) return;
    try {
      await invoke("delete_custom_profile", { id });
      toast.success("Profil supprimé");
      await refresh();
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    }
  }

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Profils"
        subtitle="Appliquez en 1 clic un set cohérent d'optimisations + protections + durcissements."
      />

      <section className="mb-8">
        <h2 className="text-sm font-medium text-fg-muted uppercase tracking-wider mb-3">
          Profils prédéfinis ({presets.length})
        </h2>
        <div className="grid grid-cols-2 gap-4">
          {presets.map((p) => (
            <ProfileCard key={p.id} profile={p} onApplied={refresh} />
          ))}
        </div>
      </section>

      <section>
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-fg-muted uppercase tracking-wider">
            Mes profils ({customs.length} / {MAX_CUSTOMS})
          </h2>
          <ActionButton
            variant="accent"
            onClick={() => setShowCloneDialog(true)}
            disabled={customsLimitReached}
          >
            + Cloner un preset
          </ActionButton>
        </div>

        {customs.length === 0 ? (
          <EmptyState
            title="Aucun profil personnalisé"
            hint="Cliquez sur '+ Cloner un preset' pour créer votre premier profil personnalisé basé sur un preset existant."
          />
        ) : (
          <div className="grid grid-cols-2 gap-4">
            {customs.map((p) => (
              <ProfileCard
                key={p.id}
                profile={p}
                onApplied={refresh}
                onEdit={() => setEditingId(p.id)}
                onDelete={() => handleDelete(p.id, p.name)}
              />
            ))}
          </div>
        )}
      </section>

      {showCloneDialog && (
        <CloneProfileDialog
          presets={presets}
          onClone={handleClone}
          onCancel={() => setShowCloneDialog(false)}
        />
      )}
    </div>
  );
}

export default ProfilesPage;
