//! The record: a board written down whole, and reopened from what was written.

use super::Board;
use super::check::Check;
use super::error::PlayError;
use super::journal::{Journal, Made};
use crate::solver::SolveError;
use crate::sudoku::{Given, Position, Puzzle, Status, in_range};
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

/// A board written down: what [`Board::write`] hands a caller, and what
/// [`Board::reopen`] makes the same board from.
///
/// A record is a plain value. The engine keeps nothing: where a record is kept, and in
/// what form, is the caller's. Under the `serde` feature it serialises, through
/// whatever format a consumer brings; the library has no format of its own.
///
/// # What a record holds
///
/// The least that says which board it was:
///
/// - the givens;
/// - the moves in the order made, each with its kind, its target and its digit;
/// - how many of them are undone, which are always the latest;
/// - each check, with how many moves stood when it was asked, its target and its
///   digit.
///
/// Nothing in it can be worked out from the rest: no cell's digit, no note, nothing a
/// move displaced or upkeep struck, no check's answer. And nothing in it is read from
/// the solution. Reopening puts the givens to the solver, which finds the solution
/// again, and works the rest out by making the moves.
///
/// # What reopening holds a record to
///
/// A record can say anything. Its fields are private, but a deserialised record is
/// whatever its source held: deserialising checks nothing, and [`Board::reopen`]
/// checks everything it can. A record that no board could have been written to is
/// refused with a [`ReopenError`], and never with a panic.
///
/// One thing cannot be read back. A check holds the digit that stood in its cell when
/// it was asked, and after an undo and a new move the moves that stood then may be
/// gone from the record. So a check is held to what can be known: its target is a cell
/// of the grid that is not a given, its digit is from 1 to 9, it was asked after at
/// least one move, a record that holds a check holds a move, and its digit is not one
/// that would have solved the puzzle as it was placed, which is so only for the right
/// digit of a puzzle with one cell to play. Its digit is not compared with any move,
/// and its count of moves may exceed the moves the record holds. Its answer is not in the record, so a record cannot lie about it: the answer
/// is worked out again from the solution.
///
/// # Across versions
///
/// No promise is made yet that a record written by one version of the engine reopens
/// in another. A record carries no version.
///
/// ```
/// use pawdoku::board::{Board, Record};
/// use pawdoku::sudoku::{Given, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let mut board = Board::open(givens)?;
/// let here = Position::new(1, 3);
/// board.place(here, 4)?;
/// board.check(here)?;
/// board.undo()?;
///
/// let record: Record = board.write();
/// let reopened = Board::reopen(&record)?;
///
/// // The same moves with the same one undone, and the same check with its answer.
/// assert_eq!(reopened.moves().collect::<Vec<_>>(), board.moves().collect::<Vec<_>>());
/// assert_eq!(reopened.checks().collect::<Vec<_>>(), board.checks().collect::<Vec<_>>());
/// assert!(reopened.can_redo());
///
/// // A record is a value: it compares, and the reopened board writes the same one.
/// assert_eq!(reopened.write(), record);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Record {
    givens: BTreeSet<Given>,
    moves: Vec<Written>,
    undone: usize,
    checks: Vec<Asked>,
}

impl Record {
    /// `Recording.write`: the record of the board that holds this puzzle, these moves
    /// and these checks. It reads them and changes none.
    pub(super) fn of(puzzle: &Puzzle, journal: &Journal, checks: &[Check]) -> Self {
        let made = journal.made();
        Self {
            givens: puzzle.givens().clone(),
            moves: made.iter().map(Written::from).collect(),
            undone: made.len().saturating_sub(journal.standing()),
            checks: checks.iter().map(Asked::from).collect(),
        }
    }
}

/// One move as a record holds it: its kind, its target and its digit. Each kind
/// carries what it needs and nothing else, so a record cannot say a placement has no
/// digit or an erasure has one. What the move displaced and what upkeep struck are
/// left out: making the move again finds them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum Written {
    /// A digit placed.
    Place { target: Position, digit: u8 },
    /// A digit erased.
    Erase { target: Position },
    /// A mark written.
    WriteMark { target: Position, digit: u8 },
    /// A mark struck.
    StrikeMark { target: Position, digit: u8 },
}

