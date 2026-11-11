export interface BatchNoun {
  singular: string;
  plural: string;
  feminine?: boolean;
  participle?: string;
}

export const DEFAULT_PARTICIPLE = "appliqué";

export function agree(participle: string, count: number, feminine = false): string {
  return `${participle}${feminine ? "e" : ""}${count > 1 ? "s" : ""}`;
}

export function labelFor(count: number, noun: BatchNoun): string {
  return count > 1 ? noun.plural : noun.singular;
}

export function none(noun: BatchNoun): string {
  return `${noun.feminine ? "Aucune" : "Aucun"} ${noun.singular}`;
}

export function previewSentence(total: number, noun: BatchNoun): string {
  if (total === 0) return `${none(noun)} à modifier actuellement`;
  const verb = total > 1 ? "seraient" : "serait";
  return `Aperçu : ${total} ${labelFor(total, noun)} ${verb} ${agree("modifié", total, noun.feminine)}`;
}
