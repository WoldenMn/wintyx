import { describe, it, expect } from "vitest";
import {
  getCategoryLabel,
  getHardeningLevelLabel,
  INVOKE_TIMEOUT_BACKEND_120S_MS,
  INVOKE_TIMEOUT_BACKEND_60S_MS,
  INVOKE_TIMEOUT_MS,
  invokeTimeoutFor,
} from "../constants";

describe("invokeTimeoutFor", () => {
  it("garde le delai par defaut pour une commande rapide ou inconnue", () => {
    expect(invokeTimeoutFor("get_windows_build")).toBe(INVOKE_TIMEOUT_MS);
    expect(invokeTimeoutFor("commande_inconnue")).toBe(INVOKE_TIMEOUT_MS);
  });

  it("donne aux commandes WMI au moins le delai backend de 60 s", () => {
    for (const c of ["get_driver_audit", "get_network_adapters", "get_disks", "get_system_info"]) {
      expect(invokeTimeoutFor(c)).toBeGreaterThanOrEqual(60_000);
    }
    expect(INVOKE_TIMEOUT_BACKEND_60S_MS).toBeGreaterThan(60_000);
  });

  it("donne scan, benchmark et rapport au moins le delai backend de 120 s", () => {
    for (const c of ["run_threat_scan", "run_benchmark", "generate_report"]) {
      expect(invokeTimeoutFor(c)).toBeGreaterThanOrEqual(120_000);
    }
    expect(INVOKE_TIMEOUT_BACKEND_120S_MS).toBeGreaterThan(120_000);
  });
});

describe("getCategoryLabel", () => {
  it("traduit les categories bloatware connues", () => {
    expect(getCategoryLabel("Gaming")).toBe("Jeux");
    expect(getCategoryLabel("Entertainment")).toBe("Divertissement");
    expect(getCategoryLabel("System")).toBe("Système");
  });

  it("traduit les categories durcissement avec espace", () => {
    expect(getCategoryLabel("Remote Access")).toBe("Accès distant");
    expect(getCategoryLabel("File Sharing")).toBe("Partage de fichiers");
  });

  it("traduit les categories protection des donnees (francais accentue)", () => {
    expect(getCategoryLabel("Telemetrie")).toBe("Télémétrie");
    expect(getCategoryLabel("Securite")).toBe("Sécurité");
    expect(getCategoryLabel("Publicite")).toBe("Publicité");
    expect(getCategoryLabel("Energie")).toBe("Énergie");
    expect(getCategoryLabel("Demarrage")).toBe("Démarrage");
  });

  it("retourne la valeur brute pour une categorie inconnue (fallback)", () => {
    expect(getCategoryLabel("UnknownCat")).toBe("UnknownCat");
  });
});

describe("getHardeningLevelLabel", () => {
  it("traduit les niveaux backend en francais", () => {
    expect(getHardeningLevelLabel("Essential")).toBe("Essentiel");
    expect(getHardeningLevelLabel("Recommended")).toBe("Recommandé");
  });

  it("retourne la valeur brute pour un niveau inconnu (fallback)", () => {
    expect(getHardeningLevelLabel("Other")).toBe("Other");
  });
});
