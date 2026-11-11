import { describe, it, expect, beforeEach, vi } from "vitest";
import { recordSample, _bufferSizeForTests, _flushForTests, _resetBufferForTests } from "../perf-collector";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

describe("perf-collector", () => {
  beforeEach(() => {
    _resetBufferForTests();
    vi.clearAllMocks();
  });

  it("recordSample appends to buffer", () => {
    recordSample({ ts: 1, cmd: "x", argsKey: "", durationMs: 10, cacheHit: false });
    expect(_bufferSizeForTests()).toBe(1);
  });

  it("flush on empty buffer is noop", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    await _flushForTests();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("flush sends batch then clears buffer", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    recordSample({ ts: 1, cmd: "a", argsKey: "", durationMs: 1, cacheHit: false });
    recordSample({ ts: 2, cmd: "b", argsKey: "", durationMs: 2, cacheHit: true });
    await _flushForTests();
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("perf_dump_frontend", expect.objectContaining({
      samples: expect.arrayContaining([
        expect.objectContaining({ cmd: "a" }),
        expect.objectContaining({ cmd: "b" }),
      ]),
    }));
    expect(_bufferSizeForTests()).toBe(0);
  });

  it("MAX_BUFFER trigger auto-flush", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    for (let i = 0; i < 500; i++) {
      recordSample({ ts: i, cmd: "x", argsKey: "", durationMs: 1, cacheHit: false });
    }
    // Attendre micro-tick pour que le flush async se resolve
    await new Promise((r) => setTimeout(r, 0));
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("flush in-flight guard prevents concurrent invokes", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    // Make invoke take a moment so we can fire a second flush concurrently
    let resolveInvoke!: () => void;
    (invoke as ReturnType<typeof vi.fn>).mockImplementationOnce(
      () => new Promise<void>((resolve) => {
        resolveInvoke = resolve;
      }),
    );

    recordSample({ ts: 1, cmd: "a", argsKey: "", durationMs: 1, cacheHit: false });
    const first = _flushForTests();
    // Second flush should be a noop while first is in-flight
    recordSample({ ts: 2, cmd: "b", argsKey: "", durationMs: 2, cacheHit: false });
    const second = _flushForTests();
    expect(invoke).toHaveBeenCalledTimes(1);
    resolveInvoke();
    await first;
    await second;
  });
});
