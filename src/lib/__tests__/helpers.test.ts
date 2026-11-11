import { describe, it, expect } from "vitest";
import {
  getScoreColor,
  getScoreBg,
  getUsageColor,
  getUsageBarColor,
  getRiskScoreColor,
  getRiskBadgeVariant,
  getSizeBadgeVariant,
  formatBytes,
  formatUptime,
  groupBy,
  summarizeBatch,
  diffPercent,
  isImprovement,
  computeScoreDelta,
} from "../helpers";
import type { ScoreSnapshot } from "../bindings";

function snap(date: string, global: number): ScoreSnapshot {
  return { date, timestamp: `${date} 12:00:00`, globalScore: global,
    scores: { dataProtection: global, services: global, hardening: global, performance: global, bloatware: global } };
}

describe("getScoreColor", () => {
  it("returns text-danger for score 0", () => {
    expect(getScoreColor(0)).toBe("text-danger");
  });

  it("returns text-danger for score 49", () => {
    expect(getScoreColor(49)).toBe("text-danger");
  });

  it("returns text-warning for score 50", () => {
    expect(getScoreColor(50)).toBe("text-warning");
  });

  it("returns text-warning for score 79", () => {
    expect(getScoreColor(79)).toBe("text-warning");
  });

  it("returns text-success for score 80", () => {
    expect(getScoreColor(80)).toBe("text-success");
  });

  it("returns text-success for score 100", () => {
    expect(getScoreColor(100)).toBe("text-success");
  });
});

describe("getRiskScoreColor", () => {
  it("returns text-success below 40 (inverted thresholds)", () => {
    expect(getRiskScoreColor(0)).toBe("text-success");
    expect(getRiskScoreColor(39)).toBe("text-success");
  });
  it("returns text-warning for 40..69", () => {
    expect(getRiskScoreColor(40)).toBe("text-warning");
    expect(getRiskScoreColor(69)).toBe("text-warning");
  });
  it("returns text-danger for 70+", () => {
    expect(getRiskScoreColor(70)).toBe("text-danger");
    expect(getRiskScoreColor(100)).toBe("text-danger");
  });
});

describe("getRiskBadgeVariant", () => {
  it("maps critical and high to danger", () => {
    expect(getRiskBadgeVariant("critical")).toBe("danger");
    expect(getRiskBadgeVariant("high")).toBe("danger");
  });
  it("maps medium to warning and low to info", () => {
    expect(getRiskBadgeVariant("medium")).toBe("warning");
    expect(getRiskBadgeVariant("low")).toBe("info");
  });
  it("is case-insensitive and falls back to success", () => {
    expect(getRiskBadgeVariant("CRITICAL")).toBe("danger");
    expect(getRiskBadgeVariant("Essential")).toBe("danger");
    expect(getRiskBadgeVariant("Recommended")).toBe("warning");
    expect(getRiskBadgeVariant("unknown")).toBe("success");
  });
});

describe("getSizeBadgeVariant", () => {
  it("returns neutral up to 10 MB", () => {
    expect(getSizeBadgeVariant(0)).toBe("neutral");
    expect(getSizeBadgeVariant(10 * 1048576)).toBe("neutral");
  });
  it("returns warning above 10 MB up to 100 MB", () => {
    expect(getSizeBadgeVariant(11 * 1048576)).toBe("warning");
    expect(getSizeBadgeVariant(100 * 1048576)).toBe("warning");
  });
  it("returns danger above 100 MB", () => {
    expect(getSizeBadgeVariant(101 * 1048576)).toBe("danger");
  });
});

describe("getScoreBg", () => {
  it("returns bg-danger/10 for score 0", () => {
    expect(getScoreBg(0)).toBe("bg-danger/10");
  });

  it("returns bg-danger/10 for score 49", () => {
    expect(getScoreBg(49)).toBe("bg-danger/10");
  });

  it("returns bg-warning/10 for score 50", () => {
    expect(getScoreBg(50)).toBe("bg-warning/10");
  });

  it("returns bg-warning/10 for score 79", () => {
    expect(getScoreBg(79)).toBe("bg-warning/10");
  });

  it("returns bg-success/10 for score 80", () => {
    expect(getScoreBg(80)).toBe("bg-success/10");
  });

  it("returns bg-success/10 for score 100", () => {
    expect(getScoreBg(100)).toBe("bg-success/10");
  });
});

