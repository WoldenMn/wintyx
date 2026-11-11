import { useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../../hooks/use-invoke";
import { useToast } from "../../components/ui/Toast";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageSkeleton } from "../../components/ui/Skeleton";
import { ErrorState } from "../../components/ui/ErrorState";
import { ActionButton } from "../../components/ui/ActionButton";
import { ProfileItemSelector } from "../../components/ui/ProfileItemSelector";
import { describeError } from "../../lib/helpers";
import type { Profile, AvailableItems } from "../../lib/types";

interface Props {
  profileId: string;
  onSave: () => void;
  onCancel: () => void;
}

export function ProfileEditView({ profileId, onSave, onCancel }: Props) {
  const { data: profiles, isLoading: profilesLoading, error: profilesError, refresh: refreshProfiles } = useInvoke<Profile[]>("get_profiles");
  const { data: items, isLoading: itemsLoading, error: itemsError, refresh: refreshItems } = useInvoke<AvailableItems>("get_available_items");

  if (profilesLoading || itemsLoading) return <PageSkeleton kind="list" count={6} />;

  const loadError = profilesError || itemsError;
  if (loadError) {
    return (
      <div className="p-7 space-y-4">
        <ErrorState
          message={loadError}
          onRetry={() => {
            refreshProfiles();
            refreshItems();
          }}
        />
        <ActionButton variant="neutral" onClick={onCancel}>Retour</ActionButton>
      </div>
    );
  }

  const profile = profiles?.find((p) => p.id === profileId);
  if (!profile) {
    return (
      <div className="p-7">
        <PageHeader title="Profil introuvable" subtitle="Ce profil n'existe plus." />
        <ActionButton variant="neutral" onClick={onCancel}>Retour</ActionButton>
      </div>
    );
  }

  return <ProfileEditForm profile={profile} items={items} onSave={onSave} onCancel={onCancel} />;
}

interface FormProps {
  profile: Profile;
  items: AvailableItems | null;
  onSave: () => void;
  onCancel: () => void;
}

function ProfileEditForm({ profile, items, onSave, onCancel }: FormProps) {
  const [name, setName] = useState(profile.name);
  const [description, setDescription] = useState(profile.description);
  const [dpIds, setDpIds] = useState<Set<string>>(new Set(profile.dataProtectionIds));
  const [hardIds, setHardIds] = useState<Set<string>>(new Set(profile.hardeningIds));
  const [perfIds, setPerfIds] = useState<Set<string>>(new Set(profile.performanceIds));
  const [servicesIds, setServicesIds] = useState<Set<string>>(new Set(profile.servicesIds ?? []));
  const [bloatwareIds, setBloatwareIds] = useState<Set<string>>(new Set(profile.bloatwareIds ?? []));
  const [startupIds, setStartupIds] = useState<Set<string>>(new Set(profile.startupIds ?? []));
  const [isSaving, setIsSaving] = useState(false);
  const toast = useToast();

  const totalSelected = dpIds.size + hardIds.size + perfIds.size
                      + servicesIds.size + bloatwareIds.size + startupIds.size;
  const canSave = useMemo(
    () => name.trim().length > 0 && totalSelected > 0 && !isSaving,
    [name, totalSelected, isSaving],
  );

  function toggle(set: Set<string>, id: string, setter: (s: Set<string>) => void) {
    const next = new Set(set);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setter(next);
  }

  async function handleSave() {
    setIsSaving(true);
    try {
      await invoke<Profile>("update_custom_profile", {
        profile: {
          ...profile,
          name: name.trim(),
          description,
          dataProtectionIds: Array.from(dpIds),
          hardeningIds: Array.from(hardIds),
          performanceIds: Array.from(perfIds),
          servicesIds: Array.from(servicesIds),
          bloatwareIds: Array.from(bloatwareIds),
          startupIds: Array.from(startupIds),
          isCustom: true,
        },
      });
      toast.success("Profil enregistré");
      onSave();
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title={`Éditer : ${profile.name}`}
        subtitle="Personnalisez le nom, la description et les actions incluses."
      />

      <div className="bg-bg-card border border-border rounded-lg theme-card p-5 mb-6">
        <div className="space-y-3">
          <div>
            <label htmlFor="edit-name" className="block text-xs text-fg-muted mb-1">
              Nom
            </label>
            <input
              id="edit-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              maxLength={80}
              className="w-full px-3 py-2 bg-bg-input border border-border rounded-md text-sm text-fg-primary focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
            />
          </div>
          <div>
            <label htmlFor="edit-description" className="block text-xs text-fg-muted mb-1">
              Description
            </label>
            <textarea
              id="edit-description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              maxLength={200}
              rows={2}
              className="w-full px-3 py-2 bg-bg-input border border-border rounded-md text-sm text-fg-primary resize-none focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
            />
          </div>
        </div>
      </div>

      <ProfileItemSelector
        title="Protection des données"
        items={items?.dataProtection ?? []}
        selectedIds={dpIds}
        onToggle={(id) => toggle(dpIds, id, setDpIds)}
      />
      <ProfileItemSelector
        title="Durcissement"
        items={items?.hardening ?? []}
        selectedIds={hardIds}
        onToggle={(id) => toggle(hardIds, id, setHardIds)}
      />
      <ProfileItemSelector
        title="Performance"
        items={items?.performance ?? []}
        selectedIds={perfIds}
        onToggle={(id) => toggle(perfIds, id, setPerfIds)}
      />
      <ProfileItemSelector
        title="Services Windows"
        items={items?.services ?? []}
        selectedIds={servicesIds}
        onToggle={(id) => toggle(servicesIds, id, setServicesIds)}
      />
      <ProfileItemSelector
        title="Bloatware (apps préinstallées)"
        items={items?.bloatware ?? []}
        selectedIds={bloatwareIds}
        onToggle={(id) => toggle(bloatwareIds, id, setBloatwareIds)}
      />
      <ProfileItemSelector
        title="Démarrage"
        items={items?.startup ?? []}
        selectedIds={startupIds}
        onToggle={(id) => toggle(startupIds, id, setStartupIds)}
      />

      <div className="sticky bottom-0 z-10 -mx-7 -mb-7 mt-6 bg-bg-card border-t border-border px-7 py-3 shadow-lg">
        <div className="flex items-center gap-3 flex-wrap">
          <span className="text-sm text-fg-primary">
            {totalSelected} action(s) sélectionnée(s)
          </span>
          <div className="flex gap-2 ml-auto">
            <ActionButton variant="neutral" onClick={onCancel} disabled={isSaving}>Annuler</ActionButton>
            <ActionButton variant="accent" onClick={handleSave} disabled={!canSave} loading={isSaving}>
              Enregistrer
            </ActionButton>
          </div>
        </div>
      </div>
    </div>
  );
}
