// Writes target/wasm-package-test/snapshots.ts: each JSON snapshot the core accepted
// (crates/pawdoku/tests/snapshots/*_as_json.snap), stripped of its insta header, as
// `export const <name> = <JSON> satisfies <Interface>;`, for `tsc` to hold the
// package's interfaces to the core's wire shape (S12, question 1, the first link).
// `satisfies` on a literal checks excess properties too, so a field the interface has
// lost, or the snapshot has gained, fails. A snapshot not named below fails here, so a
// new wire value is given its interface before the gate passes.

import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = new URL("../../../../", import.meta.url);
const snapshots = new URL("crates/pawdoku/tests/snapshots/", root);
const out = new URL("target/wasm-package-test/", root);

/** Each snapshot's interface. A `bigint` field is a number in JSON, hence `Wire<T>`. */
export const INTERFACES = {
  a_generation_result_from_a_stream: "Wire<GenerationResult>",
  a_search_with_no_solution: "SearchResult",
  a_search_with_one_solution: "SearchResult",
  a_search_with_two_solutions: "SearchResult",
  the_generation_result_of_seed_zero_in_tier_3: "Wire<GenerationResult>",
  the_reading_after_a_move: "ReadingAfter",
  the_record_of_the_scripted_game: "BoardRecord",
  the_seven_operations: "Operation[]",
  the_view_of_the_scripted_game: "BoardView",
  what_a_check_hands_back: "Played",
};

/** The snapshots, by the name between `snapshot_` and `_as_json`, each its JSON text. */
export function readSnapshots() {
  const found = {};
  for (const file of readdirSync(snapshots).sort()) {
    const name = /^snapshots__tests__snapshot_(.+)_as_json\.snap$/.exec(file)?.[1];
    if (name === undefined) continue;
    const text = readFileSync(new URL(file, snapshots), "utf8");
    found[name] = text.split("---\n").slice(2).join("---\n");
  }
  return found;
}

const isMain = process.argv[1] === fileURLToPath(import.meta.url);
if (isMain) {
  const found = readSnapshots();
  const unnamed = Object.keys(found).filter((name) => !(name in INTERFACES));
  const missing = Object.keys(INTERFACES).filter((name) => !(name in found));
  if (unnamed.length > 0 || missing.length > 0) {
    console.error(`snapshots: no interface for ${unnamed.join(", ") || "none"}; no snapshot for ${missing.join(", ") || "none"}`);
    process.exit(1);
  }
  const types = new Set(Object.values(INTERFACES).map((type) => type.replace(/^Wire<|>$|\[\]$/g, "")));
  const lines = [
    "// Written by crates/pawdoku-wasm/tests/package/snapshots.mjs; not kept in Git.",
    `import type { ${[...types].sort().join(", ")} } from "../../crates/pawdoku-wasm/pkg/pawdoku_wasm.js";`,
    "",
    "/** A value as JSON holds it: a `bigint` field is a number. */",
    "type Wire<T> = { [K in keyof T]: T[K] extends bigint ? number : T[K] extends bigint | null ? number | null : T[K] };",
    "",
  ];
  for (const [name, json] of Object.entries(found)) {
    lines.push(`export const ${name} = ${json.trim()} satisfies ${INTERFACES[name]};`, "");
  }
  mkdirSync(out, { recursive: true });
  writeFileSync(new URL("snapshots.ts", out), lines.join("\n"));
  console.log(`snapshots: ${Object.keys(found).length} snapshots written to target/wasm-package-test/snapshots.ts`);
}
