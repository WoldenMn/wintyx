import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { DefenderCard, defenderState } from "../DefenderCard";
import type { DefenderStatus } from "../../../lib/bindings";

function st(realTime: boolean): DefenderStatus {
  return { realTimeProtection: realTime, antivirusEnabled: realTime, amServiceEnabled: realTime,
    tamperProtected: realTime, signatureVersion: realTime ? "1.0" : null,
    signatureLastUpdated: realTime ? "2026-06-20 08:00" : null,
    signatureAgeDays: realTime ? 4 : null, lastScan: realTime ? "2026-06-22 09:00" : null };
}

describe("defenderState", () => {
  it("active quand protection temps reel", () => expect(defenderState(st(true), "Windows Defender")).toBe("active"));
  it("veille quand AV tiers actif", () => expect(defenderState(st(false), "Antivirus Tiers")).toBe("passive-thirdparty"));
  it("non protege quand pas d'AV tiers", () => {
    expect(defenderState(st(false), "")).toBe("unprotected");
    expect(defenderState(st(false), "Windows Defender")).toBe("unprotected");
    expect(defenderState(st(false), "Unknown")).toBe("unprotected");
    expect(defenderState(st(false), undefined)).toBe("unprotected");
  });
  it("indisponible si status absent", () => expect(defenderState(undefined, "Antivirus Tiers")).toBe("unavailable"));
});

describe("DefenderCard", () => {
  it("affiche protection active", () => {
    render(<DefenderCard status={st(true)} activeAntivirus="Windows Defender" loading={false} error={null} />);
    expect(screen.getByText(/protection active/i)).toBeInTheDocument();
  });
  it("affiche la veille avec le nom de l'AV tiers", () => {
    render(<DefenderCard status={st(false)} activeAntivirus="Antivirus Tiers" loading={false} error={null} />);
    expect(screen.getByText(/Antivirus Tiers/)).toBeInTheDocument();
    expect(screen.getByText(/en veille/i)).toBeInTheDocument();
  });
  it("affiche une alerte si non protege", () => {
    render(<DefenderCard status={st(false)} activeAntivirus="" loading={false} error={null} />);
    expect(screen.getByText(/aucune protection/i)).toBeInTheDocument();
  });
  it("affiche indisponible sur erreur", () => {
    render(<DefenderCard status={undefined} activeAntivirus="" loading={false} error="boom" />);
    expect(screen.getByText(/indisponible/i)).toBeInTheDocument();
  });
});
