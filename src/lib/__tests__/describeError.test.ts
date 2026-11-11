import { describe, it, expect } from "vitest";
import { describeError, isCommandError } from "../helpers";

describe("isCommandError", () => {
  it("returns false for string", () => {
    expect(isCommandError("boom")).toBe(false);
  });

  it("returns false for null", () => {
    expect(isCommandError(null)).toBe(false);
  });

  it("returns false for plain object without code", () => {
    expect(isCommandError({ message: "hi" })).toBe(false);
  });

  it("returns true for CommandError shape", () => {
    expect(isCommandError({ code: "NotAdmin", details: "feature" })).toBe(true);
  });
});

describe("describeError", () => {
  it("falls back to String(e) for legacy string errors", () => {
    expect(describeError("legacy msg")).toBe("legacy msg");
  });

  it("handles NotAdmin", () => {
    const msg = describeError({ code: "NotAdmin", details: "Arreter un service" });
    expect(msg).toContain("Arreter un service");
    expect(msg).toContain("administrateur");
  });

  it("handles WmiUnavailable", () => {
    const msg = describeError({ code: "WmiUnavailable", details: "HRESULT 0x80041" });
    expect(msg).toContain("WMI");
    expect(msg).toContain("HRESULT 0x80041");
  });

  it("handles Timeout with structured details", () => {
    const msg = describeError({
      code: "Timeout",
      details: { command: "reg.exe", timeoutMs: 30000 },
    });
    expect(msg).toContain("reg.exe");
    expect(msg).toContain("30000");
  });

  it("handles NotFound", () => {
    expect(describeError({ code: "NotFound", details: "service X" })).toContain(
      "service X",
    );
  });

  it("handles Registry", () => {
    expect(
      describeError({ code: "Registry", details: "acces denied HKLM" }),
    ).toContain("registre");
  });

  it("handles ProcessFailed", () => {
    const msg = describeError({
      code: "ProcessFailed",
      details: { command: "pwsh", detail: "exit 1" },
    });
    expect(msg).toContain("pwsh");
    expect(msg).toContain("exit 1");
  });

  it("handles Generic", () => {
    expect(describeError({ code: "Generic", details: "anything" })).toBe("anything");
  });

  it("handles Error instance", () => {
    expect(describeError(new Error("native"))).toContain("native");
  });

  it("handles PermissionDenied", () => {
    const msg = describeError({ code: "PermissionDenied", details: "fichier X" });
    expect(msg).toContain("Accès refusé");
    expect(msg).toContain("fichier X");
  });

  it("handles InvalidInput", () => {
    const msg = describeError({ code: "InvalidInput", details: "nom invalide" });
    expect(msg).toContain("Entrée invalide");
    expect(msg).toContain("nom invalide");
  });
});