/// What a record keeps of a move the board made.
impl From<&Made> for Written {
    fn from(made: &Made) -> Self {
        match *made {
            Made::Place { target, digit, .. } => Self::Place { target, digit },
            Made::Erase { target, .. } => Self::Erase { target },
            Made::WriteMark { target, digit } => Self::WriteMark { target, digit },
            Made::StrikeMark { target, digit } => Self::StrikeMark { target, digit },
        }
    }
}

impl Written {
    /// Makes the move on `board`, through the board's own guards.
    fn make(self, board: &mut Board) -> Result<(), PlayError> {
        match self {
            Self::Place { target, digit } => board.place(target, digit),
            Self::Erase { target } => board.erase(target),
            Self::WriteMark { target, digit } => board.write_mark(target, digit),
            Self::StrikeMark { target, digit } => board.strike_mark(target, digit),
        }
    }
}

/// One check as a record holds it: how many moves stood when it was asked, its target
/// and its digit. Its index is its place in the record, and its answer is left out:
/// the solution gives it again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Asked {
    after_move: usize,
    target: Position,
    digit: u8,
}

/// What a record keeps of a check the board answered.
impl From<&Check> for Asked {
    fn from(check: &Check) -> Self {
        Self {
            after_move: check.after_move(),
            target: check.target(),
            digit: check.digit(),
        }
    }
}

impl Asked {
    /// The check as `board` keeps it, at `index`, with its answer worked out from the
    /// solution, or the refusal of a check no board answered.
    fn read_back(self, index: usize, board: &Board) -> Result<Check, ReopenError> {
        let Self {
            after_move,
            target,
            digit,
        } = self;
        let refused = |source| ReopenError::Check { index, source };
        let cell = board.cell(target);
        let cell = cell.ok_or_else(|| refused(PlayError::NoSuchCell { position: target }))?;
        let given = refused(PlayError::GivenCell { position: target });
        require(!cell.is_given(), given)?;
        let out_of_range = refused(PlayError::DigitOutOfRange { digit });
        require(in_range(digit), out_of_range)?;
        require(after_move >= 1, ReopenError::CheckAfterNoMove { index })?;
        let is_right = board.puzzle.is_solution_digit(target, digit);
        // With one cell to play, its right digit solves the puzzle as it is placed.
        let to_play = board.cells().filter(|cell| !cell.is_given()).count();
        require(!is_right || to_play > 1, refused(PlayError::Solved))?;
        Ok(Check::new(index, after_move, target, digit, is_right))
    }
}

/// Why a record was not reopened: one variant for each way a record fails to read
/// back.
///
/// A refused reopening gives no board, and leaves the record as it was. When a record
/// fails in several ways the first is reported, in the order [`Board::reopen`]
/// documents.
///
/// The `Display` text of each variant is stable API: bindings build their exceptions
/// from it. Where a variant carries the solver's or the board's own refusal, that
/// refusal is its source and its text closes the variant's.
///
/// ```
/// use pawdoku::board::{PlayError, ReopenError};
/// use pawdoku::sudoku::Position;
///
/// let given = Position::new(1, 1);
/// let refusal = ReopenError::Move { index: 3, source: PlayError::GivenCell { position: given } };
/// assert_eq!(
///     refusal.to_string(),
///     "move 3 of the record does not read back: the cell at row 1, column 1 holds a given"
/// );
///
/// let refusal = ReopenError::TooManyUndone { undone: 2, moves: 1 };
/// assert_eq!(refusal.to_string(), "the record's undone count, 2, exceeds its move count, 1");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ReopenError {
    /// The record says more moves are undone than it holds.
    #[error("the record's undone count, {undone}, exceeds its move count, {moves}")]
    TooManyUndone {
        /// How many moves the record says are undone.
        undone: usize,
        /// How many moves the record holds.
        moves: usize,
    },
    /// The record holds checks and no move. A check is asked of a digit the player
    /// placed, and a record keeps a move from the first one made, so no board writes
    /// this.
    #[error("the record holds no move, and its check count is {checks}")]
    ChecksWithoutMoves {
        /// How many checks the record holds.
        checks: usize,
    },
    /// The solver refused the record's givens: they have no solution, or several, or
    /// leave nothing to play. They are givens no board opens on.
    #[error("the record's givens open no board: {source}")]
    Givens {
        /// The solver's refusal.
        source: SolveError,
    },
    /// The board's own guards refused one of the record's moves, made in its turn
    /// after the moves before it.
    #[error("move {index} of the record does not read back: {source}")]
    Move {
        /// Which move, counted from 1 in the record's order.
        index: usize,
        /// The board's refusal.
        source: PlayError,
    },
    /// The moves before this one solve the puzzle, and the record goes on. Solved is
    /// final, so no board made this move.
    #[error("move {index} of the record comes after the puzzle is solved")]
    MoveAfterSolved {
        /// Which move, counted from 1 in the record's order.
        index: usize,
    },
    /// The record's moves solve the puzzle and it says some are undone. Nothing is
    /// taken back from a solved puzzle, so no board stood this way.
    #[error("the record's moves solve the puzzle, and its undone count is {undone}")]
    UndoneWhenSolved {
        /// How many moves the record says are undone.
        undone: usize,
    },
    /// One of the record's checks is one no board answers: its target is off the grid
    /// or a given, its digit is not from 1 to 9, or its digit solves the puzzle, which
    /// the right digit does where one cell is left to play. The source is what a board
    /// says of such a check.
    #[error("check {index} of the record does not read back: {source}")]
    Check {
        /// Which check, counted from 1 in the record's order.
        index: usize,
        /// What a board would say of it.
        source: PlayError,
    },
    /// One of the record's checks says it was asked when no move stood. A check is
    /// asked of a digit the player placed, and only a standing move puts one there.
    #[error("check {index} of the record was asked after no move")]
    CheckAfterNoMove {
        /// Which check, counted from 1 in the record's order.
        index: usize,
    },
}

