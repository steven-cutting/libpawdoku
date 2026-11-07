//! The engine's plain values as they cross: the TypeScript declaration of each, and the
//! two serialisers and the one reader every export goes through.
//!
//! The declarations are written here by hand, in one `typescript_custom_section`, each
//! field as the core's wire shape writes it (T41): fields in snake case, enum values in
//! Pascal case, an absent digit `null`, a `u64` a `bigint`. A TypeScript name is the
//! core's Rust name, except the record's, `BoardRecord`, because `Record` would shadow
//! TypeScript's own `Record<K, V>`. `tests/package/` holds them to the core's snapshots
//! and to the engine (S12, question 1).

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(typescript_custom_section)]
const DECLARATIONS: &str = r#"
/** A cell's place on the board, each coordinate counted from 1 at the top left. */
export interface Position {
    /** The row, from 1 to 9. */
    row: number;
    /** The column, from 1 to 9. */
    column: number;
}

/** A digit fixed in a cell before play begins. */
export interface Given {
    /** The cell it is fixed in. */
    position: Position;
    /** The digit, from 1 to 9. */
    digit: number;
}

/** Whether every cell holds a digit and none conflicts. */
export type Status = "Unsolved" | "Solved";

/** What a search concluded of the givens. */
export type Verdict = "NoSolution" | "OneSolution" | "ManySolutions";

/** The four kinds of move a record holds. */
export type MoveKind = "Place" | "Erase" | "WriteMark" | "StrikeMark";

/** The generator's five tiers, from the most givens to the fewest. */
export type Tier = "One" | "Two" | "Three" | "Four" | "Five";

/** A cell as `Playing` exposes it. */
export interface BoardCell {
    /** Its row, from 1 to 9. */
    row: number;
    /** Its column, from 1 to 9. */
    column: number;
    /** The digit it holds, given or placed, or `null` when it holds none. */
    digit: number | null;
    /** Whether its digit is one of the puzzle's givens. */
    is_given: boolean;
    /** Whether its digit equals a peer's. */
    is_conflicting: boolean;
    /** Its note's marks while the note is shown; absent while a digit stands. */
    note?: number[];
}

/** One move on the board's record, without the reading after it. */
export interface Move {
    /** Which move this is, counted from 1 in the order made. */
    index: number;
    /** What the player did. */
    kind: MoveKind;
    /** The cell it was made in. */
    target: Position;
    /** The digit placed, or the mark written or struck; `null` for an erasure. */
    digit: number | null;
    /** Whether it has been undone and not redone. */
    is_undone: boolean;
}

/** One check the player asked, with its answer. */
export interface Check {
    /** Which check this is, counted from 1 in the order asked. */
    index: number;
    /** How many moves stood when it was asked. */
    after_move: number;
    /** The cell checked. */
    target: Position;
    /** The digit the cell held then. */
    digit: number;
    /** Whether that digit is the solution's. */
    is_right: boolean;
}

/** Everything `Playing` exposes of a board, without the readings after its moves. */
export interface BoardView {
    /** Whether the board is solved. */
    status: Status;
    /** Whether every cell holds a digit. */
    is_full: boolean;
    /** Whether no digit equals a peer's. */
    is_consistent: boolean;
    /** Whether a move stands to undo. */
    can_undo: boolean;
    /** Whether an undone move waits to be redone. */
    can_redo: boolean;
    /** The 81 cells, in row order. */
    cells: BoardCell[];
    /** Every move on the record, undone ones included. */
    moves: Move[];
    /** Every check asked. */
    checks: Check[];
}

/** One cell as a move left it. */
export interface CellReading {
    /** Its row, from 1 to 9. */
    row: number;
    /** Its column, from 1 to 9. */
    column: number;
    /** The digit it held after the move, or `null`. */
    digit: number | null;
    /** The note it held after the move, shown or waiting beneath a digit. */
    note: number[];
}

/** The board as one move left it, which an undone move still has. */
export interface ReadingAfter {
    /** The move read, counted from 1. */
    move_index: number;
    /** The 81 cells, in row order. */
    cells: CellReading[];
}

/** One move as a record keeps it: its kind is the one key, holding the move's cell and digit. */
export type RecordMove =
    | { Place: { target: Position; digit: number } }
    | { Erase: { target: Position } }
    | { WriteMark: { target: Position; digit: number } }
    | { StrikeMark: { target: Position; digit: number } };

