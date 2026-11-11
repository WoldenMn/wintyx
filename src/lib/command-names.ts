// DX-01 v2.16.0 : noms de commandes Tauri compile-checked.
//
// `useInvoke<T>(command)` contourne le wrapper `commands` genere par
// tauri-specta (piege documente : string non verifiee, une typo compile).
// On derive ici, au niveau type uniquement (aucun import runtime de
// bindings.ts), l'union des noms snake_case reellement enregistres cote
// Rust a partir des cles camelCase du wrapper genere.

type Commands = (typeof import("./bindings"))["commands"];

// "getSystemInfo" -> "get_system_info". Un caractere est une majuscule s'il
// change sous Uppercase/Lowercase ; les chiffres et underscores (invariants
// par casse) sont recopies tels quels.
type CamelToSnake<S extends string> = S extends `${infer Head}${infer Tail}`
  ? `${Head extends Uppercase<Head>
      ? Head extends Lowercase<Head>
        ? Head
        : `_${Lowercase<Head>}`
      : Head}${CamelToSnake<Tail>}`
  : S;

export type CommandName = CamelToSnake<keyof Commands & string>;