/// Passes when what reopening requires of a record `holds`, and refuses with `refusal`
/// when it does not.
const fn require(holds: bool, refusal: ReopenError) -> Result<(), ReopenError> {
    if holds { Ok(()) } else { Err(refusal) }
}

/// What can be asked of a record's two counts before anything is made: no more moves
/// are undone than the record holds, and a record that holds a check holds a move.
fn counted(record: &Record) -> Result<(), ReopenError> {
    let (undone, moves, checks) = (record.undone, record.moves.len(), record.checks.len());
    require(
        undone <= moves,
        ReopenError::TooManyUndone { undone, moves },
    )?;
    let has_a_move = moves > 0 || checks == 0;
    require(has_a_move, ReopenError::ChecksWithoutMoves { checks })
}

/// Makes every move of the record on `board`, in order, each through the board's own
/// guards. A move that would follow the one that solves the puzzle is refused first.
fn replay(board: &mut Board, moves: &[Written]) -> Result<(), ReopenError> {
    for (index, written) in (1..).zip(moves) {
        let in_play = board.status() == Status::Unsolved;
        require(in_play, ReopenError::MoveAfterSolved { index })?;
        let made = written.make(board);
        made.map_err(|source| ReopenError::Move { index, source })?;
    }
    Ok(())
}

/// Takes back the latest `undone` moves, which leaves them on the board's record,
/// undone, for redo to re-take.
///
/// Every move stands when this is called and `undone` is no more than their count, so
/// a move is always there to take back. The one refusal left to the board's guard is
/// that the puzzle is solved, and nothing is taken back from a solved puzzle.
fn take_back(board: &mut Board, undone: usize) -> Result<(), ReopenError> {
    for _ in 0..undone {
        let taken = board.undo();
        taken.or(Err(ReopenError::UndoneWhenSolved { undone }))?;
    }
    Ok(())
}

/// Puts every check of the record on `board`, in order, each held to what a check can
/// be held to and answered again from the solution.
fn ask_again(board: &mut Board, checks: &[Asked]) -> Result<(), ReopenError> {
    for (index, asked) in (1..).zip(checks) {
        let check = asked.read_back(index, board)?;
        board.checks.push(check);
    }
    Ok(())
}

/// `Recording.reopen`: the board `record` was written from, or why there is none.
pub(super) fn reopened(record: &Record) -> Result<Board, ReopenError> {
    counted(record)?;
    let givens = record.givens.iter().copied();
    let board = Board::open(givens);
    let mut board = board.map_err(|source| ReopenError::Givens { source })?;
    replay(&mut board, &record.moves)?;
    take_back(&mut board, record.undone)?;
    ask_again(&mut board, &record.checks)?;
    Ok(board)
}

#[cfg(test)]
mod tests {
    use super::{Asked, Record, ReopenError, Written};
    use crate::board::{Board, Check, PlayError};
    use crate::solver::{SolveError, solve};
    use crate::sudoku::{Given, Position, WellPosedError};
    use alloc::collections::BTreeSet;
    use alloc::format;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    /// The rules' fixture: thirty givens, one solution, rows top to bottom.
    const FIXTURE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    /// A cell the fixture leaves to the player. Its solution digit is 4.
    const FREE: Position = Position::new(1, 3);

