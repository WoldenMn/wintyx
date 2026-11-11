import { describe, it, expect } from "vitest";
import { getAdapterStatusLabel, getMemorySlotLabel, getTaskStateLabel } from "../labels";

describe("labels", () => {
  it("traduit les statuts de carte réseau connus et garde les autres", () => {
    expect(getAdapterStatusLabel("Up")).toBe("Actif");
    expect(getAdapterStatusLabel("Disconnected")).toBe("Déconnecté");
    expect(getAdapterStatusLabel("Inconnu-xyz")).toBe("Inconnu-xyz");
  });

  it("traduit les états de tâche planifiée connus et garde les autres", () => {
    expect(getTaskStateLabel("Ready")).toBe("Prête");
    expect(getTaskStateLabel("Running")).toBe("En cours");
    expect(getTaskStateLabel("Custom")).toBe("Custom");
  });

  it("distingue deux barrettes au même DeviceLocator par leur banque (données WMI du poste)", () => {
    const a = getMemorySlotLabel({ bankLabel: "P0 CHANNEL A", deviceLocator: "DIMM 1" }, 0);
    const b = getMemorySlotLabel({ bankLabel: "P0 CHANNEL B", deviceLocator: "DIMM 1" }, 1);
    expect(a).toBe("P0 Canal A, DIMM 1");
    expect(b).toBe("P0 Canal B, DIMM 1");
    expect(a).not.toBe(b);
  });

  it("retombe sur l'un des champs puis sur la position", () => {
    expect(getMemorySlotLabel({ bankLabel: "", deviceLocator: "DIMM 2" }, 3)).toBe("DIMM 2");
    expect(getMemorySlotLabel({ bankLabel: "BANK 0", deviceLocator: "" }, 3)).toBe("BANK 0");
    expect(getMemorySlotLabel({ bankLabel: " ", deviceLocator: "" }, 3)).toBe("Emplacement 4");
  });
});
