import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { reportFrontendError, resetFrontendLogDedupe } from "../frontend-log";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
  resetFrontendLogDedupe();
  mockedInvoke.mockResolvedValue(undefined);
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("reportFrontendError", () => {
  it("sends the level and a source-prefixed message to log_frontend_error", () => {
    reportFrontendError("PageErrorBoundary:Santé", new Error("kaboom"));
    expect(mockedInvoke).toHaveBeenCalledWith("log_frontend_error", {
      level: "error",
      message: "PageErrorBoundary:Santé : kaboom",
    });
  });

  it("describes command errors instead of printing [object Object]", () => {
    reportFrontendError("commande get_disks", { code: "NotFound", details: "disque" });
    expect(mockedInvoke).toHaveBeenCalledWith("log_frontend_error", {
      level: "error",
      message: "commande get_disks : Introuvable : disque",
    });
  });

  it("supports the warn level", () => {
    reportFrontendError("x", "souci", "warn");
    expect(mockedInvoke).toHaveBeenCalledWith("log_frontend_error", { level: "warn", message: "x : souci" });
  });

  it("does not log the same message twice in a row", () => {
    reportFrontendError("x", "meme erreur");
    reportFrontendError("x", "meme erreur");
    expect(mockedInvoke).toHaveBeenCalledTimes(1);
    reportFrontendError("x", "autre erreur");
    expect(mockedInvoke).toHaveBeenCalledTimes(2);
  });

  it("falls back to the console when the logging command rejects", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    mockedInvoke.mockRejectedValue("journal indisponible");
    reportFrontendError("x", "boom");
    await vi.waitFor(() => expect(warn).toHaveBeenCalled());
  });

  it("never throws when invoke throws synchronously or returns nothing", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    mockedInvoke.mockImplementation(() => {
      throw new Error("pas de runtime Tauri");
    });
    expect(() => reportFrontendError("x", "a")).not.toThrow();
    expect(warn).toHaveBeenCalled();
    mockedInvoke.mockReturnValue(undefined as never);
    expect(() => reportFrontendError("x", "b")).not.toThrow();
  });
});