    /// A second cell left to the player, in the row of [`FREE`].
    const BESIDE: Position = Position::new(1, 4);

    /// A cell the fixture gives: a 5.
    const GIVEN: Position = Position::new(1, 1);

    /// A position the grid does not have.
    const OFF_THE_GRID: Position = Position::new(0, 10);

    /// The filled cells of a picture read a row after a row, as givens.
    fn givens(picture: &str) -> BTreeSet<Given> {
        let cells = picture.bytes().zip(0_u8..);
        cells
            .filter(|(cell, _)| cell.is_ascii_digit())
            .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'))
            .collect()
    }

    /// A record built field by field on the fixture's givens, as a deserialised one
    /// might arrive: nothing has checked it.
    fn record(moves: &[Written], undone: usize, checks: &[Asked]) -> Record {
        Record {
            givens: givens(FIXTURE),
            moves: moves.to_vec(),
            undone,
            checks: checks.to_vec(),
        }
    }

    /// A record of no move and no check on other givens.
    fn record_of(givens: BTreeSet<Given>) -> Record {
        Record {
            givens,
            moves: Vec::new(),
            undone: 0,
            checks: Vec::new(),
        }
    }

    /// Why `record` is not reopened.
    fn refusal(record: &Record) -> ReopenError {
        Board::reopen(record).unwrap_err()
    }

    /// The fixture's one solution, a row after a row.
    fn solution() -> String {
        let proof = solve(givens(FIXTURE)).unwrap();
        let digits = proof.solution().iter().flatten();
        digits.map(|digit| char::from(b'0' + digit)).collect()
    }

    /// The fifty-one placements that solve the fixture, in grid order.
    fn solving() -> Vec<Written> {
        let given = givens(FIXTURE);
        let placed = givens(&solution()).into_iter();
        let placed = placed.filter(|right| !given.contains(right));
        placed
            .map(|right| Written::Place {
                target: right.position(),
                digit: right.digit(),
            })
            .collect()
    }

    const PLACED: Written = Written::Place {
        target: FREE,
        digit: 4,
    };

    /// A check that is sound in itself: of the 4 at [`FREE`], after one move.
    const SOUND: Asked = Asked {
        after_move: 1,
        target: FREE,
        digit: 4,
    };

    fn refused_move(index: usize, source: PlayError) -> ReopenError {
        ReopenError::Move { index, source }
    }

    fn refused_check(index: usize, source: PlayError) -> ReopenError {
        ReopenError::Check { index, source }
    }

    #[test]
    fn a_record_built_by_hand_reopens() {
        let board = Board::reopen(&record(&[PLACED], 0, &[SOUND])).unwrap();
        assert_eq!(board.cell(FREE).unwrap().digit(), Some(4));
        let check = board.checks().next().unwrap();
        assert_eq!((check.index(), check.after_move()), (1, 1));
        assert!(check.is_right());
    }

    #[test]
    fn givens_with_no_solution_are_refused() {
        let fives = [1, 2].map(|column| Given::new(Position::new(1, column), 5));
        let source = SolveError::NoSolution;
        assert_eq!(
            refusal(&record_of(fives.into())),
            ReopenError::Givens { source }
        );
    }

    #[test]
    fn givens_with_several_solutions_are_refused() {
        let source = SolveError::ManySolutions;
        assert_eq!(
            refusal(&record_of(BTreeSet::new())),
            ReopenError::Givens { source }
        );
    }

    #[test]
    fn givens_that_leave_nothing_to_play_are_refused() {
        let source = SolveError::NotPosed(WellPosedError::NothingLeftToPlay { givens: 81 });
        let full = record_of(givens(&solution()));
        assert_eq!(refusal(&full), ReopenError::Givens { source });
    }

    #[test]
    fn a_move_on_a_given_is_refused() {
        let on_a_given = Written::Place {
            target: GIVEN,
            digit: 4,
        };
        let source = PlayError::GivenCell { position: GIVEN };
        assert_eq!(
            refusal(&record(&[PLACED, on_a_given], 0, &[])),
            refused_move(2, source)
        );
    }

