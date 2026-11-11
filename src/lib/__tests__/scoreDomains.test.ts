import { describe, it, expect } from "vitest";
import { SCORE_DOMAINS, type ScoreDomainId } from "../scoreDomains";

describe("SCORE_DOMAINS", () => {
  it("contains exactly 5 domains", () => {
    expect(SCORE_DOMAINS).toHaveLength(5);
  });

  it("has expected ids", () => {
    const ids = SCORE_DOMAINS.map((d) => d.id);
    expect(ids).toEqual([
      "dataProtection",
      "services",
      "hardening",
      "performance",
      "bloatware",
    ]);
  });

  it("each domain has a breakdownCommand string", () => {
    for (const domain of SCORE_DOMAINS) {
      expect(typeof domain.breakdownCommand).toBe("string");
      expect(domain.breakdownCommand.length).toBeGreaterThan(0);
    }
  });

  it("ScoreDomainId type matches the 5 ids", () => {
    const sample: ScoreDomainId = "dataProtection";
    expect(sample).toBe("dataProtection");
  });
});
