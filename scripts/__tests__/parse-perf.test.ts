import { describe, it, expect } from "vitest";
import { aggregate, parseCsv, parseJsonl, renderMarkdown } from "../parse-perf";

describe("parse-perf", () => {
  it("aggregate computes p50/p95/max correctly", () => {
    const samples = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
    const a = aggregate(samples);
    expect(a.count).toBe(10);
    expect(a.min).toBe(10);
    expect(a.max).toBe(100);
    expect(a.p50).toBeGreaterThanOrEqual(50);
    expect(a.p50).toBeLessThanOrEqual(60);
    expect(a.p95).toBeGreaterThanOrEqual(90);
    expect(a.mean).toBeCloseTo(55, 0);
    expect(a.total).toBe(550);
  });

  it("aggregate on N=1 collapses p50/p95/p99/max to the single value", () => {
    const a = aggregate([42]);
    expect(a.count).toBe(1);
    expect(a.min).toBe(42);
    expect(a.p50).toBe(42);
    expect(a.p95).toBe(42);
    expect(a.p99).toBe(42);
    expect(a.max).toBe(42);
    expect(a.mean).toBe(42);
    expect(a.total).toBe(42);
  });

  it("aggregate on empty input returns zeroed stats", () => {
    const a = aggregate([]);
    expect(a.count).toBe(0);
    expect(a.p50).toBe(0);
    expect(a.max).toBe(0);
    expect(a.total).toBe(0);
  });

  it("parseCsv handles backend CSV lines", () => {
    const csv = "1700000000000,get_system_info,1234\n1700000000001,get_disks,5678\n";
    const samples = parseCsv(csv);
    expect(samples).toHaveLength(2);
    expect(samples[0].cmd).toBe("get_system_info");
    expect(samples[0].durationUs).toBe(1234);
    expect(samples[1].cmd).toBe("get_disks");
  });

  it("parseJsonl handles frontend JSONL lines", () => {
    const jsonl =
      '{"ts":1,"cmd":"get_x","argsKey":"","durationMs":42.5,"cacheHit":false}\n' +
      '{"ts":2,"cmd":"get_y","argsKey":"","durationMs":10,"cacheHit":true}\n';
    const samples = parseJsonl(jsonl);
    expect(samples).toHaveLength(2);
    expect(samples[0].durationMs).toBe(42.5);
    expect(samples[1].cacheHit).toBe(true);
  });

  it("renderMarkdown produces valid markdown with sections", () => {
    const md = renderMarkdown({
      backendSamples: [{ ts: 1, cmd: "x", durationUs: 1000 }],
      frontendSamples: [{ ts: 1, cmd: "x", argsKey: "", durationMs: 1.5, cacheHit: false }],
      date: "2026-05-27",
    });
    expect(md).toContain("# Wintyx");
    expect(md).toContain("## 1. Top 10 hotspots backend");
    expect(md).toContain("## 2. Top 10 hotspots frontend");
    expect(md).toContain("## 3. Delta frontend - backend");
  });

  it("parseJsonl skips malformed lines silently", () => {
    const jsonl =
      '{"ts":1,"cmd":"good","argsKey":"","durationMs":10,"cacheHit":false}\n' +
      'not-valid-json\n' +
      '{"ts":2,"cmd":"also-good","argsKey":"","durationMs":20,"cacheHit":false}\n' +
      '{"ts":3,"argsKey":"","durationMs":30,"cacheHit":false}\n';
    const samples = parseJsonl(jsonl);
    expect(samples).toHaveLength(2);
    expect(samples[0].cmd).toBe("good");
    expect(samples[1].cmd).toBe("also-good");
  });

  it("parseCsv skips lines with wrong field count or non-finite numbers", () => {
    const csv =
      "1700000000000,get_x,1234\n" +
      "malformed\n" +
      "1700000000001,get_y,5678,extra\n" +
      "not-a-number,get_z,9999\n" +
      "1700000000002,get_w,not-a-number\n" +
      "1700000000003,,4242\n" +
      "1700000000004,get_v,7777\n";
    const samples = parseCsv(csv);
    expect(samples).toHaveLength(2);
    expect(samples[0].cmd).toBe("get_x");
    expect(samples[1].cmd).toBe("get_v");
  });
});