    #[test]
    fn a_move_off_the_grid_is_refused() {
        let nowhere = Written::Erase {
            target: OFF_THE_GRID,
        };
        let source = PlayError::NoSuchCell {
            position: OFF_THE_GRID,
        };
        assert_eq!(
            refusal(&record(&[nowhere], 0, &[])),
            refused_move(1, source)
        );
    }

    #[test]
    fn a_move_with_a_digit_out_of_range_is_refused() {
        let placed = Written::Place {
            target: FREE,
            digit: 10,
        };
        let marked = Written::WriteMark {
            target: FREE,
            digit: 0,
        };
        let source = |digit| PlayError::DigitOutOfRange { digit };
        assert_eq!(
            refusal(&record(&[placed], 0, &[])),
            refused_move(1, source(10))
        );
        assert_eq!(
            refusal(&record(&[marked], 0, &[])),
            refused_move(1, source(0))
        );
    }

    #[test]
    fn a_placement_of_the_standing_digit_is_refused() {
        let source = PlayError::DigitAlreadyStands {
            position: FREE,
            digit: 4,
        };
        assert_eq!(
            refusal(&record(&[PLACED, PLACED], 0, &[])),
            refused_move(2, source)
        );
    }

    #[test]
    fn an_erasure_of_an_empty_cell_is_refused() {
        let erased = Written::Erase { target: FREE };
        let source = PlayError::NoPlayersDigit { position: FREE };
        assert_eq!(
            refusal(&record(&[erased], 0, &[])),
            refused_move(1, source.clone())
        );
        // The digit goes with the first erasure, so the second has nothing to erase.
        assert_eq!(
            refusal(&record(&[PLACED, erased, erased], 0, &[])),
            refused_move(3, source)
        );
    }

    #[test]
    fn a_mark_written_where_a_digit_stands_is_refused() {
        let marked = Written::WriteMark {
            target: FREE,
            digit: 2,
        };
        let source = PlayError::MarksNotAccepted { position: FREE };
        assert_eq!(
            refusal(&record(&[PLACED, marked], 0, &[])),
            refused_move(2, source)
        );
    }

    #[test]
    fn a_mark_struck_that_is_not_there_is_refused() {
        let struck = Written::StrikeMark {
            target: BESIDE,
            digit: 4,
        };
        let written = Written::WriteMark {
            target: BESIDE,
            digit: 4,
        };
        let source = PlayError::MarkNotThere {
            position: BESIDE,
            digit: 4,
        };
        assert_eq!(
            refusal(&record(&[struck], 0, &[])),
            refused_move(1, source.clone())
        );
        // Upkeep strikes the 4 beside the placement, so it is not there to strike.
        assert_eq!(
            refusal(&record(&[written, PLACED, struck], 0, &[])),
            refused_move(3, source)
        );
    }

    #[test]
    fn a_move_after_the_puzzle_is_solved_is_refused() {
        let after = [
            Written::Erase { target: FREE },
            Written::WriteMark {
                target: OFF_THE_GRID,
                digit: 0,
            },
        ];
        for last in after {
            let mut moves = solving();
            assert_eq!(moves.len(), 51);
            assert!(Board::reopen(&record(&moves, 0, &[])).is_ok());
            moves.push(last);
            assert_eq!(
                refusal(&record(&moves, 0, &[])),
                ReopenError::MoveAfterSolved { index: 52 }
            );
        }
    }

    #[test]
    fn more_undone_moves_than_moves_are_refused() {
        let cases = [(0, 1), (1, 2), (1, usize::MAX)];
        for (moves, undone) in cases {
            let made = [PLACED];
            assert_eq!(
                refusal(&record(&made[..moves], undone, &[])),
                ReopenError::TooManyUndone { undone, moves }
            );
        }
        assert!(Board::reopen(&record(&[PLACED], 1, &[])).is_ok());
    }

    #[test]
    fn undone_moves_on_a_solved_puzzle_are_refused() {
        for undone in [1, 51] {
            assert_eq!(
                refusal(&record(&solving(), undone, &[])),
                ReopenError::UndoneWhenSolved { undone }
            );
        }
    }

    #[test]
    fn a_check_on_a_given_is_refused() {
        let on_a_given = Asked {
            target: GIVEN,
            digit: 5,
            ..SOUND
        };
        let source = PlayError::GivenCell { position: GIVEN };
        assert_eq!(
            refusal(&record(&[PLACED], 0, &[SOUND, on_a_given])),
            refused_check(2, source)
        );
    }

