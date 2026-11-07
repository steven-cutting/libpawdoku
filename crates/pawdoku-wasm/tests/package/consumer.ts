// A game's use of every export, type-checked with `tsc --noEmit --strict` against
// pkg/pawdoku_wasm.d.ts and never run: it holds that each export can be used as a game
// uses it with no cast, and that each value reads back into its narrow type.

import init, {
  Board,
  RandomStream,
  generate,
  generateFromSeed,
  generationVersion,
  initSync,
  open,
  play,
  randomVersion,
  read,
  readingAfter,
  recordVersion,
  search,
} from "../../pkg/pawdoku_wasm.js";
import type {
  BoardRecord,
  BoardView,
  Check,
  EngineErrorName,
  GenerationRefused,
  GenerationResult,
  Given,
  Operation,
  Played,
  Position,
  ReadingAfter,
  SearchResult,
  Tier,
} from "../../pkg/pawdoku_wasm.js";

/** What a game keeps between visits: the record, and what replays the puzzle. */
interface Kept {
  record: string;
  seed: string;
  tier: Tier;
  versions: { record: string; generation: string; random: string };
}

/** Loads the module in the browser, once. */
export async function load(): Promise<void> {
  await init();
}

/** Loads the module in a test, from the bytes of the `.wasm`. */
export function loadIn(bytes: BufferSource): void {
  initSync({ module: bytes });
}

/** A new puzzle from a seed, kept with what replays it. */
export function newPuzzle(seed: bigint, tier: Tier): Kept {
  const result: GenerationResult = generateFromSeed(tier, seed);
  const board = Board.open(result.givens);
  try {
    const record: BoardRecord = board.write();
    const kept: bigint | null = result.seed;
    const taken: bigint = result.draws_taken + result.start_index;
    void taken;
    return {
      record: JSON.stringify(record),
      seed: String(kept),
      tier: result.tier,
      versions: {
        record: recordVersion(),
        generation: result.generation_version === generationVersion() ? generationVersion() : "",
        random: result.random_version === randomVersion() ? randomVersion() : "",
      },
    };
  } finally {
    board.free();
  }
}

/** A held board through a game: every operation, the view, a reading and the record. */
export function held(givens: Given[], here: Position): BoardRecord {
  using board = Board.open(givens);
  board.writeMark(here, 2);
  board.strikeMark(here, 2);
  board.place(here, 4);
  const check: Check = board.check(here);
  const right: boolean = check.is_right;
  void right;
  board.erase(here);
  board.undo();
  board.redo();
  const view: BoardView = board.view();
  const digit: number | null = view.cells[0].digit;
  const note: number[] | undefined = view.cells[0].note;
  void digit;
  void note;
  const reading: ReadingAfter = board.readingAfter(view.moves.length);
  const after: number[] = reading.cells[0].note;
  void after;
  return board.write();
}

/** Returning to a kept game: one `try`, as a record that does not reopen throws. */
export function reopen(stored: string): Board | null {
  try {
    return Board.reopen(JSON.parse(stored) as BoardRecord);
  } catch (error) {
    // `ReopenError` for a record that does not reopen, `Error` for one of the wrong shape.
    const name = (error as Error).name as EngineErrorName | "Error";
    if (name !== "ReopenError" && name !== "Error") throw error;
    return null;
  }
}

/** Play from a record: values in and out, nothing held. */
export function fromRecords(givens: Given[], here: Position): BoardView {
  const opened: Played = open(givens);
  const place: Operation = { kind: "Place", position: here, digit: 4 };
  let played: Played = play(opened.record, place);
  played = play(played.record, { kind: "CheckCell", position: here });
  const check: Check | undefined = played.check;
  void check;
  const reading: ReadingAfter = readingAfter(played.record, 1);
  void reading;
  return read(played.record);
}

/** The setter's search. */
export function setter(givens: Given[]): number {
  const result: SearchResult = search(givens);
  return result.verdict === "OneSolution" ? result.solutions.length : result.guesses;
}

/** Generation from a stream the caller holds, and a refusal's properties. */
export function fromStream(): bigint {
  const stream = RandomStream.seeded(2024n);
  const draw: number = stream.nextDraw();
  void draw;
  try {
    const result = generate("Three", stream);
    return result.draws_taken;
  } catch (error) {
    const refused = error as GenerationRefused;
    const tier: Tier = refused.tier;
    void tier;
    return refused.draws_taken;
  } finally {
    const index: bigint = stream.index;
    void index;
    stream.free();
    void RandomStream.replay([0.25, 0.5]);
  }
}
