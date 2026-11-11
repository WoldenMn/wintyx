import { describe, it, expect } from "vitest";
import { agree, previewSentence, type BatchNoun } from "../batch-text";
import { summarizeBatch } from "../helpers";

const optimisation: BatchNoun = { singular: "optimisation", plural: "optimisations", feminine: true };
const fantome: BatchNoun = { singular: "fantôme", plural: "fantômes", participle: "supprimé" };

describe("agree", () => {
  it("accorde le participe en genre et en nombre, 0 et 1 restent au singulier", () => {
    expect(agree("appliqué", 0, true)).toBe("appliquée");
    expect(agree("appliqué", 1, false)).toBe("appliqué");
    expect(agree("appliqué", 1, true)).toBe("appliquée");
    expect(agree("appliqué", 2, false)).toBe("appliqués");
    expect(agree("appliqué", 8, true)).toBe("appliquées");
  });
});

describe("previewSentence", () => {
  it("0 élément", () => {
    expect(previewSentence(0, optimisation)).toBe("Aucune optimisation à modifier actuellement");
    expect(previewSentence(0, fantome)).toBe("Aucun fantôme à modifier actuellement");
  });

  it("1 élément", () => {
    expect(previewSentence(1, optimisation)).toBe("Aperçu : 1 optimisation serait modifiée");
    expect(previewSentence(1, fantome)).toBe("Aperçu : 1 fantôme serait modifié");
  });

  it("n éléments", () => {
    expect(previewSentence(8, optimisation)).toBe("Aperçu : 8 optimisations seraient modifiées");
    expect(previewSentence(40, fantome)).toBe("Aperçu : 40 fantômes seraient modifiés");
  });
});

describe("summarizeBatch phrases", () => {
  const ok = (total: number, success: number, failed: number, noun: BatchNoun) =>
    summarizeBatch({ total, success, failed, errors: [] }, noun).message;

  it("réussite accordée", () => {
    expect(ok(1, 1, 0, optimisation)).toBe("1 optimisation appliquée");
    expect(ok(8, 8, 0, optimisation)).toBe("8 optimisations appliquées");
    expect(ok(3, 3, 0, fantome)).toBe("3 fantômes supprimés");
  });

  it("échec total et partiel", () => {
    expect(ok(2, 0, 2, optimisation)).toBe("2 optimisations en échec");
    expect(ok(5, 3, 2, fantome)).toBe("3/5 fantômes supprimés, 2 en échec");
  });

  it("aperçu et lot vide", () => {
    expect(summarizeBatch({ total: 4, success: 0, failed: 0, errors: [], dryRun: true }, fantome).message).toBe(
      "Aperçu : 4 fantômes seraient modifiés",
    );
    expect(ok(0, 0, 0, optimisation)).toBe("Aucune optimisation à traiter");
  });
});