    #[test]
    fn a_check_off_the_grid_is_refused() {
        let nowhere = Asked {
            target: OFF_THE_GRID,
            ..SOUND
        };
        let source = PlayError::NoSuchCell {
            position: OFF_THE_GRID,
        };
        assert_eq!(
            refusal(&record(&[PLACED], 0, &[nowhere])),
            refused_check(1, source)
        );
    }

    #[test]
    fn a_check_with_a_digit_out_of_range_is_refused() {
        for digit in [0, 10, 255] {
            let asked = Asked { digit, ..SOUND };
            let source = PlayError::DigitOutOfRange { digit };
            assert_eq!(
                refusal(&record(&[PLACED], 0, &[asked])),
                refused_check(1, source)
            );
        }
    }

    #[test]
    fn a_check_after_no_move_is_refused() {
        let asked = Asked {
            after_move: 0,
            ..SOUND
        };
        assert_eq!(
            refusal(&record(&[PLACED], 0, &[SOUND, SOUND, asked])),
            ReopenError::CheckAfterNoMove { index: 3 }
        );
    }

    #[test]
    fn checks_in_a_record_with_no_move_are_refused() {
        assert_eq!(
            refusal(&record(&[], 0, &[SOUND, SOUND])),
            ReopenError::ChecksWithoutMoves { checks: 2 }
        );
    }

    /// The fixture with every cell given but [`FREE`]: one cell is left to play, and
    /// placing its digit, a 4, solves the puzzle.
    fn one_cell_to_play(moves: &[Written], checks: &[Asked]) -> Record {
        let mut givens = givens(&solution());
        givens.remove(&Given::new(FREE, 4));
        Record {
            givens,
            moves: moves.to_vec(),
            undone: 0,
            checks: checks.to_vec(),
        }
    }

    /// Where one cell is left to play, its right digit solves the puzzle as it is
    /// placed, and nothing is checked on a solved puzzle. So no board answered a check
    /// of that digit, whatever moves the record holds.
    #[test]
    fn a_check_of_the_digit_that_solves_the_puzzle_is_refused() {
        let marked = Written::WriteMark {
            target: FREE,
            digit: 2,
        };
        let source = PlayError::Solved;
        for made in [PLACED, marked] {
            assert_eq!(
                refusal(&one_cell_to_play(&[made], &[SOUND])),
                refused_check(1, source.clone())
            );
        }

        // It is the last thing a check is held to: one asked after no move says so.
        let after_none = Asked {
            after_move: 0,
            ..SOUND
        };
        assert_eq!(
            refusal(&one_cell_to_play(&[PLACED], &[after_none])),
            ReopenError::CheckAfterNoMove { index: 1 }
        );

        // A wrong digit there solves nothing, so its check was asked and reopens; and
        // with two cells to play the right digit may be checked, as everywhere above.
        let wrong = Written::Place {
            target: FREE,
            digit: 9,
        };
        let asked = Asked { digit: 9, ..SOUND };
        let board = Board::reopen(&one_cell_to_play(&[wrong, PLACED], &[asked])).unwrap();
        let answers: Vec<bool> = board.checks().map(Check::is_right).collect();
        assert_eq!(answers, [false]);
        assert!(Board::reopen(&one_cell_to_play(&[PLACED], &[])).is_ok());
    }

    /// What cannot be read back is accepted: a check after more moves than the record
    /// holds, of a digit no move of the record placed. Its answer is worked out from
    /// the solution, so the record cannot lie about it.
    #[test]
    fn a_check_that_cannot_be_read_back_is_accepted() {
        let marked = Written::WriteMark {
            target: BESIDE,
            digit: 1,
        };
        let wrong = Asked {
            after_move: usize::MAX,
            target: FREE,
            digit: 9,
        };
        let right = Asked { digit: 4, ..wrong };
        let board = Board::reopen(&record(&[marked], 1, &[wrong, right])).unwrap();
        let checks: Vec<_> = board.checks().collect();
        let read = |at: usize| (checks[at].index(), checks[at].after_move());
        assert_eq!((read(0), read(1)), ((1, usize::MAX), (2, usize::MAX)));
        let answers = checks.iter().map(|check| check.is_right());
        assert_eq!(answers.collect::<Vec<_>>(), [false, true]);
    }