describe("getUsageColor", () => {
  it("returns text-success for value 0", () => {
    expect(getUsageColor(0)).toBe("text-success");
  });

  it("returns text-success for value 69", () => {
    expect(getUsageColor(69)).toBe("text-success");
  });

  it("returns text-warning for value 70", () => {
    expect(getUsageColor(70)).toBe("text-warning");
  });

  it("returns text-warning for value 89", () => {
    expect(getUsageColor(89)).toBe("text-warning");
  });

  it("returns text-danger for value 90", () => {
    expect(getUsageColor(90)).toBe("text-danger");
  });

  it("returns text-danger for value 100", () => {
    expect(getUsageColor(100)).toBe("text-danger");
  });
});

describe("getUsageBarColor", () => {
  it("returns bg-success for value 0", () => {
    expect(getUsageBarColor(0)).toBe("bg-success");
  });

  it("returns bg-success for value 69", () => {
    expect(getUsageBarColor(69)).toBe("bg-success");
  });

  it("returns bg-warning for value 70", () => {
    expect(getUsageBarColor(70)).toBe("bg-warning");
  });

  it("returns bg-warning for value 89", () => {
    expect(getUsageBarColor(89)).toBe("bg-warning");
  });

  it("returns bg-danger for value 90", () => {
    expect(getUsageBarColor(90)).toBe("bg-danger");
  });

  it("returns bg-danger for value 100", () => {
    expect(getUsageBarColor(100)).toBe("bg-danger");
  });
});

describe("formatBytes", () => {
  it("formats 0 bytes", () => {
    expect(formatBytes(0)).toBe("0 B");
  });

  it("formats bytes under 1 KB", () => {
    expect(formatBytes(500)).toBe("500 B");
  });

  it("formats exactly 1 KB", () => {
    expect(formatBytes(1024)).toBe("1.0 Ko");
  });

  it("formats exactly 1 MB", () => {
    expect(formatBytes(1048576)).toBe("1.0 Mo");
  });

  it("formats exactly 1 GB", () => {
    expect(formatBytes(1073741824)).toBe("1.00 Go");
  });

  it("formats fractional KB", () => {
    expect(formatBytes(1536)).toBe("1.5 Ko");
  });

  it("formats fractional GB", () => {
    expect(formatBytes(2147483648)).toBe("2.00 Go");
  });
});

describe("formatUptime", () => {
  it("formats minutes only", () => {
    expect(formatUptime(60)).toBe("1m");
  });

  it("formats zero minutes", () => {
    expect(formatUptime(0)).toBe("0m");
  });

  it("formats hours and minutes", () => {
    expect(formatUptime(3600)).toBe("1h 0m");
  });

  it("formats hours with minutes", () => {
    expect(formatUptime(3660)).toBe("1h 1m");
  });

  it("formats days, hours, minutes", () => {
    expect(formatUptime(86400)).toBe("1j 0h 0m");
  });

  it("formats complex duration", () => {
    expect(formatUptime(90061)).toBe("1j 1h 1m");
  });

  it("formats multiple days", () => {
    expect(formatUptime(172800 + 7200 + 300)).toBe("2j 2h 5m");
  });
});

describe("groupBy", () => {
  it("returns empty object for empty array", () => {
    expect(groupBy([], "key" as never)).toEqual({});
  });

  it("groups items into a single group", () => {
    const items = [
      { category: "a", value: 1 },
      { category: "a", value: 2 },
    ];
    const result = groupBy(items, "category");
    expect(result).toEqual({
      a: [
        { category: "a", value: 1 },
        { category: "a", value: 2 },
      ],
    });
  });

  it("groups items into multiple groups", () => {
    const items = [
      { category: "a", value: 1 },
      { category: "b", value: 2 },
      { category: "a", value: 3 },
      { category: "c", value: 4 },
    ];
    const result = groupBy(items, "category");
    expect(Object.keys(result)).toHaveLength(3);
    expect(result["a"]).toHaveLength(2);
    expect(result["b"]).toHaveLength(1);
    expect(result["c"]).toHaveLength(1);
  });

  it("preserves item references", () => {
    const item = { type: "x", data: "hello" };
    const result = groupBy([item], "type");
    expect(result["x"][0]).toBe(item);
  });
});

