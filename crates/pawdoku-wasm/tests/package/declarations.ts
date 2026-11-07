// The package's declarations held to the engine (S12, question 1, the third link).
// `tsc --noEmit --strict` checks this module against pkg/pawdoku_wasm.d.ts, and
// `play.mjs` imports its tables and puts each value to the engine.
//
// A snapshot held to an interface with `satisfies` catches an interface narrower than
// the value, never a wider one. This module catches the wider one: each exported union
// has a table with one value per member, so a new member is a missing key; each plain
// value's interface is held equal to its shape written out here, optional fields and
// field types included; and what a game must not be able to pass is an
// `@ts-expect-error`, so a declaration that starts to allow it is an unused directive.
// Run under Node's type stripping, so every type import is `import type`.

import type {
  BoardCell,
  BoardRecord,
  BoardView,
  CellReading,
  Check,
  EngineErrorName,
  Given,
  GenerationRefused,
  GenerationResult,
  Move,
  MoveKind,
  Operation,
  Played,
  Position,
  ReadingAfter,
  RecordCheck,
  RecordMove,
  SearchResult,
  Status,
  Tier,
  Verdict,
} from "../../pkg/pawdoku_wasm.js";

/** True when `A` and `B` are the same type, optionality and readonly included. */
type Same<A, B> = (<X>() => X extends A ? 1 : 2) extends <X>() => X extends B ? 1 : 2 ? true : false;

/** A cell the rules' fixture leaves to the player, where every operation is asked. */
export const here = { row: 1, column: 3 } satisfies Position;

// One value per member of each union. A member added to a declaration is a missing
// key here; one removed is an excess key.

/** The seven operations, as the core's snapshot of them lists them, in its order. */
export const operations = {
  Place: { kind: "Place", position: here, digit: 4 },
  Erase: { kind: "Erase", position: here },
  WriteMark: { kind: "WriteMark", position: here, digit: 2 },
  StrikeMark: { kind: "StrikeMark", position: here, digit: 2 },
  CheckCell: { kind: "CheckCell", position: here },
  Undo: { kind: "Undo" },
  Redo: { kind: "Redo" },
} satisfies { [K in Operation["kind"]]: Extract<Operation, { kind: K }> };

/** The five tiers. */
export const tiers = {
  One: "One",
  Two: "Two",
  Three: "Three",
  Four: "Four",
  Five: "Five",
} satisfies { [K in Tier]: K };

export const statuses = { Unsolved: "Unsolved", Solved: "Solved" } satisfies { [K in Status]: K };

export const verdicts = {
  NoSolution: "NoSolution",
  OneSolution: "OneSolution",
  ManySolutions: "ManySolutions",
} satisfies { [K in Verdict]: K };

export const moveKinds = {
  Place: "Place",
  Erase: "Erase",
  WriteMark: "WriteMark",
  StrikeMark: "StrikeMark",
} satisfies { [K in MoveKind]: K };

export const recordMoves = {
  Place: { Place: { target: here, digit: 4 } },
  Erase: { Erase: { target: here } },
  WriteMark: { WriteMark: { target: here, digit: 2 } },
  StrikeMark: { StrikeMark: { target: here, digit: 2 } },
} satisfies { [K in MoveKind]: Extract<RecordMove, Record<K, unknown>> };

export const engineErrorNames = {
  PlayError: "PlayError",
  ReopenError: "ReopenError",
  SolveError: "SolveError",
  GenerateError: "GenerateError",
  RandomError: "RandomError",
  FromRecordError: "FromRecordError",
} satisfies { [K in EngineErrorName]: K };

// Each plain value's interface, written out: a field added, removed, made optional or
// widened in a declaration makes its line `false`, which `true` does not satisfy.

type Shapes = [
  Same<Position, { row: number; column: number }>,
  Same<Given, { position: Position; digit: number }>,
  Same<
    BoardCell,
    {
      row: number;
      column: number;
      digit: number | null;
      is_given: boolean;
      is_conflicting: boolean;
      note?: number[];
    }
  >,
  Same<
    Move,
    { index: number; kind: MoveKind; target: Position; digit: number | null; is_undone: boolean }
  >,
  Same<
    Check,
    { index: number; after_move: number; target: Position; digit: number; is_right: boolean }
  >,
  Same<
    BoardView,
    {
      status: Status;
      is_full: boolean;
      is_consistent: boolean;
      can_undo: boolean;
      can_redo: boolean;
      cells: BoardCell[];
      moves: Move[];
      checks: Check[];
    }
  >,
  Same<CellReading, { row: number; column: number; digit: number | null; note: number[] }>,
  Same<ReadingAfter, { move_index: number; cells: CellReading[] }>,
  Same<RecordCheck, { after_move: number; target: Position; digit: number }>,
  Same<
    RecordMove,
    | { Place: { target: Position; digit: number } }
    | { Erase: { target: Position } }
    | { WriteMark: { target: Position; digit: number } }
    | { StrikeMark: { target: Position; digit: number } }
  >,
  Same<
    BoardRecord,
    {
      version: string;
      givens: Given[];
      moves: RecordMove[];
      undone: number;
      checks: RecordCheck[];
    }
  >,
  Same<
    Operation,
    | { kind: "Place"; position: Position; digit: number }
    | { kind: "Erase"; position: Position }
    | { kind: "WriteMark"; position: Position; digit: number }
    | { kind: "StrikeMark"; position: Position; digit: number }
    | { kind: "CheckCell"; position: Position }
    | { kind: "Undo" }
    | { kind: "Redo" }
  >,
  Same<Played, { record: BoardRecord; view: BoardView; check?: Check }>,
  Same<SearchResult, { verdict: Verdict; guesses: number; solutions: number[][][] }>,
  Same<
    GenerationResult,
    {
      tier: Tier;
      seed: bigint | null;
      start_index: bigint;
      draws_taken: bigint;
      generation_version: string;
      random_version: string;
      givens: Given[];
    }
  >,
  Same<Pick<GenerationRefused, "name" | "tier" | "draws_taken">, { name: "GenerateError"; tier: Tier; draws_taken: bigint }>,
];

export const shapes: Shapes = [
  true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
];

// What a game must not be able to pass. An unused directive is itself an error.

// @ts-expect-error a Place needs its digit
export const noDigit: Operation = { kind: "Place", position: here };
// @ts-expect-error no such operation
export const fly: Operation = { kind: "Fly" };
// @ts-expect-error no such tier
export const six: Tier = "Six";
// @ts-expect-error a position needs its column
export const noColumn: Position = { row: 1 };
// @ts-expect-error a given needs its digit
export const noGivenDigit: Given = { position: here };
// @ts-expect-error a record needs its version
export const unversioned: BoardRecord = { givens: [], moves: [], undone: 0, checks: [] };
// @ts-expect-error a record's move is one of the four kinds
export const flown: RecordMove = { Fly: { target: here } };
