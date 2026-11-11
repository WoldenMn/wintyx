import { describe, it, expect } from "vitest";
import { combineBatchResults } from "../helpers";
import type { BatchResult } from "../types";

function batch(partial: Partial<BatchResult>): BatchResult {
  return { total: 0, success: 0, failed: 0, errors: [], dryRun: false, previewItems: [], ...partial };
}

describe("combineBatchResults", () => {
  it("sums counters and concatenates errors and preview items in order", () => {
    const combined = combineBatchResults([
      batch({ total: 3, success: 2, failed: 1, errors: ["a"], previewItems: ["x"] }),
      batch({ total: 2, success: 2, failed: 0, errors: [], previewItems: ["y", "z"] }),
    ]);
    expect(combined).toEqual(
      batch({ total: 5, success: 4, failed: 1, errors: ["a"], previewItems: ["x", "y", "z"] }),
    );
  });

  it("stays a dry run only when every result is a dry run", () => {
    expect(combineBatchResults([batch({ dryRun: true }), batch({ dryRun: true })]).dryRun).toBe(true);
    expect(combineBatchResults([batch({ dryRun: true }), batch({ dryRun: false })]).dryRun).toBe(false);
  });

  it("returns an empty result for an empty list", () => {
    expect(combineBatchResults([])).toEqual(batch({}));
  });
});
