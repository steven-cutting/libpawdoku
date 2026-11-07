//! The engine as JavaScript receives it, run in Node by `wasm-bindgen-test-runner`
//! (`just wasm-test`): the held `Board` against play from a record, every refusal's
//! `name` and text, the record's version, the search and the generator. Values are
//! compared as JavaScript values, through `JSON.stringify`, and a generation result,
//! which holds `bigint`s, through a replacer. On the host this file compiles to nothing.

#![cfg(target_arch = "wasm32")]

use js_sys::{Function, JSON, Reflect};
use pawdoku::board::{FromRecordError, PlayError, RECORD_VERSION, ReopenError};
use pawdoku::generation::{GENERATION_VERSION, GenerateError, Tier};
use pawdoku::random::RandomError;
use pawdoku::solver::{SolveError, solve};
use pawdoku::sudoku::{Given, Position};
use pawdoku_wasm::{
    Board, RandomStream, generate, generate_from_seed, generation_version, open, play, read,
    reading_after, record_version, search,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::wasm_bindgen_test;

/// The rules' fixture: thirty givens, one solution, rows top to bottom.
const FIXTURE: &str =
    "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

/// The core's snapshot of the generation result of seed 0 in tier 3.
const SEED_ZERO_IN_TIER_3: &str = include_str!(
    "../../pawdoku/tests/snapshots/snapshots__tests__snapshot_the_generation_result_of_seed_zero_in_tier_3_as_json.snap"
);

/// A cell the fixture leaves to the player; its solution digit is 4.
const FREE: (u8, u8) = (1, 3);
/// A second free cell, in the row of [`FREE`].
const BESIDE: (u8, u8) = (1, 4);
/// Below the given 3 of row 1; its solution digit is 7.
const BELOW: (u8, u8) = (2, 2);
/// Below the given 5 of row 1, in its box.
const CORNER: (u8, u8) = (3, 1);
/// A cell the fixture gives: a 5.
const GIVEN: (u8, u8) = (1, 1);

/// The fixture's givens, in the core's type.
fn core_givens() -> Vec<Given> {
    let cells = FIXTURE.bytes().zip(0_u8..);
    cells
        .filter(|(cell, _)| cell.is_ascii_digit())
        .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'))
        .collect()
}

/// `text` parsed as JSON into a JavaScript value.
fn js(text: &str) -> JsValue {
    JSON::parse(text).unwrap_or_else(|_| JsValue::from_str(text))
}

/// The fixture's givens as JavaScript holds them.
fn givens() -> JsValue {
    let each: Vec<String> = core_givens()
        .iter()
        .map(|given| {
            let at = given.position();
            format!(
                r#"{{"position":{{"row":{},"column":{}}},"digit":{}}}"#,
                at.row(),
                at.column(),
                given.digit()
            )
        })
        .collect();
    js(&format!("[{}]", each.join(",")))
}

/// A position as JavaScript holds it.
fn position((row, column): (u8, u8)) -> JsValue {
    js(&format!(r#"{{"row":{row},"column":{column}}}"#))
}

/// `value` as `JSON.stringify` writes it, or the text of what it threw.
fn text(value: &JsValue) -> String {
    JSON::stringify(value).map_or_else(|thrown| message(&thrown), String::from)
}

/// `value` as `JSON.stringify` writes it, a `bigint` written as a number.
fn text_of_generation(value: &JsValue) -> String {
    let replacer = Function::new_with_args(
        "_key, value",
        "return typeof value === 'bigint' ? Number(value) : value",
    );
    JSON::stringify_with_replacer(value, &replacer)
        .map_or_else(|thrown| message(&thrown), String::from)
}

/// `property` of `value`.
fn get(value: &JsValue, property: &str) -> JsValue {
    Reflect::get(value, &JsValue::from_str(property)).unwrap_or(JsValue::UNDEFINED)
}

/// A thrown value's `message`.
fn message(thrown: &JsValue) -> String {
    thrown.dyn_ref::<js_sys::Error>().map_or_else(
        || format!("{thrown:?}"),
        |error| String::from(error.message()),
    )
}

/// A thrown value's `name` and `message`.
fn named(thrown: &JsValue) -> (String, String) {
    let name = thrown
        .dyn_ref::<js_sys::Error>()
        .map_or_else(String::new, |error| String::from(error.name()));
    (name, message(thrown))
}

/// One of `Playing`'s seven operations, as a script holds it: asked of a held board
/// and handed to `play` as an operation value.
#[derive(Debug, Clone, Copy)]
enum Op {
    Place((u8, u8), u8),
    Erase((u8, u8)),
    WriteMark((u8, u8), u8),
    StrikeMark((u8, u8), u8),
    CheckCell((u8, u8)),
    Undo,
    Redo,
}

impl Op {
    /// The operation value handed to `play`.
    fn operation(self) -> JsValue {
        let at =
            |(row, column): (u8, u8)| format!(r#""position":{{"row":{row},"column":{column}}}"#);
        js(&match self {
            Self::Place(cell, digit) => {
                format!(r#"{{"kind":"Place",{},"digit":{digit}}}"#, at(cell))
            }
            Self::Erase(cell) => format!(r#"{{"kind":"Erase",{}}}"#, at(cell)),
            Self::WriteMark(cell, digit) => {
                format!(r#"{{"kind":"WriteMark",{},"digit":{digit}}}"#, at(cell))
            }
            Self::StrikeMark(cell, digit) => {
                format!(r#"{{"kind":"StrikeMark",{},"digit":{digit}}}"#, at(cell))
            }
            Self::CheckCell(cell) => format!(r#"{{"kind":"CheckCell",{}}}"#, at(cell)),
            Self::Undo => String::from(r#"{"kind":"Undo"}"#),
            Self::Redo => String::from(r#"{"kind":"Redo"}"#),
        })
    }

    /// Asks it of a held board, with the check it added when it is a check.
    fn ask(self, board: &mut Board) -> Result<Option<JsValue>, JsValue> {
        let digit = |digit: u8| JsValue::from(digit);
        match self {
            Self::Place(cell, d) => board.place(position(cell), digit(d)).map(|()| None),
            Self::Erase(cell) => board.erase(position(cell)).map(|()| None),
            Self::WriteMark(cell, d) => board.write_mark(position(cell), digit(d)).map(|()| None),
            Self::StrikeMark(cell, d) => board.strike_mark(position(cell), digit(d)).map(|()| None),
            Self::CheckCell(cell) => board.check(position(cell)).map(Some),
            Self::Undo => board.undo().map(|()| None),
            Self::Redo => board.redo().map(|()| None),
        }
    }

    /// Asks it of the core's own board, for the refusal the core gives.
    fn ask_core(self, board: &mut pawdoku::board::Board) -> Result<(), PlayError> {
        let at = |(row, column): (u8, u8)| Position::new(row, column);
        match self {
            Self::Place(cell, digit) => board.place(at(cell), digit),
            Self::Erase(cell) => board.erase(at(cell)),
            Self::WriteMark(cell, digit) => board.write_mark(at(cell), digit),
            Self::StrikeMark(cell, digit) => board.strike_mark(at(cell), digit),
            Self::CheckCell(cell) => board.check(at(cell)).map(drop),
            Self::Undo => board.undo(),
            Self::Redo => board.redo(),
        }
    }
}

/// `THE_SCRIPTED_GAME` of `crates/pawdoku/tests/snapshots.rs`: every kind of move, an
/// undo, a redo, a move that discards an undone one, and a check answered each way.
const THE_SCRIPTED_GAME: [Op; 18] = [
    Op::WriteMark(BESIDE, 4),
    Op::WriteMark(BESIDE, 6),
    Op::WriteMark(BESIDE, 9),
    Op::StrikeMark(BESIDE, 9),
    Op::WriteMark(FREE, 2),
    Op::Place(FREE, 4),
    Op::CheckCell(FREE),
    Op::Place(BELOW, 3),
    Op::CheckCell(BELOW),
    Op::Erase(BELOW),
    Op::Place(CORNER, 5),
    Op::WriteMark(BELOW, 7),
    Op::Undo,
    Op::Redo,
    Op::Undo,
    Op::WriteMark(BELOW, 1),
    Op::Place(BELOW, 7),
    Op::Undo,
];

/// Refused operations, each asked of the scripted board: a given cell, a digit out of
/// range, a cell off the board, a mark in a cell that holds a digit, a check and an
/// erasure of a cell with no digit of the player's, a mark that is not there, and a
/// digit that already stands.
const REFUSED: [Op; 8] = [
    Op::Place(GIVEN, 4),
    Op::Place(BESIDE, 0),
    Op::Erase((10, 1)),
    Op::StrikeMark(CORNER, 8),
    Op::CheckCell(BESIDE),
    Op::Erase(BESIDE),
    Op::StrikeMark(BELOW, 9),
    Op::Place(FREE, 4),
];

/// The whole game: every empty cell given its solution digit, in grid order.
fn the_whole_game() -> Vec<Op> {
    let proof = solve(core_givens()).unwrap_or_else(|_| unreachable_fixture());
    let given = core_givens();
    let digits = proof.solution().iter().flatten().copied();
    let cells = (1..=9_u8).flat_map(|row| (1..=9_u8).map(move |column| (row, column)));
    cells
        .zip(digits)
        .filter(|&((row, column), _)| {
            !given
                .iter()
                .any(|g| g.position() == Position::new(row, column))
        })
        .map(|(cell, digit)| Op::Place(cell, digit))
        .collect()
}

/// The fixture has one solution; a test that finds otherwise fails here.
fn unreachable_fixture() -> ! {
    panic!("the fixture has one solution")
}

/// The scripted board, held, and its record as play from a record left it.
fn played(script: &[Op]) -> (Board, JsValue) {
    let mut board = Board::open(givens()).unwrap_or_else(|_| unreachable_fixture());
    let mut record = get(&open(givens()).unwrap_or(JsValue::NULL), "record");
    for &op in script {
        record = both_ways(&mut board, &record, op);
    }
    (board, record)
}

/// Asks `op` both ways and holds them to the same answer at this step: the same view,
/// record and check when it is accepted, the same `name` and text when it is not, and
/// every move's reading the same. Hands back the record the next call takes.
fn both_ways(board: &mut Board, record: &JsValue, op: Op) -> JsValue {
    let kept = text(record);
    let held = op.ask(board);
    let from_record = play(record.clone(), op.operation());
    assert_eq!(text(record), kept, "{op:?} changed the record handed over");
    let next = match (held, from_record) {
        (Ok(check), Ok(played)) => {
            assert_eq!(
                text(&get(&played, "view")),
                text(&board.view().unwrap_or_default()),
                "{op:?}"
            );
            assert_eq!(
                text(&get(&played, "record")),
                text(&board.write().unwrap_or_default()),
                "{op:?}"
            );
            let check = check.map_or_else(|| String::from("undefined"), |check| text(&check));
            let added = get(&played, "check");
            let added = if added.is_undefined() {
                String::from("undefined")
            } else {
                text(&added)
            };
            assert_eq!(added, check, "{op:?}");
            get(&played, "record")
        }
        (Err(held), Err(from_record)) => {
            assert_eq!(named(&from_record), named(&held), "{op:?}");
            record.clone()
        }
        (held, from_record) => panic!("{op:?}: held {held:?}, from a record {from_record:?}"),
    };
    readings_agree(board, &next);
    next
}

/// Every move's reading, both ways, and the index past the last refused both ways.
fn readings_agree(board: &Board, record: &JsValue) {
    let moves = get(&board.view().unwrap_or_default(), "moves");
    let count = js_sys::Array::from(&moves).length();
    for index in 1..=count + 1 {
        let index = JsValue::from(index);
        let held = board
            .reading_after(index.clone())
            .map(|r| text(&r))
            .map_err(|e| named(&e));
        let from_record = reading_after(record.clone(), index)
            .map(|r| text(&r))
            .map_err(|e| named(&e));
        assert_eq!(held, from_record);
    }
}

#[wasm_bindgen_test]
fn the_scripted_games_are_the_same_game_either_way() {
    let (_, record) = played(&THE_SCRIPTED_GAME);
    assert_eq!(text(&get(&record, "undone")), "1");
    let (board, record) = played(&the_whole_game());
    assert_eq!(
        text(&get(&board.view().unwrap_or_default(), "status")),
        r#""Solved""#
    );
    assert_eq!(
        text(&read(record).unwrap_or_default()),
        text(&board.view().unwrap_or_default())
    );
}

#[wasm_bindgen_test]
fn open_both_ways_gives_the_same_board() {
    let board = Board::open(givens()).unwrap_or_else(|_| unreachable_fixture());
    let opened = open(givens()).unwrap_or_default();
    assert_eq!(
        text(&get(&opened, "view")),
        text(&board.view().unwrap_or_default())
    );
    assert_eq!(
        text(&get(&opened, "record")),
        text(&board.write().unwrap_or_default())
    );
    assert!(get(&opened, "check").is_undefined());
}

#[wasm_bindgen_test]
fn givens_with_no_board_are_refused_with_the_solvers_text_either_way() {
    let refusal = (
        String::from("SolveError"),
        SolveError::ManySolutions.to_string(),
    );
    let none = js("[]");
    assert_eq!(
        Board::open(none.clone()).map(drop).map_err(|e| named(&e)),
        Err(refusal.clone())
    );
    assert_eq!(open(none).map(drop).map_err(|e| named(&e)), Err(refusal));
}

#[wasm_bindgen_test]
fn each_refusal_throws_the_boards_text_and_changes_nothing() {
    let (mut board, record) = played(&THE_SCRIPTED_GAME);
    for op in REFUSED {
        let (view, written) = (
            text(&board.view().unwrap_or_default()),
            text(&board.write().unwrap_or_default()),
        );
        let held = op.ask(&mut board).map(drop).map_err(|e| named(&e));
        let Err((name, said)) = held else {
            panic!("{op:?} was not refused")
        };
        assert_eq!(name, "PlayError", "{op:?}");
        let core = op.ask_core(&mut core_board(&THE_SCRIPTED_GAME));
        assert_eq!(
            core.map_err(|refusal| refusal.to_string()),
            Err(said.clone()),
            "{op:?}"
        );
        assert_eq!(text(&board.view().unwrap_or_default()), view, "{op:?}");
        assert_eq!(text(&board.write().unwrap_or_default()), written, "{op:?}");
        let from_record = play(record.clone(), op.operation())
            .map(drop)
            .map_err(|e| named(&e));
        assert_eq!(from_record, Err((name, said)), "{op:?}");
    }
}

/// The scripted game played on the core's own board, to ask it what it refuses.
fn core_board(script: &[Op]) -> pawdoku::board::Board {
    let mut board =
        pawdoku::board::Board::open(core_givens()).unwrap_or_else(|_| unreachable_fixture());
    for &op in script {
        drop(op.ask_core(&mut board));
    }
    board
}

#[wasm_bindgen_test]
fn a_number_that_is_not_a_digit_or_an_index_is_refused_not_wrapped() {
    let (mut board, record) = played(&THE_SCRIPTED_GAME);
    let (view, written) = (
        text(&board.view().unwrap_or_default()),
        text(&board.write().unwrap_or_default()),
    );
    let wide = String::from("invalid value: integer `260`, expected u8");
    let place = board
        .place(position(CORNER), JsValue::from(260))
        .map_err(|e| named(&e));
    assert_eq!(place, Err((String::from("Error"), wide.clone())));
    let mark = board
        .write_mark(position(CORNER), JsValue::from(260))
        .map_err(|e| named(&e));
    assert_eq!(mark, Err((String::from("Error"), wide.clone())));
    for index in [1.5, 4_294_967_297.0] {
        let reading = board
            .reading_after(JsValue::from(index))
            .map(drop)
            .map_err(|e| named(&e));
        let Err((name, _)) = reading else {
            panic!("{index} was read")
        };
        assert_eq!(name, "Error");
        let stateless = reading_after(record.clone(), JsValue::from(index))
            .map(drop)
            .map_err(|e| named(&e));
        assert_eq!(
            stateless.map_err(|(name, _)| name),
            Err(String::from("Error"))
        );
    }
    assert_eq!(text(&board.view().unwrap_or_default()), view);
    assert_eq!(text(&board.write().unwrap_or_default()), written);
    let operation = js(r#"{"kind":"Place","position":{"row":3,"column":1},"digit":260}"#);
    let from_record = play(record, operation).map(drop).map_err(|e| named(&e));
    assert_eq!(from_record, Err((String::from("Error"), wide)));
}

#[wasm_bindgen_test]
fn an_index_that_names_no_move_is_refused_with_its_own_name() {
    let (board, record) = played(&THE_SCRIPTED_GAME);
    let none = FromRecordError::NoSuchMove { move_index: 0 };
    let refusal = Err((String::from("FromRecordError"), none.to_string()));
    let held = board
        .reading_after(JsValue::from(0))
        .map(drop)
        .map_err(|e| named(&e));
    assert_eq!(held, refusal);
    let stateless = reading_after(record, JsValue::from(0))
        .map(drop)
        .map_err(|e| named(&e));
    assert_eq!(stateless, refusal);
}

/// The scripted record with its version replaced by `version`, or removed when `None`,
/// and with `undone` renamed when `renamed`.
fn another_version(version: Option<&str>, renamed: bool) -> JsValue {
    let (_, record) = played(&THE_SCRIPTED_GAME);
    let mut written = text(&record);
    let this = format!(r#""version":"{RECORD_VERSION}","#);
    let that = version.map_or_else(String::new, |version| format!(r#""version":"{version}","#));
    written = written.replace(&this, &that);
    if renamed {
        written = written.replace(r#""undone":"#, r#""undone_count":"#);
    }
    js(&written)
}

#[wasm_bindgen_test]
fn a_record_of_another_version_is_refused_naming_both_either_way() {
    for (version, renamed) in [
        (Some("record-9"), false),
        (Some("record-9"), true),
        (None, false),
        (None, true),
    ] {
        let record = another_version(version, renamed);
        let refusal = ReopenError::Version {
            named: version.map(String::from),
            reads: RECORD_VERSION,
        };
        let said = refusal.to_string();
        assert!(said.contains(RECORD_VERSION), "{said}");
        if let Some(version) = version {
            assert!(said.contains(version), "{said}");
        }
        let expected = Err((String::from("ReopenError"), said));
        let wrapped = FromRecordError::from(refusal).to_string();
        assert_eq!(expected.as_ref().map_err(|(_, said)| said), Err(&wrapped));
        assert_eq!(
            Board::reopen(record.clone())
                .map(drop)
                .map_err(|e| named(&e)),
            expected
        );
        assert_eq!(
            read(record.clone()).map(drop).map_err(|e| named(&e)),
            expected
        );
        let undo = Op::Undo.operation();
        assert_eq!(
            play(record.clone(), undo).map(drop).map_err(|e| named(&e)),
            expected
        );
        let first = JsValue::from(1);
        assert_eq!(
            reading_after(record, first)
                .map(drop)
                .map_err(|e| named(&e)),
            expected
        );
    }
}

#[wasm_bindgen_test]
fn a_record_of_the_wrong_shape_throws_serdes_error() {
    let record = js(&format!(r#"{{"version":"{RECORD_VERSION}","givens":[]}}"#));
    let refused = Board::reopen(record.clone())
        .map(drop)
        .map_err(|e| named(&e));
    assert_eq!(
        refused,
        Err((String::from("Error"), String::from("missing field `moves`")))
    );
    assert_eq!(read(record).map(drop).map_err(|e| named(&e)), refused);
    let (_, record) = played(&THE_SCRIPTED_GAME);
    let fly = play(record, js(r#"{"kind":"Fly"}"#))
        .map(drop)
        .map_err(|e| named(&e));
    assert_eq!(fly.map_err(|(name, _)| name), Err(String::from("Error")));
}

#[wasm_bindgen_test]
fn the_record_version_is_the_engines() {
    assert_eq!(record_version(), RECORD_VERSION);
    assert_eq!(record_version(), "record-1");
    let (board, _) = played(&[]);
    assert_eq!(
        text(&get(&board.write().unwrap_or_default(), "version")),
        r#""record-1""#
    );
}

#[wasm_bindgen_test]
fn a_record_through_json_reopens_to_the_same_board_and_holds_only_numbers() {
    let (board, _) = played(&THE_SCRIPTED_GAME);
    let record = board.write().unwrap_or_default();
    let kept = JSON::parse(&text(&record)).unwrap_or_default();
    let reopened = Board::reopen(kept).unwrap_or_else(|e| panic!("{:?}", named(&e)));
    assert_eq!(
        text(&reopened.view().unwrap_or_default()),
        text(&board.view().unwrap_or_default())
    );
    let view = board.view().unwrap_or_default();
    let first = |list: JsValue| js_sys::Array::from(&list).get(0);
    assert_eq!(
        get(&first(get(&view, "moves")), "index").js_typeof(),
        "number"
    );
    assert_eq!(
        get(&first(get(&view, "checks")), "after_move").js_typeof(),
        "number"
    );
    assert_eq!(get(&record, "undone").js_typeof(), "number");
}

#[wasm_bindgen_test]
fn the_search_of_the_fixture_is_the_cores() {
    let core = pawdoku::solver::search(core_givens());
    let result = search(givens()).unwrap_or_default();
    assert_eq!(text(&get(&result, "verdict")), r#""OneSolution""#);
    assert_eq!(text(&get(&result, "guesses")), core.guesses().to_string());
    let solutions = js_sys::Array::from(&get(&result, "solutions"));
    assert_eq!(solutions.length(), 1);
    let rows: Vec<String> = core.solutions()[0]
        .iter()
        .map(|row| format!("[{}]", row.map(|digit| digit.to_string()).join(",")))
        .collect();
    assert_eq!(text(&solutions.get(0)), format!("[{}]", rows.join(",")));
}

#[wasm_bindgen_test]
fn generation_from_seed_zero_in_tier_3_is_the_cores_snapshot() {
    let result = generate_from_seed(JsValue::from_str("Three"), 0).unwrap_or_default();
    let json = SEED_ZERO_IN_TIER_3
        .splitn(3, "---\n")
        .nth(2)
        .unwrap_or_default();
    assert_eq!(text_of_generation(&result), text(&js(json)));
    assert_eq!(get(&result, "draws_taken"), JsValue::from(23_u64));
    for field in ["seed", "start_index", "draws_taken"] {
        assert_eq!(get(&result, field).js_typeof(), "bigint", "{field}");
    }
    assert_eq!(generation_version(), GENERATION_VERSION);
    assert_eq!(generation_version(), "generation-1");
}

#[wasm_bindgen_test]
fn a_seed_past_two_to_the_53_survives_the_round_trip() {
    let seed = 1_u64 << 63;
    let result = generate_from_seed(JsValue::from_str("Five"), seed).unwrap_or_default();
    assert_eq!(get(&result, "seed"), JsValue::from(seed));
    let same = generate_from_seed(JsValue::from_str("Five"), seed).unwrap_or_default();
    assert_eq!(text_of_generation(&same), text_of_generation(&result));
}

#[wasm_bindgen_test]
fn generate_advances_the_stream_it_is_handed_by_the_draws_it_takes() {
    let mut stream = RandomStream::seeded(2024);
    for _ in 0..3 {
        assert!(stream.next_draw().is_ok());
    }
    let result = generate(JsValue::from_str("Two"), &mut stream).unwrap_or_default();
    assert!(get(&result, "seed").is_null());
    assert_eq!(get(&result, "start_index"), JsValue::from(3_u64));
    let taken = get(&result, "draws_taken");
    assert_eq!(JsValue::from(stream.index() - 3), taken);
    // Replayed from the same seed at the same index, the same givens.
    let mut again = RandomStream::seeded(2024);
    for _ in 0..3 {
        assert!(again.next_draw().is_ok());
    }
    let replayed = generate(JsValue::from_str("Two"), &mut again).unwrap_or_default();
    assert_eq!(text_of_generation(&replayed), text_of_generation(&result));
}

/// A refused `generate`'s `name`, text, `tier` and `draws_taken`, and the stream's index.
fn refused_from(tier: &str, mut stream: RandomStream) -> (String, String, JsValue, JsValue, u64) {
    let thrown = generate(JsValue::from_str(tier), &mut stream).map(drop);
    let Err(thrown) = thrown else {
        panic!("generation in tier {tier} was not refused")
    };
    assert!(
        JSON::stringify(&thrown).is_ok(),
        "JSON.stringify of the refusal threw"
    );
    let (name, said) = named(&thrown);
    (
        name,
        said,
        get(&thrown, "tier"),
        get(&thrown, "draws_taken"),
        stream.index(),
    )
}

/// A replay stream of `draws`.
fn replay(draws: Vec<f64>) -> RandomStream {
    RandomStream::replay(draws).unwrap_or_else(|e| panic!("{:?}", named(&e)))
}

#[wasm_bindgen_test]
fn a_stream_that_runs_out_leaves_its_draws_taken() {
    let exhausted = GenerateError::from(RandomError::Exhausted { index: 5 }).to_string();
    assert_eq!(exhausted, "the replay stream is exhausted at draw 5");
    for tier in ["Three", "One", "Five"] {
        let (name, said, at, taken, index) = refused_from(tier, replay(vec![0.0; 5]));
        assert_eq!(
            (name.as_str(), said.as_str()),
            ("GenerateError", exhausted.as_str())
        );
        assert_eq!(at, JsValue::from_str(tier));
        assert_eq!((taken, index), (JsValue::from(5_u64), 5));
    }
    let mut drawn = replay(vec![0.0; 5]);
    for _ in 0..3 {
        assert!(drawn.next_draw().is_ok());
    }
    let (_, _, at, taken, index) = refused_from("Two", drawn);
    assert_eq!(
        (at, taken, index),
        (JsValue::from_str("Two"), JsValue::from(2_u64), 5)
    );
}

#[wasm_bindgen_test]
fn a_hundred_failed_attempts_are_refused_with_every_draw_counted() {
    let short = short_seeding();
    let draws: Vec<f64> = (0..100).flat_map(|_| short.iter().copied()).collect();
    let (name, said, at, taken, index) = refused_from("Four", replay(draws));
    assert_eq!(name, "GenerateError");
    assert_eq!(
        said,
        GenerateError::GridAttemptsSpent { attempts: 100 }.to_string()
    );
    assert_eq!(
        (at, taken, index),
        (JsValue::from_str("Four"), JsValue::from(1900_u64), 1900)
    );
    assert_eq!(Tier::Four.number(), 4);
}

/// One failed grid attempt of nineteen draws, `a_short_seeding` of
/// `crates/pawdoku/src/generation/grid.rs`.
fn short_seeding() -> Vec<f64> {
    let pick = |index: u8, n: u8| (f64::from(index) + 0.5) / f64::from(n);
    let mut script = vec![0.0; 16];
    script.extend([pick(9, 73), pick(6, 7), 0.0]);
    script
}

#[wasm_bindgen_test]
fn a_random_refusal_is_named_random_error() {
    let refused = RandomStream::replay(vec![0.5, 1.0])
        .map(drop)
        .map_err(|e| named(&e));
    let text = RandomError::OutOfRange {
        index: 1,
        value: 1.0,
    }
    .to_string();
    assert_eq!(refused, Err((String::from("RandomError"), text)));
}

/// serde's text for an object refused unread while a class is borrowed.
const UNREAD: &str = "invalid type: an object, expected a value";

/// The value `body` returns, built in JavaScript: an object whose getter or trap throws.
fn hostile(body: &str) -> JsValue {
    Function::new_no_args(body)
        .call0(&JsValue::NULL)
        .unwrap_or_else(|e| panic!("{:?}", named(&e)))
}

#[wasm_bindgen_test]
fn a_value_that_throws_as_it_is_read_comes_back_as_itself_and_changes_nothing() {
    let (mut board, _) = played(&THE_SCRIPTED_GAME);
    let (view, written) = (
        text(&board.view().unwrap_or_default()),
        text(&board.write().unwrap_or_default()),
    );
    let getter = || hostile("return { get row() { throw new RangeError('row'); }, column: 3 };");
    let proxy = || {
        hostile(
            "const trap = () => { throw new RangeError('trap'); }; \
                return new Proxy({}, { get: trap, has: trap, ownKeys: trap, getPrototypeOf: trap });",
        )
    };
    let tagged = || {
        hostile(
            "return { get [Symbol.toStringTag]() { throw new RangeError('tag'); }, \
                toJSON() { throw new RangeError('json'); } };",
        )
    };
    let row = (String::from("RangeError"), String::from("row"));
    let trap = (String::from("RangeError"), String::from("trap"));
    let unread = (String::from("Error"), String::from(UNREAD));
    let digit = JsValue::from(4);
    let asked = [
        (
            board.place(getter(), digit.clone()).map_err(|e| named(&e)),
            &row,
        ),
        (board.erase(getter()).map_err(|e| named(&e)), &row),
        (
            board
                .write_mark(proxy(), digit.clone())
                .map_err(|e| named(&e)),
            &trap,
        ),
        (
            board
                .strike_mark(proxy(), digit.clone())
                .map_err(|e| named(&e)),
            &trap,
        ),
        (board.check(getter()).map(drop).map_err(|e| named(&e)), &row),
        (
            board.place(position(FREE), tagged()).map_err(|e| named(&e)),
            &unread,
        ),
        (
            board.place(position(FREE), proxy()).map_err(|e| named(&e)),
            &unread,
        ),
        (
            board
                .reading_after(tagged())
                .map(drop)
                .map_err(|e| named(&e)),
            &unread,
        ),
    ];
    for (at, (refused, thrown)) in asked.into_iter().enumerate() {
        assert_eq!(refused, Err(thrown.clone()), "call {at}");
    }
    let function = board
        .place(position(FREE), hostile("return function digit() {};"))
        .map_err(|e| named(&e));
    assert_eq!(
        function.map_err(|(name, _)| name),
        Err(String::from("Error"))
    );
    assert_eq!(text(&board.view().unwrap_or_default()), view);
    assert_eq!(text(&board.write().unwrap_or_default()), written);
}

#[wasm_bindgen_test]
fn a_board_reads_a_position_as_play_does_from_its_own_fields() {
    let own = "return new (class { constructor() { this.row = 1; this.column = 3; } })();";
    let inherited = "return new (class { get row() { return 1; } get column() { return 3; } })();";
    let elsewhere = "return { row: 1, column: 3, toJSON() { return { row: 1, column: 4 }; } };";
    let (placed, _) = played(&[Op::Place(FREE, 4)]);
    let placed = text(&placed.write().unwrap_or_default());
    let asked = [
        (own, Some(&placed)),
        (inherited, None),
        (elsewhere, None),
        ("return [1, 3];", None),
    ];
    for (body, answer) in asked {
        let mut board = Board::open(givens()).unwrap_or_else(|_| unreachable_fixture());
        let held = board
            .place(hostile(body), JsValue::from(4))
            .map(|()| text(&board.write().unwrap_or_default()))
            .map_err(|e| named(&e).0);
        let place = js(r#"{"kind":"Place","digit":4}"#);
        Reflect::set(&place, &JsValue::from_str("position"), &hostile(body)).unwrap_or_default();
        let record = get(&open(givens()).unwrap_or_default(), "record");
        let from_record = play(record, place)
            .map(|played| text(&get(&played, "record")))
            .map_err(|e| named(&e).0);
        assert_eq!(held, from_record, "{body}");
        let expected = answer.cloned().ok_or_else(|| String::from("Error"));
        assert_eq!(held, expected, "{body}");
    }
}

#[wasm_bindgen_test]
fn a_tier_that_is_not_a_string_is_refused_unread_and_the_stream_stays_usable() {
    let mut stream = RandomStream::seeded(2024);
    let getter = hostile("return { get Three() { throw new RangeError('tier'); } };");
    let refused = generate(getter, &mut stream)
        .map(drop)
        .map_err(|e| named(&e));
    assert_eq!(refused, Err((String::from("Error"), String::from(UNREAD))));
    assert_eq!(stream.index(), 0);
    let mut fresh = RandomStream::seeded(2024);
    let next = (stream.next_draw().ok(), fresh.next_draw().ok());
    assert_eq!(next.0, next.1);
    let (again, expected) = (
        generate(JsValue::from_str("Three"), &mut stream).unwrap_or_default(),
        generate(JsValue::from_str("Three"), &mut fresh).unwrap_or_default(),
    );
    assert_eq!(text_of_generation(&again), text_of_generation(&expected));
}
