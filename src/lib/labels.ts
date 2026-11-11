const ADAPTER_STATUS_FR: Record<string, string> = {
  Up: "Actif",
  Down: "Inactif",
  Connected: "Connecté",
  Disconnected: "Déconnecté",
  Disabled: "Désactivé",
  "Not Present": "Absent",
  Unknown: "Inconnu",
};

const TASK_STATE_FR: Record<string, string> = {
  Ready: "Prête",
  Running: "En cours",
  Disabled: "Désactivée",
  Queued: "En attente",
  Unknown: "Inconnu",
};

export function getAdapterStatusLabel(status: string): string {
  return ADAPTER_STATUS_FR[status] ?? status;
}

export function getTaskStateLabel(state: string): string {
  return TASK_STATE_FR[state] ?? state;
}

export function getMemorySlotLabel(
  stick: { bankLabel: string; deviceLocator: string },
  index: number,
): string {
  const locator = stick.deviceLocator.trim();
  const bank = stick.bankLabel.trim().replace(/CHANNEL/i, "Canal");
  if (locator && bank) return `${bank}, ${locator}`;
  return locator || bank || `Emplacement ${index + 1}`;
}
