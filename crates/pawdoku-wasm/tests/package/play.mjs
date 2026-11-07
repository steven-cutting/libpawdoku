// The published package in Node, as a consumer loads it: `initSync` with the bytes of
// pkg/pawdoku_wasm_bg.wasm, then a game played held and from records, every JSON
// snapshot the core accepted held equal to the live value the package gives, and each
// value of declarations.ts's tables put to the engine. `just wasm-package-test` runs it
// after `just wasm-build`. Exits 1 on the first failure, naming it.

import { readFileSync } from "node:fs";
import { deepStrictEqual, ok, strictEqual } from "node:assert/strict";
import {
  Board,
  RandomStream,
  generate,
  generateFromSeed,
  initSync,
  open,
  play,
  read,
  readingAfter,
  recordVersion,
  search,
} from "../../pkg/pawdoku_wasm.js";
import { here, operations, tiers } from "./declarations.ts";
import { INTERFACES, readSnapshots } from "./snapshots.mjs";

initSync({ module: readFileSync(new URL("../../pkg/pawdoku_wasm_bg.wasm", import.meta.url)) });

/** The rules' fixture, and the same with one given fewer, which has two solutions. */
const FIXTURE = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
const TWO_SOLUTIONS = "53..7....6..195....98......8...6...34..8.3..17...2...6.6....28....419..5....8..79";

/** `picture`'s digits as givens. */
const givensOf = (picture) =>
  [...picture].flatMap((cell, at) =>
    /[1-9]/.test(cell) ? [{ position: { row: Math.floor(at / 9) + 1, column: (at % 9) + 1 }, digit: Number(cell) }] : [],
  );

/** JSON text, a `bigint` written as a number, as the core's snapshots write a `u64`. */
const json = (value) => JSON.stringify(value, (_key, field) => (typeof field === "bigint" ? Number(field) : field));

let passed = 0;
/** Runs `check`, counting it, or names it and exits. */
function step(name, check) {
  try {
    check();
    passed += 1;
  } catch (error) {
    console.error(`package test: ${name}: ${error.name}: ${error.message}`);
    process.exit(1);
  }
}

/** The `name` and `message` of what `call` throws, or `null` when it throws nothing. */
function thrown(call) {
  try {
    call();
    return null;
  } catch (error) {
    return [error.name, error.message];
  }
}

const givens = givensOf(FIXTURE);

step("a game held and from records is the same game", () => {
  const board = Board.open(givens);
  let { record } = open(givens);
  for (const operation of [operations.WriteMark, operations.Place]) {
    if (operation.kind === "WriteMark") board.writeMark(operation.position, operation.digit);
    else board.place(operation.position, operation.digit);
    const played = play(record, operation);
    strictEqual(JSON.stringify(played.view), JSON.stringify(board.view()));
    strictEqual(JSON.stringify(played.record), JSON.stringify(board.write()));
    record = played.record;
  }
  const kept = JSON.stringify(board.write());
  const reopened = Board.reopen(JSON.parse(kept));
  strictEqual(JSON.stringify(reopened.view()), JSON.stringify(board.view()));
  strictEqual(JSON.stringify(read(JSON.parse(kept))), JSON.stringify(board.view()));
  strictEqual(JSON.stringify(readingAfter(record, 2)), JSON.stringify(board.readingAfter(2)));
  strictEqual(recordVersion(), "record-1");
  board.free();
  reopened.free();
  deepStrictEqual(thrown(() => board.free()), ["Error", "null pointer passed to rust"]);
});

/** Each snapshot's live value, from the package. */
const snapshots = Object.fromEntries(Object.entries(readSnapshots()).map(([name, text]) => [name, JSON.parse(text)]));
const scripted = snapshots.the_record_of_the_scripted_game;
const live = {
  the_record_of_the_scripted_game: () => Board.reopen(scripted).write(),
  the_view_of_the_scripted_game: () => Board.reopen(scripted).view(),
  the_reading_after_a_move: () => readingAfter(scripted, 6),
  what_a_check_hands_back: () => play(scripted, operations.CheckCell),
  the_seven_operations: () => Object.values(operations),
  a_search_with_one_solution: () => search(givens),
  a_search_with_two_solutions: () => search(givensOf(TWO_SOLUTIONS)),
  a_search_with_no_solution: () => search([{ position: { row: 0, column: 10 }, digit: 3 }]),
  the_generation_result_of_seed_zero_in_tier_3: () => generateFromSeed("Three", 0n),
  a_generation_result_from_a_stream: () => {
    const stream = RandomStream.seeded(0n);
    generate("Three", stream);
    return generate("Three", stream);
  },
};

step("every snapshot has a live value", () => deepStrictEqual(Object.keys(live).sort(), Object.keys(INTERFACES).sort()));
for (const [name, value] of Object.entries(live)) {
  step(`the live value of ${name} is its snapshot`, () => strictEqual(json(value()), JSON.stringify(snapshots[name])));
}

step("each operation of the declarations' table reaches the engine", () => {
  for (const operation of Object.values(operations)) {
    const refused = thrown(() => play(scripted, operation));
    ok(refused === null || refused[0] !== "Error", `${operation.kind}: ${refused}`);
  }
});

step("each tier of the declarations' table reaches the engine", () => {
  for (const tier of Object.values(tiers)) strictEqual(generateFromSeed(tier, 1n).tier, tier);
});

step("what the declarations forbid, the engine refuses as a shape", () => {
  strictEqual(thrown(() => play(scripted, { kind: "Fly" }))?.[0], "Error");
  strictEqual(thrown(() => play(scripted, { kind: "Place", position: here }))?.[0], "Error");
  strictEqual(thrown(() => generateFromSeed("Six", 0n))?.[0], "Error");
});

step("a refusal is named for the engine's error and carries its text", () => {
  const board = Board.open(givens);
  deepStrictEqual(thrown(() => board.place({ row: 1, column: 1 }, 4)), ["PlayError", "the cell at row 1, column 1 holds a given"]);
  deepStrictEqual(thrown(() => board.place(here, 260)), ["Error", "invalid value: integer `260`, expected u8"]);
  board.free();
  const stream = RandomStream.replay([0, 0, 0, 0, 0]);
  try {
    generate("Three", stream);
    ok(false, "generation from five zeros was not refused");
  } catch (error) {
    deepStrictEqual([error.name, error.message, error.tier, error.draws_taken], ["GenerateError", "the replay stream is exhausted at draw 5", "Three", 5n]);
    strictEqual(JSON.stringify(error), '{"name":"GenerateError"}');
  }
  strictEqual(stream.index, 5n);
});

step("a value that throws as it is read leaves the board usable", () => {
  const board = Board.open(givens);
  board.place(here, 4);
  const kept = JSON.stringify(board.write());
  const getter = { get row() { throw new RangeError("row"); }, column: 3 };
  const trap = () => { throw new RangeError("trap"); };
  const proxy = new Proxy({}, { get: trap, has: trap, ownKeys: trap, getPrototypeOf: trap });
  deepStrictEqual(thrown(() => board.erase(getter)), ["RangeError", "row"]);
  deepStrictEqual(thrown(() => board.place(here, proxy)), ["RangeError", "trap"]);
  deepStrictEqual(thrown(() => board.readingAfter(proxy)), ["RangeError", "trap"]);
  strictEqual(JSON.stringify(board.write()), kept);
  board.erase(here);
  strictEqual(board.view().can_undo, true);
  board.free();
});

console.log(`package test: ${passed} checks passed`);
