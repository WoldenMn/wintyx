import { describe, it, expect } from "vitest";
import type { CommandName } from "../command-names";

// Spot-checks DX-01 au niveau type : ce fichier passe par `tsc -b` (include src),
// donc une derive CamelToSnake incorrecte casserait la compilation ici.
describe("CommandName", () => {
  it("derive les noms snake_case exacts depuis les cles camelCase generees", () => {
    const twoWords: CommandName = "get_disks";
    const multiWords: CommandName = "get_data_protection_items";
    const batch: CommandName = "get_all_scores";
    expect([twoWords, multiWords, batch]).toHaveLength(3);
  });

  it("rejette une typo a la compilation (piege useInvoke ferme)", () => {
    // @ts-expect-error nom de commande inexistant : ne doit pas compiler
    const typo: CommandName = "get_diskss";
    expect(typo).toBe("get_diskss");
  });
});