/** One check as a record keeps it; its answer is worked out again on reopening. */
export interface RecordCheck {
    /** How many moves stood when it was asked. */
    after_move: number;
    /** The cell checked. */
    target: Position;
    /** The digit the cell held then. */
    digit: number;
}

/**
 * What a board writes to be kept between visits, and what reopens it. Store it with
 * `JSON.stringify`; it holds no `bigint`. A record of another format version is refused.
 */
export interface BoardRecord {
    /** The format version it was written under; this package writes `recordVersion()`. */
    version: string;
    /** The puzzle's givens. */
    givens: Given[];
    /** Every move made, in order, undone ones included. */
    moves: RecordMove[];
    /** How many of the last moves are undone. */
    undone: number;
    /** Every check asked, in order. */
    checks: RecordCheck[];
}

/** One of `Playing`'s seven operations, put to a record by `play`. */
export type Operation =
    | { kind: "Place"; position: Position; digit: number }
    | { kind: "Erase"; position: Position }
    | { kind: "WriteMark"; position: Position; digit: number }
    | { kind: "StrikeMark"; position: Position; digit: number }
    | { kind: "CheckCell"; position: Position }
    | { kind: "Undo" }
    | { kind: "Redo" };

/** What play from a record hands back: the record after the call and the board's view. */
export interface Played {
    /** The record after the call, to keep and to hand to the next call. */
    record: BoardRecord;
    /** The view of the board after the call. */
    view: BoardView;
    /** The check the call added; present only after a `CheckCell`. */
    check?: Check;
}

/** What a search of some givens concluded. */
export interface SearchResult {
    /** No solution, one, or many. */
    verdict: Verdict;
    /** How many guesses the search made. */
    guesses: number;
    /** The solutions found, at most two, each nine rows of nine digits. */
    solutions: number[][][];
}

/**
 * A puzzle the generator made, with what replays it. Its three counts are `bigint`s, so
 * `JSON.stringify` throws on it: keep the seed as `String(seed)`.
 */
export interface GenerationResult {
    /** The tier asked for. */
    tier: Tier;
    /** The seed it was generated from, or `null` when it was generated from a stream. */
    seed: bigint | null;
    /** The stream's index when generation began. */
    start_index: bigint;
    /** How many draws generation took. */
    draws_taken: bigint;
    /** The generator's version, `generationVersion()`. */
    generation_version: string;
    /** The random stream's version, `randomVersion()`. */
    random_version: string;
    /** The puzzle's givens. The solution is never shown. */
    givens: Given[];
}

/** The `name` of every `Error` the engine throws for a refusal of its own. */
export type EngineErrorName =
    | "PlayError"
    | "ReopenError"
    | "SolveError"
    | "GenerateError"
    | "RandomError"
    | "FromRecordError";

/** What a refused generation throws, on either path. */
export interface GenerationRefused extends Error {
    name: "GenerateError";
    /** The tier asked for. */
    tier: Tier;
    /** How many draws were taken before the refusal. */
    draws_taken: bigint;
}
"#;

/// The board's values and the search's: `None` is `null`, as `serde_json` writes it,
/// and a map is a plain object. Never `bigint`s, so a record stays `JSON.stringify`-able.
const BOARD: Serializer = Serializer::new()
    .serialize_missing_as_null(true)
    .serialize_maps_as_objects(true);

/// The generation result's: as [`BOARD`], and a `u64` is a `bigint`, so a seed above
/// `2 ** 53` is exact. Nothing else may use it: it makes every `usize` a `bigint` too.
const GENERATION: Serializer = BOARD.serialize_large_number_types_as_bigints(true);

/// `value` as a board value or a search result crosses.
pub(crate) fn board<T: Serialize + ?Sized>(value: &T) -> Result<JsValue, JsValue> {
    value.serialize(&BOARD).map_err(JsValue::from)
}

/// `value` as the generation result crosses.
pub(crate) fn generation<T: Serialize + ?Sized>(value: &T) -> Result<JsValue, JsValue> {
    value.serialize(&GENERATION).map_err(JsValue::from)
}

/// `value` read as a `T`, or serde's refusal of its shape, an `Error` named `Error`.
/// A number parameter crosses through here so that `260` for a digit is refused rather
/// than wrapped.
pub(crate) fn read<T: DeserializeOwned>(value: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(JsValue::from)
}