describe("summarizeBatch", () => {
  const noun = { singular: "item", plural: "items" };

  it("returns success for empty batch", () => {
    const { level, message } = summarizeBatch(
      { total: 0, success: 0, failed: 0, errors: [] },
      noun,
    );
    expect(level).toBe("success");
    expect(message).toContain("Aucun");
  });

  it("returns success when all succeed", () => {
    const { level, message } = summarizeBatch(
      { total: 3, success: 3, failed: 0, errors: [] },
      noun,
    );
    expect(level).toBe("success");
    expect(message).toContain("3");
    expect(message).toContain("3 items appliqués");
  });

  it("returns error when all fail", () => {
    const { level, message } = summarizeBatch(
      { total: 2, success: 0, failed: 2, errors: ["a", "b"] },
      noun,
    );
    expect(level).toBe("error");
    expect(message).toContain("2");
    expect(message).toContain("échec");
  });

  it("returns warning for partial success", () => {
    const { level, message } = summarizeBatch(
      { total: 5, success: 3, failed: 2, errors: ["x", "y"] },
      noun,
    );
    expect(level).toBe("warning");
    expect(message).toContain("3/5");
    expect(message).toContain("2 en échec");
  });

  it("uses singular form for total of 1", () => {
    const { message } = summarizeBatch(
      { total: 1, success: 1, failed: 0, errors: [] },
      noun,
    );
    expect(message).toBe("1 item appliqué");
  });
});

describe("diffPercent", () => {
  it("returns 0 when before is 0", () => {
    expect(diffPercent(0, 100)).toBe(0);
  });

  it("returns positive percentage for increase", () => {
    expect(diffPercent(100, 150)).toBe(50);
  });

  it("returns negative percentage for decrease", () => {
    expect(diffPercent(200, 100)).toBe(-50);
  });

  it("returns 0 for identical values", () => {
    expect(diffPercent(42, 42)).toBe(0);
  });

  it("handles fractional values", () => {
    expect(diffPercent(100, 125)).toBe(25);
  });
});

describe("isImprovement", () => {
  it("higherIsBetter: increase counts as improvement", () => {
    expect(isImprovement(100, 150, true)).toBe(true);
  });

  it("higherIsBetter: decrease counts as regression", () => {
    expect(isImprovement(100, 50, true)).toBe(false);
  });

  it("lowerIsBetter: decrease counts as improvement", () => {
    expect(isImprovement(200, 100, false)).toBe(true);
  });

  it("lowerIsBetter: increase counts as regression", () => {
    expect(isImprovement(50, 100, false)).toBe(false);
  });

  it("identical values are not an improvement", () => {
    expect(isImprovement(50, 50, true)).toBe(false);
    expect(isImprovement(50, 50, false)).toBe(false);
  });
});

describe("computeScoreDelta", () => {
  it("retourne null si moins de 2 points", () => {
    expect(computeScoreDelta([], 7)).toBeNull();
    expect(computeScoreDelta([snap("2026-06-24", 50)], 7)).toBeNull();
  });

  it("calcule le delta vs le point a N jours", () => {
    const h = [snap("2026-06-01", 40), snap("2026-06-17", 55), snap("2026-06-24", 60)];
    expect(computeScoreDelta(h, 7)).toBe(5);
    expect(computeScoreDelta(h, 30)).toBe(20);
  });

  it("choisit le point le plus recent a-ou-avant la cible", () => {
    const h = [snap("2026-06-10", 30), snap("2026-06-15", 45), snap("2026-06-24", 60)];
    expect(computeScoreDelta(h, 7)).toBe(15);
  });
});