    /// A record wrong in every way reports the first, in the order `Board::reopen`
    /// documents: the two counts, the givens, the moves, the undone moves of a solved
    /// puzzle, and the checks in their own order.
    #[test]
    fn refusals_come_in_a_fixed_order() {
        let bad_check = Asked {
            after_move: 0,
            target: GIVEN,
            digit: 0,
        };
        let mut wrong = Record {
            givens: BTreeSet::new(),
            moves: Vec::new(),
            undone: 1,
            checks: [bad_check].into(),
        };
        let (undone, moves) = (1, 0);
        assert_eq!(
            refusal(&wrong),
            ReopenError::TooManyUndone { undone, moves }
        );
        wrong.undone = 0;
        assert_eq!(
            refusal(&wrong),
            ReopenError::ChecksWithoutMoves { checks: 1 }
        );
        wrong.moves = [PLACED, PLACED].into();
        let source = SolveError::ManySolutions;
        assert_eq!(refusal(&wrong), ReopenError::Givens { source });
        wrong.givens = givens(FIXTURE);
        assert!(matches!(
            refusal(&wrong),
            ReopenError::Move { index: 2, .. }
        ));

        wrong.moves = solving();
        wrong.undone = 2;
        assert_eq!(refusal(&wrong), ReopenError::UndoneWhenSolved { undone: 2 });
        wrong.undone = 0;
        let source = PlayError::GivenCell { position: GIVEN };
        assert_eq!(refusal(&wrong), refused_check(1, source));
        wrong.checks[0].target = FREE;
        let source = PlayError::DigitOutOfRange { digit: 0 };
        assert_eq!(refusal(&wrong), refused_check(1, source));
        wrong.checks[0].digit = 4;
        assert_eq!(refusal(&wrong), ReopenError::CheckAfterNoMove { index: 1 });
        wrong.checks[0].after_move = 1;
        assert!(Board::reopen(&wrong).is_ok());
    }

    #[test]
    fn a_refused_reopening_leaves_the_record_as_it_was() {
        let on_a_given = Written::Place {
            target: GIVEN,
            digit: 4,
        };
        let wrong = record(&[PLACED, on_a_given], 1, &[SOUND]);
        let before = wrong.clone();
        assert!(Board::reopen(&wrong).is_err());
        assert_eq!(wrong, before);
        assert_eq!(refusal(&wrong), refusal(&before));
    }

    #[test]
    fn reopening_refusal_texts_are_stable() {
        let source = PlayError::GivenCell { position: GIVEN };
        let texts = [
            (
                ReopenError::TooManyUndone {
                    undone: 3,
                    moves: 2,
                },
                "the record's undone count, 3, exceeds its move count, 2",
            ),
            (
                ReopenError::ChecksWithoutMoves { checks: 2 },
                "the record holds no move, and its check count is 2",
            ),
            (
                ReopenError::Givens {
                    source: SolveError::ManySolutions,
                },
                "the record's givens open no board: the givens have more than one solution",
            ),
            (
                refused_move(7, source.clone()),
                "move 7 of the record does not read back: \
                the cell at row 1, column 1 holds a given",
            ),
            (
                ReopenError::MoveAfterSolved { index: 52 },
                "move 52 of the record comes after the puzzle is solved",
            ),
            (
                ReopenError::UndoneWhenSolved { undone: 1 },
                "the record's moves solve the puzzle, and its undone count is 1",
            ),
            (
                refused_check(2, source),
                "check 2 of the record does not read back: \
                the cell at row 1, column 1 holds a given",
            ),
            (
                ReopenError::CheckAfterNoMove { index: 4 },
                "check 4 of the record was asked after no move",
            ),
        ];
        for (error, text) in texts {
            assert_eq!(error.to_string(), text);
        }
    }

    /// The refusals that carry another say so through `source`, for a caller that
    /// walks the chain.
    #[test]
    fn a_refusal_names_its_source() {
        use core::error::Error;

        let play = PlayError::GivenCell { position: GIVEN };
        let carried = [
            ReopenError::Givens {
                source: SolveError::NoSolution,
            },
            refused_move(1, play.clone()),
            refused_check(1, play),
        ];
        assert!(carried.iter().all(|error| error.source().is_some()));
        let alone = ReopenError::MoveAfterSolved { index: 1 };
        assert!(alone.source().is_none());
    }

    /// Each row of the solution as a `Debug` might print it with the white space taken
    /// out: as a string of digits, and as a list of numbers.
    fn rows_as_printed() -> Vec<String> {
        let solution = solution();
        let rows = solution.as_bytes().chunks(9);
        let digits = rows.map(|row| row.iter().map(|digit| char::from(*digit).to_string()));
        let digits = digits.map(Iterator::collect::<Vec<String>>);
        digits
            .flat_map(|row| [row.concat(), row.join(",")])
            .collect()
    }

