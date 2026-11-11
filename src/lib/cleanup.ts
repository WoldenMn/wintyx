import type { CleanupTarget } from "./types";

export function isCleanable(target: CleanupTarget): boolean {
  return target.accessible && target.sizeBytes > 0;
}

export function describeCleanAll(items: CleanupTarget[], formatSize: (bytes: number) => string): string {
  const cleanable = items.filter(isCleanable);
  const blocked = items.filter((t) => !t.accessible);
  const total = cleanable.reduce((sum, t) => sum + t.sizeBytes, 0);
  const verb = cleanable.length > 1 ? "vont être nettoyées" : "va être nettoyée";
  const lines = cleanable.map((t) => `- ${t.name} (${t.sizeDisplay})`).join("\n");
  let text = `${cleanable.length} ${cleanable.length > 1 ? "cibles" : "cible"} ${verb} (${formatSize(total)}) :\n${lines}`;
  if (blocked.length > 0) {
    const heading = blocked.length > 1 ? "Non nettoyées" : "Non nettoyée";
    text += `\n\n${heading} (droits administrateur requis) :\n${blocked.map((t) => `- ${t.name}`).join("\n")}`;
  }
  return text;
}