    /// A board with a move of every kind on it, one of them undone, and a check
    /// answered each way.
    fn played() -> Board {
        let mut board = Board::open(givens(FIXTURE)).unwrap();
        board.write_mark(BESIDE, 7).unwrap();
        board.strike_mark(BESIDE, 7).unwrap();
        board.place(FREE, 9).unwrap();
        assert!(!board.check(FREE).unwrap().is_right());
        board.place(FREE, 4).unwrap();
        assert!(board.check(FREE).unwrap().is_right());
        board.erase(FREE).unwrap();
        board.undo().unwrap();
        board
    }

    /// Every field a record has, at any depth, by the names its `Debug` prints them
    /// under. `Debug` and the serde traits are derived from the same fields, and no
    /// attribute skips or renames one, so these are the fields a record serialises.
    fn fields(record: &Record) -> BTreeSet<String> {
        let printed = format!("{record:#?}");
        let words = printed.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'));
        let names = words.filter_map(|word| word.strip_suffix(':'));
        names.map(String::from).collect()
    }

    /// A record holds the givens, what the player did and what the player asked, and
    /// nothing derived or read from the solution: its fields are the ones stated and
    /// no others, a check's answer is not among them, and it has no boolean to hold
    /// one. The board it was written from holds a check answered each way.
    #[test]
    fn a_record_holds_what_it_states_and_no_answer() {
        let record = played().write();
        let stated = [
            "givens",
            "position",
            "row",
            "column",
            "digit",
            "moves",
            "target",
            "undone",
            "checks",
            "after_move",
        ];
        assert_eq!(fields(&record), stated.map(String::from).into());
        let printed = format!("{record:?}");
        assert!(!printed.contains("true") && !printed.contains("false"));
        // The record looked at is not an empty one: both checks and every move are in it.
        assert_eq!(record.checks.len(), 2);
        assert_eq!(record.moves.len(), 5);
        assert_eq!(record.undone, 1);
    }

    /// A narrower look at the same thing: no row of the solution is in what a record
    /// prints, as digits or as a list.
    #[test]
    fn a_record_prints_no_row_of_the_solution() {
        let board = Board::open(givens(FIXTURE)).unwrap();
        let printed = [board.write(), played().write()]
            .map(|record| [format!("{record:?}"), format!("{record:#?}")]);
        for text in printed.as_flattened() {
            let text: String = text.split_whitespace().collect();
            let shown = rows_as_printed().into_iter().find(|row| text.contains(row));
            assert_eq!(shown, None);
        }
    }

    /// The four fields of a serialised record, handed to a deserialiser one at a time
    /// as the elements of a sequence: no givens, no moves, one move undone, no checks.
    /// It counts the fields handed over.
    #[cfg(feature = "serde")]
    struct Fields(u8);

    #[cfg(feature = "serde")]
    impl<'de> serde::de::SeqAccess<'de> for Fields {
        type Error = serde::de::value::Error;

        fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(
            &mut self,
            seed: T,
        ) -> Result<Option<T::Value>, Self::Error> {
            use serde::de::IntoDeserializer;
            use serde::de::value::SeqDeserializer;

            // The derived visitor asks for a struct's four fields and no more: the
            // third is the undone count, and the others are sequences, here empty.
            self.0 += 1;
            if self.0 == 3 {
                return seed.deserialize(1_usize.into_deserializer()).map(Some);
            }
            let nothing = SeqDeserializer::new(core::iter::empty::<u8>());
            seed.deserialize(nothing).map(Some)
        }
    }

    /// Deserialising checks nothing, and reopening checks everything: a record no
    /// board wrote arrives through serde's own value deserialisers, with no format
    /// crate, as it could from any caller, and is refused when it is reopened.
    #[cfg(feature = "serde")]
    #[test]
    fn a_malformed_record_deserialises_and_is_refused_at_reopening() {
        use serde::Deserialize;
        use serde::de::value::SeqAccessDeserializer;

        let arrived = Record::deserialize(SeqAccessDeserializer::new(Fields(0))).unwrap();
        let (undone, moves) = (1, 0);
        assert_eq!(arrived.undone, undone);
        assert_eq!(
            refusal(&arrived),
            ReopenError::TooManyUndone { undone, moves }
        );
    }
}
