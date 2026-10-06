// The engine map: a guide page of the API reference, and nothing but documentation.
// `lib.rs` declares this module under `cfg(doc)`, so no consumer can name it. The
// figures are inline SVG styled by `crates/pawdoku/rustdoc/header.html`; each sits in
// one HTML block, which a blank line would end, with every tag closed.

//! A map of the engine: what this crate is, what it does and how, drawn at four levels.
//!
//! `pawdoku` is the classic-sudoku engine behind Pawdoku. It is a library. It knows the
//! rules, proves that a puzzle has exactly one solution, makes puzzles from a stream of
//! draws, and keeps one game in play, with notes, undo and a check. It draws nothing and
//! stores nothing. The program that calls it does both.
//!
//! The page goes from far to near. Level 0 is the engine seen from outside. Level 1 is its
//! modules. Level 2 opens each module that is built. Level 3 follows three calls through
//! the code. Five modules are built and five are specified and not built yet.
//!
//! ## How to read the figures
//!
//! <div class="pg-fig pg-key">
//! <svg class="pg" viewBox="0 0 960 30" role="img" aria-label="Key. A solid blue box is built. A dashed box is specified and not built yet. An amber box is a solution stored in a puzzle or a board, which stays hidden. A red box is a refusal.">
//! <rect class="pg-built" x="2" y="8" width="22" height="14" rx="2"/>
//! <text class="pg-n" x="32" y="20">built</text>
//! <rect class="pg-plan" x="92" y="8" width="22" height="14" rx="2"/>
//! <text class="pg-n" x="122" y="20">specified, not built yet</text>
//! <rect class="pg-sol" x="286" y="8" width="22" height="14" rx="2"/>
//! <text class="pg-n" x="316" y="20">a solution stored in a puzzle or a board, hidden</text>
//! <rect class="pg-ref" x="610" y="8" width="22" height="14" rx="2"/>
//! <text class="pg-n" x="640" y="20">a refusal</text>
//! </svg>
//! </div>
//!
//! Only a solution stored inside a [`Puzzle`] or a [`Board`] is hidden. The solver's
//! [`SearchResult`] and the [`WellPosed`] proof hand whole solution grids to whoever holds
//! them.
//!
//! ## Words used here
//!
//! - **Given.** A digit the setter printed. It is never the player's to change.
//! - **Peer.** Another cell in the same row, column or 3×3 box. Two peers holding one
//!   digit are in conflict.
//! - **Well-posed.** Givens that have exactly one solution. A puzzle to play also needs at
//!   least one cell left for the player.
//! - **Note and mark.** A note is one cell's pencil marks. A mark is one digit in it.
//! - **Move.** Placing a digit, erasing one, writing a mark or striking one. A check is
//!   not a move.
//! - **Standing and undone.** A standing move is in effect. An undone move was taken back,
//!   is still on the list, and can be re-taken.
//! - **Refusal.** The engine's "no", returned as a value that names the rule that failed.
//!   A refused operation on a board changes nothing.
//! - **Record.** One game written down as a plain value that the caller keeps.
//! - **Tier.** One of five settings for how a puzzle is made. It says nothing about how
//!   hard the puzzle is.
//!
//! ## Level 0: the whole
//!
//! ### Where the engine sits
//!
//! The engine has no program of its own. Nothing happens until a consumer calls it, and
//! every call takes plain values and returns plain values. Three consumers are planned as
//! sibling crates. None exists yet, so today the engine is reached from Rust and from its
//! own tests.
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 388" role="img" aria-label="Three planned consumers call the pawdoku engine with plain values and get plain values back. The engine holds five built modules and has no screen, storage, clock, threads or network.">
//! <defs><marker id="pg-m1" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker></defs>
//! <text class="pg-lab" x="24" y="27">CONSUMERS · CRATES STILL TO COME</text>
//! <rect class="pg-plan" x="24" y="40" width="236" height="64" rx="3"/>
//! <text class="pg-h" x="40" y="67">The game</text>
//! <text class="pg-c pg-d" x="40" y="88">pawdoku-wasm · WebAssembly</text>
//! <rect class="pg-plan" x="24" y="124" width="236" height="64" rx="3"/>
//! <text class="pg-h" x="40" y="151">Python</text>
//! <text class="pg-c pg-d" x="40" y="172">pawdoku-py · bindings</text>
//! <rect class="pg-plan" x="24" y="208" width="236" height="64" rx="3"/>
//! <text class="pg-h" x="40" y="235">A command line</text>
//! <text class="pg-c pg-d" x="40" y="256">pawdoku-cli</text>
//! <path class="pg-ln" d="M260 72H276M260 156H276M260 240H276M276 72V240"/>
//! <line class="pg-ln" x1="276" y1="128" x2="597" y2="128" marker-end="url(#pg-m1)"/>
//! <text class="pg-n" x="437" y="118" text-anchor="middle">in: givens, positions, digits, a tier, a seed</text>
//! <line class="pg-ln" x1="600" y1="196" x2="279" y2="196" marker-end="url(#pg-m1)"/>
//! <text class="pg-n" x="437" y="186" text-anchor="middle">out: puzzles, cells, moves, checks, records, refusals</text>
//! <rect class="pg-frame" x="600" y="40" width="336" height="270" rx="3"/>
//! <text class="pg-ch" x="618" y="67">pawdoku</text>
//! <text class="pg-d" x="618" y="87">a library with no process of its own</text>
//! <rect class="pg-built" x="618" y="104" width="146" height="48" rx="3"/>
//! <text class="pg-ch" x="630" y="124">sudoku</text><text class="pg-d" x="630" y="141">the rules</text>
//! <rect class="pg-built" x="774" y="104" width="146" height="48" rx="3"/>
//! <text class="pg-ch" x="786" y="124">solver</text><text class="pg-d" x="786" y="141">the search</text>
//! <rect class="pg-built" x="618" y="162" width="146" height="48" rx="3"/>
//! <text class="pg-ch" x="630" y="182">board</text><text class="pg-d" x="630" y="199">one game in play</text>
//! <rect class="pg-built" x="774" y="162" width="146" height="48" rx="3"/>
//! <text class="pg-ch" x="786" y="182">random</text><text class="pg-d" x="786" y="199">the one effect</text>
//! <rect class="pg-built" x="618" y="220" width="302" height="48" rx="3"/>
//! <text class="pg-ch" x="630" y="240">generation</text><text class="pg-d" x="630" y="257">puzzles made from a tier and a stream of draws</text>
//! <text class="pg-d" x="618" y="294">It runs only when a consumer calls it.</text>
//! <text class="pg-lab" x="24" y="346">LEFT OUT BY DESIGN</text>
//! <text x="24" y="368">no screen · no storage · no clock · no threads · no network · no randomness of its own</text>
//! </svg>
//! </div>
//!
//! *The engine is called with plain values and answers with plain values. What a player
//! sees, and where a game is kept, is the consumer's business.*
//!
//! ### One game, end to end
//!
//! A game starts from givens, which a setter supplies or [`generate`] makes.
//! [`Board::open`] refuses givens that do not have exactly one solution. Play goes through
//! the board until the grid is full with no conflict, and solved is final. At any point the
//! board can be written down as a [`Record`], and a record written by this version of the
//! engine reopens in it as the same board. A record carries no version, and reopening one
//! in another version of the engine is not promised yet.
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 310" role="img" aria-label="Givens, supplied or made by generate, open a board. The board is played through seven operations. It is written to a record that the consumer keeps, and the record reopens as the same board.">
//! <defs>
//! <marker id="pg-m2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker>
//! <marker id="pg-m2m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah-m" d="M0 0L10 5L0 10z"/></marker>
//! </defs>
//! <rect class="pg-bx" x="24" y="110" width="140" height="80" rx="3"/>
//! <text class="pg-h" x="40" y="138">Givens</text>
//! <text class="pg-d" x="40" y="158">the clues a puzzle</text>
//! <text class="pg-d" x="40" y="174">starts with</text>
//! <rect class="pg-built" x="24" y="222" width="150" height="66" rx="3"/>
//! <text class="pg-ch" x="38" y="246">generate</text>
//! <text class="pg-d" x="38" y="264">a puzzle from a tier</text>
//! <text class="pg-d" x="38" y="280">and a stream of draws</text>
//! <line class="pg-ln" x1="94" y1="222" x2="94" y2="193" marker-end="url(#pg-m2)"/>
//! <text class="pg-d" x="104" y="211">or made here</text>
//! <line class="pg-ln" x1="164" y1="150" x2="337" y2="150" marker-end="url(#pg-m2)"/>
//! <text class="pg-c" x="251" y="140" text-anchor="middle">Board::open</text>
//! <text class="pg-d" x="251" y="170" text-anchor="middle">refused unless the givens</text>
//! <text class="pg-d" x="251" y="186" text-anchor="middle">have exactly one solution</text>
//! <rect class="pg-built" x="340" y="110" width="250" height="80" rx="3"/>
//! <text class="pg-ch" x="356" y="138">Board</text>
//! <text class="pg-d" x="356" y="158">one puzzle in play</text>
//! <text class="pg-d" x="356" y="174">Unsolved, then Solved for good</text>
//! <path class="pg-ln" d="M400 110V64H530V107" marker-end="url(#pg-m2)"/>
//! <text class="pg-c" x="465" y="36" text-anchor="middle">place · erase · write_mark · strike_mark</text>
//! <text class="pg-c" x="465" y="53" text-anchor="middle">undo · redo · check</text>
//! <line class="pg-ln" x1="590" y1="132" x2="757" y2="132" marker-end="url(#pg-m2)"/>
//! <text class="pg-c" x="674" y="122" text-anchor="middle">Board::write</text>
//! <line class="pg-ln" x1="760" y1="170" x2="593" y2="170" marker-end="url(#pg-m2)"/>
//! <text class="pg-c" x="674" y="188" text-anchor="middle">Board::reopen</text>
//! <text class="pg-d" x="674" y="204" text-anchor="middle">the same board again</text>
//! <rect class="pg-bx" x="760" y="110" width="176" height="80" rx="3"/>
//! <text class="pg-ch" x="776" y="138">Record</text>
//! <text class="pg-d" x="776" y="158">the game, written down</text>
//! <text class="pg-d" x="776" y="174">no solution inside</text>
//! <line class="pg-ln-m" x1="848" y1="194" x2="848" y2="232" marker-start="url(#pg-m2m)" marker-end="url(#pg-m2m)"/>
//! <rect class="pg-plan" x="760" y="236" width="176" height="52" rx="3"/>
//! <text class="pg-d" x="776" y="258">kept by the consumer,</text>
//! <text class="pg-d" x="776" y="274">wherever it likes</text>
//! </svg>
//! </div>
//!
//! *The board is the one thing that changes as a game goes on. A record is the only form in
//! which a game in play leaves the engine.*
//!
//! The same game as code:
//!
//! ```rust
//! use pawdoku::board::Board;
//! use pawdoku::sudoku::{Given, Position, Status};
//!
//! // Thirty givens, rows top to bottom, a dot for an empty cell.
//! let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
//!     .bytes()
//!     .zip(0_u8..)
//!     .filter(|(cell, _)| cell.is_ascii_digit())
//!     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
//!
//! let mut board = Board::open(givens)?;
//! let here = Position::new(1, 3);
//!
//! // A move, a check, and the move taken back and re-taken. The check stays.
//! board.place(here, 4)?;
//! assert!(board.check(here)?.is_right());
//! board.undo()?;
//! assert!(board.can_redo());
//! board.redo()?;
//!
//! // Written down, and reopened as the same board.
//! let record = board.write();
//! let reopened = Board::reopen(&record)?;
//! assert_eq!(reopened.cells().collect::<Vec<_>>(), board.cells().collect::<Vec<_>>());
//! assert_eq!(reopened.checks().count(), 1);
//! assert_eq!(reopened.status(), Status::Unsolved);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Level 1: the modules
//!
//! ### Ten modules and who may import whom
//!
//! The map has nine specification modules and the randomness boundary. Each specification
//! module is one file under `docs/specs/` and, once built, one Rust module of the same
//! name. `random` has no specification file: it is the boundary the engine takes its draws
//! through, and it imports none of the others. A module may use only the modules its arrows
//! point to.
//!
//! A quality gate checks imports written as `crate::` paths against that rule. A relative
//! path, an alias of the crate and a re-export are outside its sight, and review has to
//! catch those. The handbook's [layering page] owns the rule and says how it is enforced.
//!
//! The direction has a purpose. The four models of a player share only the rules and the
//! technique catalogue, so each one can be tested without the other three and without a
//! puzzle generator.
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 566" role="img" aria-label="The module map. sudoku sits beneath everything. solver and technique stand on it. board stands on sudoku and solver. Four player models stand on sudoku and technique. generation stands on sudoku, solver, technique and reach. random stands beside them all. sudoku, solver, board, generation and random are built; the rest are specified only. generation draws through random.">
//! <defs><marker id="pg-m3" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker></defs>
//! <rect class="pg-built" x="150" y="36" width="250" height="80" rx="3"/>
//! <text class="pg-ch" x="166" y="62">generation</text>
//! <text class="pg-lab" x="388" y="61" text-anchor="end">BUILT · BASIC WAY</text>
//! <text class="pg-d" x="166" y="84">a program that sets puzzles:</text>
//! <text class="pg-d" x="166" y="100">draw a full grid, remove givens</text>
//! <rect class="pg-built" x="24" y="166" width="166" height="100" rx="3"/>
//! <text class="pg-ch" x="38" y="192">board</text>
//! <text class="pg-lab" x="178" y="191" text-anchor="end">BUILT</text>
//! <text class="pg-d" x="38" y="214">one puzzle in play:</text>
//! <text class="pg-d" x="38" y="230">notes, moves, undo,</text>
//! <text class="pg-d" x="38" y="246">check, record</text>
//! <text class="pg-lab" x="529" y="154" text-anchor="middle">FOUR MODELS OF A PLAYER · NONE IMPORTS ANOTHER</text>
//! <rect class="pg-plan" x="270" y="166" width="122" height="100" rx="3"/>
//! <text class="pg-cs" x="280" y="191">reach</text>
//! <text class="pg-s" x="280" y="212">how far a player</text>
//! <text class="pg-s" x="280" y="227">gets; the hint</text>
//! <text class="pg-s" x="280" y="242">they'd find next</text>
//! <rect class="pg-plan" x="402" y="166" width="122" height="100" rx="3"/>
//! <text class="pg-cs" x="412" y="191">effort</text>
//! <text class="pg-s" x="412" y="212">what a puzzle</text>
//! <text class="pg-s" x="412" y="227">costs a player,</text>
//! <text class="pg-s" x="412" y="242">step by step</text>
//! <rect class="pg-plan" x="534" y="166" width="122" height="100" rx="3"/>
//! <text class="pg-cs" x="544" y="191">lapse</text>
//! <text class="pg-s" x="544" y="212">marks fall behind,</text>
//! <text class="pg-s" x="544" y="227">guesses go wrong,</text>
//! <text class="pg-s" x="544" y="242">get put right</text>
//! <rect class="pg-plan" x="666" y="166" width="122" height="100" rx="3"/>
//! <text class="pg-cs" x="676" y="191">human-solving</text>
//! <text class="pg-s" x="676" y="212">a fallible attempt</text>
//! <text class="pg-s" x="676" y="227">shaped by chance,</text>
//! <text class="pg-s" x="676" y="242">over seeded tries</text>
//! <rect class="pg-built" x="24" y="316" width="198" height="80" rx="3"/>
//! <text class="pg-ch" x="38" y="342">solver</text>
//! <text class="pg-lab" x="210" y="341" text-anchor="end">BUILT</text>
//! <text class="pg-d" x="38" y="364">counts the solutions:</text>
//! <text class="pg-d" x="38" y="380">none, one or many</text>
//! <rect class="pg-plan" x="240" y="316" width="548" height="80" rx="3"/>
//! <text class="pg-ch" x="272" y="342">technique</text>
//! <text class="pg-lab" x="776" y="341" text-anchor="end">PLANNED</text>
//! <text class="pg-d" x="272" y="364">the 29 named techniques a person solves with, and the profile of a player;</text>
//! <text class="pg-d" x="272" y="380">the ground all four models stand on</text>
//! <rect class="pg-built" x="24" y="446" width="764" height="70" rx="3"/>
//! <text class="pg-ch" x="38" y="472">sudoku</text>
//! <text class="pg-lab" x="776" y="471" text-anchor="end">BUILT</text>
//! <text class="pg-d" x="38" y="496">the rules: the grid, the givens, conflicts, and when a puzzle is solved. Every specification module above imports it.</text>
//! <rect class="pg-built" x="814" y="36" width="122" height="480" rx="3"/>
//! <text class="pg-ch" x="826" y="62">random</text>
//! <text class="pg-lab" x="826" y="80">BUILT</text>
//! <text class="pg-s" x="826" y="106">the one effect:</text>
//! <text class="pg-s" x="826" y="121">a seeded stream</text>
//! <text class="pg-s" x="826" y="136">of draws that</text>
//! <text class="pg-s" x="826" y="151">the caller</text>
//! <text class="pg-s" x="826" y="166">supplies</text>
//! <text class="pg-s" x="826" y="200">It stands beside</text>
//! <text class="pg-s" x="826" y="215">the others, not</text>
//! <text class="pg-s" x="826" y="230">beneath them.</text>
//! <text class="pg-s" x="826" y="245">Any module that</text>
//! <text class="pg-s" x="826" y="260">draws may use it.</text>
//! <text class="pg-s" x="826" y="294">generation draws</text>
//! <text class="pg-s" x="826" y="309">from it today.</text>
//! <line class="pg-ln" x1="331" y1="116" x2="331" y2="163" marker-end="url(#pg-m3)"/>
//! <line class="pg-ln" x1="208" y1="116" x2="208" y2="313" marker-end="url(#pg-m3)"/>
//! <line class="pg-ln" x1="254" y1="116" x2="254" y2="313" marker-end="url(#pg-m3)"/>
//! <line class="pg-ln" x1="107" y1="266" x2="107" y2="313" marker-end="url(#pg-m3)"/>
//! <line class="pg-ln" x1="331" y1="266" x2="331" y2="313" marker-end="url(#pg-m3)"/>
//! <line class="pg-ln" x1="463" y1="266" x2="463" y2="313" marker-end="url(#pg-m3)"/>
//! <line class="pg-ln" x1="595" y1="266" x2="595" y2="313" marker-end="url(#pg-m3)"/>
//! <line class="pg-ln" x1="727" y1="266" x2="727" y2="313" marker-end="url(#pg-m3)"/>
//! <line class="pg-ln" x1="123" y1="396" x2="123" y2="443" marker-end="url(#pg-m3)"/>
//! <line class="pg-ln" x1="514" y1="396" x2="514" y2="443" marker-end="url(#pg-m3)"/>
//! <line class="pg-ln" x1="400" y1="76" x2="811" y2="76" marker-end="url(#pg-m3)"/>
//! <text class="pg-s" x="606" y="68" text-anchor="middle">draws through</text>
//! <rect class="pg-built" x="24" y="537" width="18" height="12" rx="2"/>
//! <text class="pg-s" x="48" y="547">built</text>
//! <rect class="pg-plan" x="92" y="537" width="18" height="12" rx="2"/>
//! <text class="pg-s" x="116" y="547">specified, not built yet</text>
//! <line class="pg-ln" x1="256" y1="543" x2="284" y2="543" marker-end="url(#pg-m3)"/>
//! <text class="pg-s" x="292" y="547">may import. Every specification module imports sudoku too; only the nearest of those arrows are drawn.</text>
//! </svg>
//! </div>
//!
//! *An arrow runs from a module to one it may import. Nothing imports `board` or
//! `generation`, and no player model imports another. The generator that is built today
//! uses the rules, the solver and the draws; `technique` and `reach` are what its
//! specification lets it stand on later.*
//!
//! ## Level 2: the components
//!
//! ### `sudoku`, the rules
//!
//! The rules hold one puzzle and its digits. A [`Puzzle`] can be made only from a
//! [`WellPosed`] proof that its givens have exactly one solution, so a puzzle with no
//! solution or with several cannot exist. The solver makes the proof. No consumer can make
//! one, and inside the crate the rule that only the solver does is held by review, not by
//! the compiler. The puzzle keeps the solution and never hands it out. Its small values
//! are [`Position`] (a row and a column, counted from 1), [`Given`] (a position and a
//! digit) and [`Cell`] (one cell as it stands).
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 400" role="img" aria-label="A Puzzle is set from a WellPosed proof and changed by two moves, place and erase. It holds givens, digits, a status and a hidden solution. It answers with copies of cells and a few facts, and refuses with one of five MoveError kinds.">
//! <defs>
//! <marker id="pg-m4" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker>
//! <marker id="pg-m4s" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah-s" d="M0 0L10 5L0 10z"/></marker>
//! </defs>
//! <rect class="pg-bx" x="24" y="50" width="236" height="96" rx="3"/>
//! <text class="pg-ch" x="40" y="76">WellPosed</text>
//! <text class="pg-d" x="40" y="96">the proof: the givens and</text>
//! <text class="pg-d" x="40" y="112">their one solution</text>
//! <text class="pg-d" x="40" y="132">made by the solver, not by callers</text>
//! <rect class="pg-bx" x="24" y="196" width="236" height="80" rx="3"/>
//! <text class="pg-h" x="40" y="222">The two moves</text>
//! <text class="pg-c" x="40" y="244">place(position, digit)</text>
//! <text class="pg-c" x="40" y="262">erase(position)</text>
//! <line class="pg-ln" x1="260" y1="98" x2="347" y2="98" marker-end="url(#pg-m4)"/>
//! <text class="pg-c" x="305" y="88" text-anchor="middle">Puzzle::set</text>
//! <line class="pg-ln" x1="260" y1="236" x2="347" y2="236" marker-end="url(#pg-m4)"/>
//! <rect class="pg-frame" x="350" y="36" width="290" height="260" rx="3"/>
//! <text x="366" y="64"><tspan class="pg-ch">Puzzle</tspan><tspan class="pg-d" dx="10">one puzzle and its digits</tspan></text>
//! <rect class="pg-bx" x="366" y="92" width="258" height="40" rx="3"/>
//! <text x="378" y="117"><tspan class="pg-h">givens</tspan><tspan class="pg-d" dx="10">the clues; they never change</tspan></text>
//! <rect class="pg-bx" x="366" y="140" width="258" height="40" rx="3"/>
//! <text x="378" y="165"><tspan class="pg-h">digits</tspan><tspan class="pg-d" dx="10">the grid as it stands</tspan></text>
//! <rect class="pg-bx" x="366" y="188" width="258" height="40" rx="3"/>
//! <text x="378" y="213"><tspan class="pg-h">status</tspan><tspan class="pg-d" dx="10">Unsolved, then Solved for good</tspan></text>
//! <rect class="pg-sol" x="366" y="236" width="258" height="40" rx="3"/>
//! <text x="378" y="261"><tspan class="pg-h pg-st">solution</tspan><tspan class="pg-d pg-st" dx="10">kept inside; never shown</tspan></text>
//! <line class="pg-ln" x1="640" y1="120" x2="721" y2="120" marker-end="url(#pg-m4)"/>
//! <rect class="pg-bx" x="724" y="50" width="212" height="140" rx="3"/>
//! <text class="pg-h" x="740" y="76">What it answers</text>
//! <text x="740" y="98"><tspan class="pg-c">Cell</tspan><tspan class="pg-d" dx="8">a copy of one cell</tspan></text>
//! <text class="pg-c" x="740" y="118">status()</text>
//! <text class="pg-c" x="740" y="136">is_full()</text>
//! <text class="pg-c" x="740" y="154">is_consistent()</text>
//! <text class="pg-c" x="740" y="172">givens()</text>
//! <line class="pg-ln-s" x1="640" y1="256" x2="721" y2="256" marker-end="url(#pg-m4s)"/>
//! <rect class="pg-sol" x="724" y="206" width="212" height="70" rx="3"/>
//! <text class="pg-lab pg-st" x="740" y="228">INSIDE THE CRATE ONLY</text>
//! <text class="pg-c pg-st" x="740" y="248">is_solution_digit</text>
//! <text class="pg-d pg-st" x="740" y="266">answers yes or no</text>
//! <rect class="pg-ref" x="24" y="320" width="912" height="60" rx="3"/>
//! <text class="pg-h pg-rt" x="40" y="345">Refuses with MoveError</text>
//! <text class="pg-c pg-rt" x="40" y="366">NoSuchCell · AlreadySolved · GivenCell · DigitOutOfRange · EmptyCell</text>
//! <text class="pg-d" x="920" y="345" text-anchor="end">A digit that conflicts with a peer is allowed and flagged, not refused.</text>
//! </svg>
//! </div>
//!
//! *The solution enters with the proof and stays inside the puzzle. Solved means every cell
//! is full and none conflicts. It is never a comparison with the stored solution.*
//!
//! ### `solver`, the search
//!
//! There is one search and two ways to ask it. [`search`] reports on any givens and never
//! fails. [`solve`] turns the same search into the proof a puzzle needs, or refuses. The
//! search takes no time limit, because it always finishes, and it uses no randomness, so
//! the same givens always give the same answer and the same count of guesses.
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 404" role="img" aria-label="Givens go into one search. The search takes the deepest waiting branch, propagates, then drops a contradicted branch, keeps a solution, or splits and counts a guess, and then takes the next waiting branch. It stops at two solutions or when no branch waits. search returns a SearchResult. solve returns a WellPosed proof or one of three refusals.">
//! <defs><marker id="pg-m5" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker></defs>
//! <rect class="pg-bx" x="24" y="150" width="150" height="96" rx="3"/>
//! <text class="pg-h" x="40" y="176">Givens</text>
//! <text class="pg-d" x="40" y="196">any set at all:</text>
//! <text class="pg-d" x="40" y="212">empty, conflicting,</text>
//! <text class="pg-d" x="40" y="228">off the grid</text>
//! <line class="pg-ln" x1="174" y1="198" x2="227" y2="198" marker-end="url(#pg-m5)"/>
//! <rect class="pg-frame" x="230" y="36" width="440" height="348" rx="3"/>
//! <text class="pg-h" x="246" y="62">One search, behind both entries</text>
//! <rect class="pg-bx" x="250" y="78" width="400" height="34" rx="3"/>
//! <text x="450" y="100" text-anchor="middle">take the deepest waiting branch</text>
//! <line class="pg-ln" x1="450" y1="112" x2="450" y2="127" marker-end="url(#pg-m5)"/>
//! <rect class="pg-bx" x="250" y="130" width="400" height="34" rx="3"/>
//! <text x="450" y="152" text-anchor="middle">propagate: three rules, until nothing changes</text>
//! <line class="pg-ln" x1="312" y1="164" x2="312" y2="195" marker-end="url(#pg-m5)"/>
//! <line class="pg-ln" x1="450" y1="164" x2="450" y2="195" marker-end="url(#pg-m5)"/>
//! <line class="pg-ln" x1="588" y1="164" x2="588" y2="195" marker-end="url(#pg-m5)"/>
//! <rect class="pg-ref" x="250" y="198" width="124" height="66" rx="3"/>
//! <text class="pg-h" x="312" y="220" text-anchor="middle">contradiction</text>
//! <text class="pg-d" x="312" y="238" text-anchor="middle">the branch</text>
//! <text class="pg-d" x="312" y="254" text-anchor="middle">is dropped</text>
//! <rect class="pg-sol" x="388" y="198" width="124" height="66" rx="3"/>
//! <text class="pg-h" x="450" y="220" text-anchor="middle">every cell placed</text>
//! <text class="pg-d" x="450" y="238" text-anchor="middle">a solution</text>
//! <text class="pg-d" x="450" y="254" text-anchor="middle">is kept</text>
//! <rect class="pg-bx" x="526" y="198" width="124" height="66" rx="3"/>
//! <text class="pg-h" x="588" y="220" text-anchor="middle">cells still open</text>
//! <text class="pg-d" x="588" y="238" text-anchor="middle">split: one child</text>
//! <text class="pg-d" x="588" y="254" text-anchor="middle">per candidate</text>
//! <path class="pg-ln" d="M312 264V280M450 264V280M588 264V280"/>
//! <path class="pg-ln" d="M312 280H661V95H653" marker-end="url(#pg-m5)"/>
//! <text class="pg-s" x="246" y="298">Then the next waiting branch. It stops at two solutions, or when none waits.</text>
//! <text class="pg-s" x="246" y="320">Propagation: a placed digit leaves its peers' candidates; a cell with</text>
//! <text class="pg-s" x="246" y="335">one candidate takes it; a digit with one place left in a unit goes there.</text>
//! <text class="pg-s" x="246" y="354">A split counts one guess. It is on the cell with the fewest candidates;</text>
//! <text class="pg-s" x="246" y="369">ties go to the first cell in grid order, lowest digit first.</text>
//! <line class="pg-ln" x1="670" y1="110" x2="707" y2="110" marker-end="url(#pg-m5)"/>
//! <rect class="pg-bx" x="710" y="40" width="226" height="140" rx="3"/>
//! <text class="pg-ch" x="726" y="66">search(givens)</text>
//! <text class="pg-d" x="726" y="86">always answers:</text>
//! <text class="pg-c" x="726" y="106">SearchResult</text>
//! <text class="pg-d" x="726" y="126">verdict: none, one or many</text>
//! <text class="pg-d" x="726" y="142">solutions: up to two full grids</text>
//! <text class="pg-d" x="726" y="158">guesses: how many splits</text>
//! <line class="pg-ln" x1="670" y1="290" x2="707" y2="290" marker-end="url(#pg-m5)"/>
//! <rect class="pg-bx" x="710" y="206" width="226" height="160" rx="3"/>
//! <text class="pg-ch" x="726" y="232">solve(givens)</text>
//! <text class="pg-d" x="726" y="252">exactly one solution:</text>
//! <text x="726" y="270"><tspan class="pg-c pg-st">WellPosed</tspan><tspan class="pg-d" dx="8">the proof</tspan></text>
//! <text class="pg-d" x="726" y="292">otherwise refuses:</text>
//! <text class="pg-c pg-rt" x="726" y="310">NoSolution</text>
//! <text class="pg-c pg-rt" x="726" y="326">ManySolutions</text>
//! <text x="726" y="342"><tspan class="pg-c pg-rt">NotPosed</tspan><tspan class="pg-d" dx="8">all 81 cells given</tspan></text>
//! </svg>
//! </div>
//!
//! *Both entries run the same search. One solution is shown to be the only one by running
//! every other waiting branch out. The guess count is part of the answer, so the order of
//! splits is fixed and documented.*
//!
//! ### `board`, the puzzle in play
//!
//! The [`Board`] is what a consumer plays on, and the only thing in the engine that changes
//! as a game goes on. It owns its puzzle and never hands it out, and everything it returns
//! is a copy, so every change to the game goes through the board. Moves can be undone
//! while the puzzle is unsolved. Solving is final, and checks are kept for good and never
//! undone.
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 450" role="img" aria-label="A Board comes from givens through open or from a Record through reopen. Inside are four parts: the puzzle with its hidden solution, the notes, the journal of moves, and the checks. It hands out five kinds of copies and has nineteen methods in four groups.">
//! <defs><marker id="pg-m6" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker></defs>
//! <rect class="pg-bx" x="24" y="84" width="160" height="64" rx="3"/>
//! <text class="pg-h" x="40" y="110">Givens</text>
//! <text class="pg-d" x="40" y="130">a new game</text>
//! <rect class="pg-bx" x="24" y="200" width="160" height="64" rx="3"/>
//! <text class="pg-ch" x="40" y="226">Record</text>
//! <text class="pg-d" x="40" y="246">a game written down</text>
//! <line class="pg-ln" x1="184" y1="116" x2="297" y2="116" marker-end="url(#pg-m6)"/>
//! <text class="pg-c" x="241" y="106" text-anchor="middle">Board::open</text>
//! <line class="pg-ln" x1="184" y1="232" x2="297" y2="232" marker-end="url(#pg-m6)"/>
//! <text class="pg-c" x="241" y="222" text-anchor="middle">Board::reopen</text>
//! <text class="pg-d" x="24" y="292">A new board comes from these two.</text>
//! <text class="pg-d" x="24" y="308">A board in hand can also be cloned,</text>
//! <text class="pg-d" x="24" y="324">history and all.</text>
//! <rect class="pg-frame" x="300" y="36" width="340" height="296" rx="3"/>
//! <text x="316" y="63"><tspan class="pg-ch">Board</tspan><tspan class="pg-d" dx="10">owns its puzzle and hides it</tspan></text>
//! <rect class="pg-bx" x="316" y="80" width="150" height="104" rx="3"/>
//! <text class="pg-ch" x="328" y="104">Puzzle</text>
//! <text class="pg-d" x="328" y="124">digits and status</text>
//! <rect class="pg-sol" x="328" y="138" width="126" height="30" rx="3"/>
//! <text class="pg-s pg-st" x="391" y="157" text-anchor="middle">the solution, hidden</text>
//! <rect class="pg-bx" x="474" y="80" width="150" height="104" rx="3"/>
//! <text class="pg-h" x="486" y="104">Notes</text>
//! <text class="pg-d" x="486" y="124">a note in every cell:</text>
//! <text class="pg-d" x="486" y="140">the marks the player</text>
//! <text class="pg-d" x="486" y="156">has written</text>
//! <rect class="pg-bx" x="316" y="200" width="150" height="104" rx="3"/>
//! <text class="pg-h" x="328" y="224">Journal</text>
//! <text class="pg-d" x="328" y="244">every move, in order,</text>
//! <text class="pg-d" x="328" y="260">what each displaced,</text>
//! <text class="pg-d" x="328" y="276">and how many stand</text>
//! <rect class="pg-bx" x="474" y="200" width="150" height="104" rx="3"/>
//! <text class="pg-h" x="486" y="224">Checks</text>
//! <text class="pg-d" x="486" y="244">every check asked,</text>
//! <text class="pg-d" x="486" y="260">in order; never</text>
//! <text class="pg-d" x="486" y="276">taken back</text>
//! <line class="pg-ln" x1="640" y1="184" x2="721" y2="184" marker-end="url(#pg-m6)"/>
//! <text class="pg-lab" x="724" y="28">HANDS OUT COPIES ONLY</text>
//! <rect class="pg-bx" x="724" y="40" width="212" height="48" rx="3"/>
//! <text class="pg-ch" x="738" y="60">BoardCell</text><text class="pg-d" x="738" y="78">one cell as it stands</text>
//! <rect class="pg-bx" x="724" y="97" width="212" height="48" rx="3"/>
//! <text class="pg-ch" x="738" y="117">Note</text><text class="pg-d" x="738" y="135">one cell's marks</text>
//! <rect class="pg-bx" x="724" y="154" width="212" height="48" rx="3"/>
//! <text class="pg-ch" x="738" y="174">Move</text><text class="pg-d" x="738" y="192">a move, and the board after it</text>
//! <rect class="pg-bx" x="724" y="211" width="212" height="48" rx="3"/>
//! <text class="pg-ch" x="738" y="231">Check</text><text class="pg-d" x="738" y="249">a question and its yes or no</text>
//! <rect class="pg-bx" x="724" y="268" width="212" height="48" rx="3"/>
//! <text class="pg-ch" x="738" y="288">Record</text><text class="pg-d" x="738" y="306">the whole game, written down</text>
//! <text class="pg-lab" x="24" y="348">ITS NINETEEN METHODS</text>
//! <rect class="pg-bx" x="24" y="356" width="120" height="76" rx="3"/>
//! <text class="pg-lab" x="36" y="377">OPEN · 1</text>
//! <text class="pg-c" x="36" y="399">open</text>
//! <rect class="pg-bx" x="152" y="356" width="330" height="76" rx="3"/>
//! <text class="pg-lab" x="164" y="377">READ · 9</text>
//! <text class="pg-c" x="164" y="399">status, is_full, is_consistent, can_undo,</text>
//! <text class="pg-c" x="164" y="417">can_redo, cell, cells, moves, checks</text>
//! <rect class="pg-bx" x="490" y="356" width="300" height="76" rx="3"/>
//! <text class="pg-lab" x="502" y="377">OPERATE · 7</text>
//! <text class="pg-c" x="502" y="399">place, erase, write_mark, strike_mark,</text>
//! <text class="pg-c" x="502" y="417">undo, redo, check</text>
//! <rect class="pg-bx" x="798" y="356" width="138" height="76" rx="3"/>
//! <text class="pg-lab" x="810" y="377">RECORD · 2</text>
//! <text class="pg-c" x="810" y="399">write, reopen</text>
//! </svg>
//! </div>
//!
//! *Four parts inside, five kinds of copy handed out. No method returns the puzzle. In
//! play, only [`Board::check`] consults the stored solution. Reopening consults it too, to
//! answer a record's checks again. Neither tells a digit of it.*
//!
//! ### `generation`, puzzles from draws
//!
//! [`generate`] makes a puzzle from a [`Tier`] and a stream of draws, by a published
//! method: draw a full grid, then take givens away while the solver's verdict stays one.
//! What comes back is the proof [`solve`] gives, so a puzzle is set from it, or a board
//! opened on its givens, in one more call. A tier is a construction setting. It says how a
//! puzzle is made and claims nothing about how hard it is.
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 336" role="img" aria-label="generate takes a tier and a stream of draws. It draws a solution grid, draws a bound, removes givens while one solution remains, and returns the givens left with the solver's proof, or one of three refusals.">
//! <defs>
//! <marker id="pg-m12" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker>
//! <marker id="pg-m12s" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah-s" d="M0 0L10 5L0 10z"/></marker>
//! <marker id="pg-m12r" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah-r" d="M0 0L10 5L0 10z"/></marker>
//! </defs>
//! <rect class="pg-bx" x="24" y="64" width="190" height="76" rx="3"/>
//! <text class="pg-ch" x="40" y="90">Tier</text>
//! <text class="pg-d" x="40" y="110">One to Five: a setting,</text>
//! <text class="pg-d" x="40" y="126">not a difficulty</text>
//! <rect class="pg-bx" x="24" y="176" width="190" height="64" rx="3"/>
//! <text class="pg-h" x="40" y="202">A stream of draws</text>
//! <text class="pg-d" x="40" y="222">any RandomStream</text>
//! <line class="pg-ln" x1="214" y1="102" x2="267" y2="102" marker-end="url(#pg-m12)"/>
//! <line class="pg-ln" x1="214" y1="208" x2="267" y2="208" marker-end="url(#pg-m12)"/>
//! <rect class="pg-frame" x="270" y="36" width="420" height="260" rx="3"/>
//! <text class="pg-ch" x="286" y="62">generate(tier, stream)</text>
//! <rect class="pg-bx" x="286" y="76" width="388" height="46" rx="3"/>
//! <text class="pg-lab" x="298" y="94">1 · THE SOLUTION GRID</text>
//! <text class="pg-d" x="298" y="112">eleven givens are drawn; the solver completes them</text>
//! <rect class="pg-bx" x="286" y="130" width="388" height="46" rx="3"/>
//! <text class="pg-lab" x="298" y="148">2 · THE BOUND</text>
//! <text class="pg-d" x="298" y="166">one draw picks a count of givens in the tier's range</text>
//! <rect class="pg-bx" x="286" y="184" width="388" height="46" rx="3"/>
//! <text class="pg-lab" x="298" y="202">3 · REMOVAL</text>
//! <text class="pg-d" x="298" y="220">81 visits; a given goes only if one solution remains</text>
//! <rect class="pg-bx" x="286" y="238" width="388" height="46" rx="3"/>
//! <text class="pg-lab" x="298" y="256">4 · THE RESULT</text>
//! <text class="pg-d" x="298" y="274">the givens left, with the solver's proof</text>
//! <line class="pg-ln-s" x1="690" y1="102" x2="753" y2="102" marker-end="url(#pg-m12s)"/>
//! <rect class="pg-sol" x="756" y="64" width="180" height="76" rx="3"/>
//! <text class="pg-c pg-st" x="770" y="90">WellPosed</text>
//! <text class="pg-d pg-st" x="770" y="110">the proof: the givens</text>
//! <text class="pg-d pg-st" x="770" y="126">and their one solution</text>
//! <line class="pg-ln-r" x1="690" y1="224" x2="753" y2="224" marker-end="url(#pg-m12r)"/>
//! <rect class="pg-ref" x="756" y="176" width="180" height="96" rx="3"/>
//! <text class="pg-h pg-rt" x="770" y="200">Refuses with</text>
//! <text class="pg-c pg-rt" x="770" y="220">Stream</text>
//! <text class="pg-c pg-rt" x="770" y="238">GridAttemptsSpent</text>
//! <text class="pg-c pg-rt" x="770" y="256">NotProved</text>
//! <text class="pg-d" x="24" y="322">The same tier and the same draws give the same puzzle, for one GENERATION_VERSION and one RANDOM_VERSION.</text>
//! </svg>
//! </div>
//!
//! *The tier and the draws decide the puzzle, and nothing else does. `Stream` is a stream
//! with no draw to give, and `GridAttemptsSpent` is a hundred failed tries at a grid. No
//! tier and no stream reaches `NotProved`: it is there so that a defect would be a refusal.*
//!
//! ### `random`, the one effect
//!
//! Randomness is the only thing the engine needs from outside, and it never fetches it.
//! The caller hands in a stream of numbers. The same seed and the same version give the
//! same numbers everywhere, so what is drawn in a browser can be replayed in Python or on a
//! command line. A draw can be refused. [`SeededStream`] always has one. [`ReplayStream`],
//! the fake that tests script, refuses a script holding a number outside the range and
//! reports when its script runs out.
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 300" role="img" aria-label="The caller supplies a stream that meets the RandomStream contract. Two streams implement it: SeededStream, which the library ships, and ReplayStream, the fake for tests. The generation module draws through it.">
//! <defs><marker id="pg-m7" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker></defs>
//! <rect class="pg-bx" x="24" y="44" width="190" height="84" rx="3"/>
//! <text class="pg-h" x="40" y="70">The caller</text>
//! <text class="pg-d" x="40" y="90">picks the seed and</text>
//! <text class="pg-d" x="40" y="106">hands in a stream</text>
//! <line class="pg-ln" x1="214" y1="86" x2="297" y2="86" marker-end="url(#pg-m7)"/>
//! <text class="pg-d" x="256" y="76" text-anchor="middle">supplies</text>
//! <rect class="pg-built" x="300" y="36" width="360" height="100" rx="3"/>
//! <text class="pg-ch" x="316" y="62">RandomStream</text>
//! <text class="pg-lab" x="648" y="61" text-anchor="end">THE CONTRACT</text>
//! <text x="316" y="86"><tspan class="pg-c">next_draw</tspan><tspan class="pg-d" dx="8">a number in [0, 1), or a RandomError</tspan></text>
//! <text x="316" y="104"><tspan class="pg-c">index</tspan><tspan class="pg-d" dx="8">how many draws so far</tspan></text>
//! <text class="pg-d" x="316" y="124">Same seed and version, same draws, on every machine.</text>
//! <line class="pg-ln" x1="386" y1="196" x2="386" y2="139" marker-end="url(#pg-m7)"/>
//! <line class="pg-ln" x1="574" y1="196" x2="574" y2="139" marker-end="url(#pg-m7)"/>
//! <text class="pg-d" x="480" y="172" text-anchor="middle">both implement it</text>
//! <rect class="pg-built" x="300" y="196" width="172" height="88" rx="3"/>
//! <text class="pg-ch" x="314" y="220">SeededStream</text>
//! <text class="pg-d" x="314" y="240">the generator shipped;</text>
//! <text class="pg-d" x="314" y="256">always has a draw</text>
//! <text class="pg-c pg-d" x="314" y="273">"seeded-stream-1"</text>
//! <rect class="pg-bx" x="488" y="196" width="172" height="88" rx="3"/>
//! <text class="pg-ch" x="502" y="220">ReplayStream</text>
//! <text class="pg-d" x="502" y="240">the fake for tests:</text>
//! <text class="pg-d" x="502" y="256">replays a script, and</text>
//! <text class="pg-d" x="502" y="272">says when it runs out</text>
//! <rect class="pg-built" x="772" y="40" width="164" height="96" rx="3"/>
//! <text class="pg-ch" x="784" y="64">generation</text>
//! <text class="pg-d" x="784" y="84">takes its draws here:</text>
//! <text class="pg-d" x="784" y="100">a grid, a bound and,</text>
//! <text class="pg-d" x="784" y="116">for two tiers, an order</text>
//! <line class="pg-ln" x1="772" y1="86" x2="663" y2="86" marker-end="url(#pg-m7)"/>
//! <text class="pg-d" x="717" y="76" text-anchor="middle">draws through</text>
//! <text class="pg-d" x="772" y="220">The engine never looks for</text>
//! <text class="pg-d" x="772" y="236">entropy itself. Only the</text>
//! <text class="pg-d" x="772" y="252">command line may ask the</text>
//! <text class="pg-d" x="772" y="268">operating system for a seed.</text>
//! </svg>
//! </div>
//!
//! *A draw is a number from 0 up to but not including 1, or a [`RandomError`]. Tests script
//! the draws through the fake, so code that uses chance is tested without chance.*
//!
//! ## Level 3: the mechanisms
//!
//! ### Opening a board
//!
//! [`Board::open`] is one call that does three things: it solves the givens, makes the
//! proof, and sets the puzzle. Opening runs the solver once. Playing never runs it again.
//! Reopening a written game runs it again, to find the solution afresh.
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 640" role="img" aria-label="Sequence for Board::open. The consumer calls the board, the board calls the solver, the solver searches and asks the rules for a proof, the board sets a puzzle from the proof and returns a board. The solver refuses givens with no solution, with many, or with nothing left to play. The solution passes from the search to the proof to the puzzle, and only a check consults it.">
//! <defs>
//! <marker id="pg-m8" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker>
//! <marker id="pg-m8r" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah-r" d="M0 0L10 5L0 10z"/></marker>
//! <marker id="pg-m8s" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah-s" d="M0 0L10 5L0 10z"/></marker>
//! </defs>
//! <line class="pg-life" x1="100" y1="60" x2="100" y2="530"/>
//! <line class="pg-life" x1="340" y1="60" x2="340" y2="530"/>
//! <line class="pg-life" x1="580" y1="60" x2="580" y2="530"/>
//! <line class="pg-life" x1="830" y1="60" x2="830" y2="530"/>
//! <rect class="pg-bx" x="30" y="24" width="140" height="36" rx="3"/>
//! <text class="pg-h" x="100" y="47" text-anchor="middle">Consumer</text>
//! <rect class="pg-built" x="270" y="24" width="140" height="36" rx="3"/>
//! <text class="pg-ch" x="340" y="47" text-anchor="middle">Board</text>
//! <rect class="pg-built" x="510" y="24" width="140" height="36" rx="3"/>
//! <text class="pg-ch" x="580" y="47" text-anchor="middle">solver</text>
//! <rect class="pg-built" x="760" y="24" width="140" height="36" rx="3"/>
//! <text class="pg-ch" x="830" y="47" text-anchor="middle">sudoku</text>
//! <line class="pg-ln" x1="100" y1="96" x2="337" y2="96" marker-end="url(#pg-m8)"/>
//! <text class="pg-c" x="220" y="88" text-anchor="middle">Board::open(givens)</text>
//! <line class="pg-ln" x1="340" y1="136" x2="577" y2="136" marker-end="url(#pg-m8)"/>
//! <text class="pg-c" x="460" y="128" text-anchor="middle">solve(givens)</text>
//! <path class="pg-ln" d="M580 156H612V186H584" marker-end="url(#pg-m8)"/>
//! <text class="pg-d" x="624" y="166">runs the search</text>
//! <text class="pg-d pg-rt" x="624" y="182">no solution: NoSolution</text>
//! <text class="pg-d pg-rt" x="624" y="198">more than one: ManySolutions</text>
//! <line class="pg-ln" x1="580" y1="240" x2="827" y2="240" marker-end="url(#pg-m8)"/>
//! <text class="pg-c" x="705" y="232" text-anchor="middle">WellPosed::vouch</text>
//! <text class="pg-d" x="705" y="258" text-anchor="middle">is the solution whole, valid</text>
//! <text class="pg-d" x="705" y="274" text-anchor="middle">and true to every given?</text>
//! <text class="pg-d pg-rt" x="705" y="290" text-anchor="middle">all 81 cells given: NotPosed</text>
//! <line class="pg-ln-s pg-back" x1="830" y1="320" x2="583" y2="320" marker-end="url(#pg-m8s)"/>
//! <text class="pg-d pg-st" x="705" y="312" text-anchor="middle">the proof, with the solution in it</text>
//! <line class="pg-ln-s pg-back" x1="580" y1="356" x2="343" y2="356" marker-end="url(#pg-m8s)"/>
//! <text class="pg-d pg-st" x="460" y="348" text-anchor="middle">the proof (WellPosed)</text>
//! <line class="pg-ln" x1="340" y1="396" x2="827" y2="396" marker-end="url(#pg-m8)"/>
//! <text class="pg-c" x="460" y="388" text-anchor="middle">Puzzle::set(proof)</text>
//! <text class="pg-d" x="705" y="388" text-anchor="middle">cannot fail</text>
//! <line class="pg-ln-s pg-back" x1="830" y1="436" x2="343" y2="436" marker-end="url(#pg-m8s)"/>
//! <text class="pg-d pg-st" x="460" y="428" text-anchor="middle">a Puzzle; the solution stays inside</text>
//! <line class="pg-ln pg-back" x1="340" y1="476" x2="103" y2="476" marker-end="url(#pg-m8)"/>
//! <text x="220" y="468" text-anchor="middle">a Board, nothing played yet</text>
//! <line class="pg-ln-r pg-back" x1="340" y1="512" x2="103" y2="512" marker-end="url(#pg-m8r)"/>
//! <text class="pg-rt" x="220" y="504" text-anchor="middle">or a SolveError</text>
//! <text class="pg-lab" x="24" y="562">WHERE THE SOLUTION GOES</text>
//! <rect class="pg-sol" x="24" y="574" width="168" height="52" rx="3"/>
//! <text class="pg-h" x="36" y="596">The search</text><text class="pg-d" x="36" y="614">finds it</text>
//! <line class="pg-ln-s" x1="192" y1="600" x2="207" y2="600" marker-end="url(#pg-m8s)"/>
//! <rect class="pg-sol" x="210" y="574" width="168" height="52" rx="3"/>
//! <text class="pg-ch" x="222" y="596">WellPosed</text><text class="pg-d" x="222" y="614">carries it to the rules</text>
//! <line class="pg-ln-s" x1="378" y1="600" x2="393" y2="600" marker-end="url(#pg-m8s)"/>
//! <rect class="pg-sol" x="396" y="574" width="168" height="52" rx="3"/>
//! <text class="pg-ch" x="408" y="596">Puzzle</text><text class="pg-d" x="408" y="614">keeps it, hidden</text>
//! <line class="pg-ln-s" x1="564" y1="600" x2="579" y2="600" marker-end="url(#pg-m8s)"/>
//! <rect class="pg-sol" x="582" y="574" width="168" height="52" rx="3"/>
//! <text class="pg-ch" x="594" y="596">Board::check</text><text class="pg-d" x="594" y="614">asks about one cell</text>
//! <line class="pg-ln-s" x1="750" y1="600" x2="765" y2="600" marker-end="url(#pg-m8s)"/>
//! <rect class="pg-bx" x="768" y="574" width="168" height="52" rx="3"/>
//! <text class="pg-h" x="780" y="596">The player</text><text class="pg-d" x="780" y="614">hears yes or no</text>
//! </svg>
//! </div>
//!
//! *Solid arrows are calls and dashed arrows are what comes back. The three refusals are
//! the solver's, and the board passes them on unchanged.*
//!
//! ### Playing a move
//!
//! Every operation checks its guards in a fixed order and reports the first one that
//! fails. A refused operation changes nothing: no move is recorded, no undone move is
//! discarded, and no check is kept.
//!
//! | Operation | Refuses with the first of these that applies |
//! | --- | --- |
//! | [`place`] | `NoSuchCell`, `Solved`, `GivenCell`, `DigitOutOfRange`, `DigitAlreadyStands` |
//! | [`erase`] | `NoSuchCell`, `Solved`, `NoPlayersDigit` |
//! | [`write_mark`] | `NoSuchCell`, `Solved`, `MarksNotAccepted`, `DigitOutOfRange`, `MarkAlreadyWritten` |
//! | [`strike_mark`] | `NoSuchCell`, `Solved`, `MarksNotAccepted`, `MarkNotThere` |
//! | [`check`] | `NoSuchCell`, `Solved`, `NoPlayersDigit` |
//! | [`undo`] | `Solved`, `NothingToUndo` |
//! | [`redo`] | `Solved`, `NothingToRedo` |
//!
//! The first two guards are shared by every operation on a cell: a position off the grid,
//! then a solved puzzle. Together these are the eleven kinds of [`PlayError`]:
//!
//! | Kind | The rule that failed |
//! | --- | --- |
//! | `NoSuchCell` | the position is off the grid |
//! | `Solved` | the puzzle is solved, and solved is final |
//! | `GivenCell` | the cell holds a given |
//! | `DigitOutOfRange` | the digit is not from 1 to 9 |
//! | `DigitAlreadyStands` | the cell already holds that digit |
//! | `NoPlayersDigit` | the cell is empty or holds a given |
//! | `MarksNotAccepted` | the cell holds a digit, so it takes no marks |
//! | `MarkAlreadyWritten` | the mark is already in the note |
//! | `MarkNotThere` | the mark is not in the note |
//! | `NothingToUndo` | no move stands |
//! | `NothingToRedo` | no move is undone |
//!
//! #### What `place` does
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 300" role="img" aria-label="Placing a digit in five steps: guard, find which peers' notes hold the digit, write the move down with what it displaces, perform it on the puzzle and the notes, and record it in the journal. An example shows a peer's note losing the mark 4 when 4 is placed beside it.">
//! <defs><marker id="pg-m9" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker></defs>
//! <rect class="pg-bx" x="24" y="40" width="168" height="96" rx="3"/>
//! <text class="pg-lab" x="36" y="61">1 · GUARD</text>
//! <text class="pg-d" x="36" y="83">on the grid, not solved,</text>
//! <text class="pg-d" x="36" y="99">not a given, digit 1–9,</text>
//! <text class="pg-d" x="36" y="115">not already there</text>
//! <line class="pg-ln" x1="192" y1="88" x2="207" y2="88" marker-end="url(#pg-m9)"/>
//! <rect class="pg-bx" x="210" y="40" width="168" height="96" rx="3"/>
//! <text class="pg-lab" x="222" y="61">2 · FIND THE UPKEEP</text>
//! <text class="pg-d" x="222" y="83">which peers' notes</text>
//! <text class="pg-d" x="222" y="99">hold this digit?</text>
//! <line class="pg-ln" x1="378" y1="88" x2="393" y2="88" marker-end="url(#pg-m9)"/>
//! <rect class="pg-bx" x="396" y="40" width="168" height="96" rx="3"/>
//! <text class="pg-lab" x="408" y="61">3 · WRITE THE MOVE</text>
//! <text class="pg-d" x="408" y="83">target, digit, what the</text>
//! <text class="pg-d" x="408" y="99">cell held before, which</text>
//! <text class="pg-d" x="408" y="115">notes lose the digit</text>
//! <line class="pg-ln" x1="564" y1="88" x2="579" y2="88" marker-end="url(#pg-m9)"/>
//! <rect class="pg-bx" x="582" y="40" width="168" height="96" rx="3"/>
//! <text class="pg-lab" x="594" y="61">4 · PERFORM</text>
//! <text class="pg-d" x="594" y="83">the puzzle takes the</text>
//! <text class="pg-d" x="594" y="99">digit; those notes</text>
//! <text class="pg-d" x="594" y="115">lose it</text>
//! <line class="pg-ln" x1="750" y1="88" x2="765" y2="88" marker-end="url(#pg-m9)"/>
//! <rect class="pg-bx" x="768" y="40" width="168" height="96" rx="3"/>
//! <text class="pg-lab" x="780" y="61">5 · RECORD</text>
//! <text class="pg-d" x="780" y="83">the journal drops any</text>
//! <text class="pg-d" x="780" y="99">undone moves, then</text>
//! <text class="pg-d" x="780" y="115">appends this one</text>
//! <text class="pg-d pg-rt" x="108" y="156" text-anchor="middle">else a PlayError;</text>
//! <text class="pg-d pg-rt" x="108" y="172" text-anchor="middle">nothing changes</text>
//! <text class="pg-d" x="666" y="156" text-anchor="middle">a full grid with no</text>
//! <text class="pg-d" x="666" y="172" text-anchor="middle">conflict: Solved</text>
//! <text class="pg-h" x="24" y="226">Upkeep, by example</text>
//! <text class="pg-d" x="24" y="246">The peer's note holds 4.</text>
//! <text class="pg-d" x="24" y="262">Placing 4 here strikes it.</text>
//! <rect class="pg-bx" x="280" y="204" width="64" height="64"/>
//! <rect class="pg-bx" x="344" y="204" width="64" height="64"/>
//! <text class="pg-mk" x="376" y="219" text-anchor="middle">2</text>
//! <text class="pg-mk" x="355" y="240" text-anchor="middle">4</text>
//! <text class="pg-mk" x="397" y="240" text-anchor="middle">6</text>
//! <text class="pg-s" x="312" y="286" text-anchor="middle">here</text>
//! <text class="pg-s" x="376" y="286" text-anchor="middle">a peer</text>
//! <line class="pg-ln" x1="422" y1="236" x2="545" y2="236" marker-end="url(#pg-m9)"/>
//! <text class="pg-c" x="484" y="226" text-anchor="middle">place(here, 4)</text>
//! <rect class="pg-bx" x="562" y="204" width="64" height="64"/>
//! <text class="pg-big" x="594" y="247" text-anchor="middle">4</text>
//! <rect class="pg-bx" x="626" y="204" width="64" height="64"/>
//! <text class="pg-mk" x="658" y="219" text-anchor="middle">2</text>
//! <text class="pg-mk pg-rt" x="637" y="240" text-anchor="middle">4</text>
//! <line class="pg-ln-r" x1="630" y1="236" x2="644" y2="236"/>
//! <text class="pg-mk" x="679" y="240" text-anchor="middle">6</text>
//! <text class="pg-s" x="594" y="286" text-anchor="middle">here</text>
//! <text class="pg-s" x="658" y="286" text-anchor="middle">a peer</text>
//! <text class="pg-d" x="716" y="214">Placing keeps this cell's own note</text>
//! <text class="pg-d" x="716" y="230">beneath the digit; erasing shows it</text>
//! <text class="pg-d" x="716" y="246">again. Erasing gives peers nothing</text>
//! <text class="pg-d" x="716" y="262">back; undo returns the 4 to exactly</text>
//! <text class="pg-d" x="716" y="278">the notes that lost it.</text>
//! </svg>
//! </div>
//!
//! *The move is written down with what it displaced before anything changes, which is what
//! lets undo put the board back exactly. A conflicting digit passes the guards, as it would
//! on paper.*
//!
//! #### The journal: undo, redo and a new move
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 316" role="img" aria-label="The journal is a list of moves and a marker for how many stand. With three moves the marker is at three. Undo moves it to two and leaves move three on the list, undone. A new move then cuts the undone move off and takes its index.">
//! <text class="pg-h" x="24" y="62">Three moves, all standing</text>
//! <text class="pg-d" x="24" y="80">a move's index is its place in the list</text>
//! <rect class="pg-built" x="270" y="44" width="130" height="44" rx="3"/><text class="pg-c" x="282" y="71">1 · write_mark</text>
//! <rect class="pg-built" x="412" y="44" width="130" height="44" rx="3"/><text class="pg-c" x="424" y="71">2 · place</text>
//! <rect class="pg-built" x="554" y="44" width="130" height="44" rx="3"/><text class="pg-c" x="566" y="71">3 · place</text>
//! <line class="pg-mkr" x1="691" y1="38" x2="691" y2="94"/>
//! <text class="pg-c pg-it" x="691" y="30" text-anchor="middle">standing = 3</text>
//! <text class="pg-d" x="716" y="63">undo can take back move 3;</text>
//! <text class="pg-d" x="716" y="79">there is nothing to redo</text>
//! <text class="pg-ch" x="24" y="162">undo</text>
//! <text class="pg-d" x="24" y="180">move 3 is reverted and stays</text>
//! <text class="pg-d" x="24" y="196">on the list, undone</text>
//! <rect class="pg-built" x="270" y="144" width="130" height="44" rx="3"/><text class="pg-c" x="282" y="171">1 · write_mark</text>
//! <rect class="pg-built" x="412" y="144" width="130" height="44" rx="3"/><text class="pg-c" x="424" y="171">2 · place</text>
//! <rect class="pg-plan" x="554" y="144" width="130" height="44" rx="3"/><text class="pg-c pg-d" x="566" y="171">3 · place</text>
//! <line class="pg-mkr" x1="548" y1="138" x2="548" y2="194"/>
//! <text class="pg-c pg-it" x="548" y="130" text-anchor="middle">standing = 2</text>
//! <text class="pg-d" x="716" y="163">redo would re-take move 3</text>
//! <text class="pg-d" x="716" y="179">and move the marker back</text>
//! <text class="pg-h" x="24" y="262">A new move instead of redo</text>
//! <text class="pg-d" x="24" y="280">the undone tail is cut and the</text>
//! <text class="pg-d" x="24" y="296">new move takes index 3</text>
//! <rect class="pg-built" x="270" y="244" width="130" height="44" rx="3"/><text class="pg-c" x="282" y="271">1 · write_mark</text>
//! <rect class="pg-built" x="412" y="244" width="130" height="44" rx="3"/><text class="pg-c" x="424" y="271">2 · place</text>
//! <rect class="pg-built" x="554" y="244" width="130" height="44" rx="3"/><text class="pg-c" x="566" y="271">3 · erase</text>
//! <line class="pg-mkr" x1="691" y1="238" x2="691" y2="294"/>
//! <text class="pg-c pg-it" x="691" y="230" text-anchor="middle">standing = 3</text>
//! <rect class="pg-plan" x="716" y="244" width="110" height="44" rx="3"/><text class="pg-c pg-d" x="728" y="271">3 · place</text>
//! <line class="pg-ln-r" x1="716" y1="266" x2="826" y2="266"/>
//! <text class="pg-d pg-rt" x="838" y="263">discarded</text>
//! <text class="pg-d pg-rt" x="838" y="279">for good</text>
//! </svg>
//! </div>
//!
//! *One list and one number. Moves up to the marker stand, and moves after it are undone,
//! so the undone moves are always the latest ones.*
//!
//! Reading back uses the same list. [`Board::moves`] works out the board after each move
//! by walking forward from the opening position, so asking what the board looked like never
//! disturbs the board.
//!
//! #### The check
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 164" role="img" aria-label="A check takes a position, reads the player's digit there, asks the puzzle whether it is the solution's digit, and keeps a Check holding yes or no.">
//! <defs>
//! <marker id="pg-m10" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker>
//! <marker id="pg-m10s" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah-s" d="M0 0L10 5L0 10z"/></marker>
//! </defs>
//! <rect class="pg-bx" x="24" y="30" width="210" height="76" rx="3"/>
//! <text class="pg-ch" x="38" y="56">check(position)</text>
//! <text class="pg-d" x="38" y="76">the player asks about</text>
//! <text class="pg-d" x="38" y="92">one cell</text>
//! <line class="pg-ln" x1="234" y1="68" x2="255" y2="68" marker-end="url(#pg-m10)"/>
//! <rect class="pg-bx" x="258" y="30" width="210" height="76" rx="3"/>
//! <text class="pg-h" x="272" y="56">The player's digit</text>
//! <text class="pg-d" x="272" y="76">as it stands in that cell</text>
//! <line class="pg-ln" x1="468" y1="68" x2="489" y2="68" marker-end="url(#pg-m10)"/>
//! <rect class="pg-sol" x="492" y="30" width="210" height="76" rx="3"/>
//! <text class="pg-ch pg-st" x="506" y="56">Puzzle</text>
//! <text class="pg-d pg-st" x="506" y="76">is that the solution's</text>
//! <text class="pg-d pg-st" x="506" y="92">digit in this cell?</text>
//! <line class="pg-ln-s" x1="702" y1="68" x2="723" y2="68" marker-end="url(#pg-m10s)"/>
//! <rect class="pg-bx" x="726" y="30" width="210" height="76" rx="3"/>
//! <text class="pg-ch" x="740" y="56">Check</text>
//! <text class="pg-d" x="740" y="76">yes or no, kept on the</text>
//! <text class="pg-d" x="740" y="92">board in the order asked</text>
//! <text class="pg-d" x="24" y="134">A check is not a move. It changes no cell and no note, discards no undone move, and undo never removes it.</text>
//! <text class="pg-d" x="24" y="150">The solution's digit is never told, here or anywhere else on a board.</text>
//! </svg>
//! </div>
//!
//! *In play this is the only place the stored solution is consulted. What comes out is yes
//! or no, about a digit the player already chose.*
//!
//! ### Write and reopen
//!
//! A [`Record`] holds only what cannot be worked out. Reopening makes the record's moves
//! again through the board's own operations, so everything a record leaves out is worked
//! out by the code that produced it the first time. A record's checks are held to less. The
//! moves that stood when a check was asked may since have been discarded, so reopening
//! tests each check on its own and answers it again from the solution, and does not match
//! it to the moves that are left.
//!
//! Written in the record:
//!
//! - the givens;
//! - every move in the order made: its kind and its cell, and its digit for a placement or
//!   a mark. An erasure records only its cell;
//! - how many of the latest moves are undone;
//! - every check: how many moves stood, its cell and the player's digit.
//!
//! Left out, and worked out again:
//!
//! - every cell's digit and every note;
//! - what each move displaced, and which notes upkeep struck;
//! - each check's answer;
//! - anything read from the solution.
//!
//! Loading a record from storage checks at most its shape and the types of its fields. It
//! does not ask whether the record describes a game. [`Board::reopen`] is where that is
//! judged: it works through five stages in a fixed order and refuses at the first thing no
//! board could have written.
//!
//! <div class="pg-fig">
//! <svg class="pg" viewBox="0 0 960 300" role="img" aria-label="Reopening in five stages: check the counts, solve the givens, replay every move, take back the undone ones, and answer each check again. Each stage has its own ReopenError kinds. If nothing is refused the result is the same board.">
//! <defs><marker id="pg-m11" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="pg-ah" d="M0 0L10 5L0 10z"/></marker></defs>
//! <rect class="pg-bx" x="24" y="36" width="168" height="112" rx="3"/>
//! <text class="pg-lab" x="36" y="57">1 · COUNT</text>
//! <text class="pg-h" x="36" y="79">Check the counts</text>
//! <text class="pg-d" x="36" y="99">no more undone than</text>
//! <text class="pg-d" x="36" y="115">moves; a check needs</text>
//! <text class="pg-d" x="36" y="131">a move</text>
//! <line class="pg-ln" x1="192" y1="92" x2="207" y2="92" marker-end="url(#pg-m11)"/>
//! <rect class="pg-bx" x="210" y="36" width="168" height="112" rx="3"/>
//! <text class="pg-lab" x="222" y="57">2 · OPEN</text>
//! <text class="pg-h" x="222" y="79">Solve the givens</text>
//! <text class="pg-d" x="222" y="99">Board::open, as for a</text>
//! <text class="pg-d" x="222" y="115">new game: the solver</text>
//! <text class="pg-d" x="222" y="131">runs again</text>
//! <line class="pg-ln" x1="378" y1="92" x2="393" y2="92" marker-end="url(#pg-m11)"/>
//! <rect class="pg-bx" x="396" y="36" width="168" height="112" rx="3"/>
//! <text class="pg-lab" x="408" y="57">3 · REPLAY</text>
//! <text class="pg-h" x="408" y="79">Make every move</text>
//! <text class="pg-d" x="408" y="99">in order, through the</text>
//! <text class="pg-d" x="408" y="115">board's own guards</text>
//! <line class="pg-ln" x1="564" y1="92" x2="579" y2="92" marker-end="url(#pg-m11)"/>
//! <rect class="pg-bx" x="582" y="36" width="168" height="112" rx="3"/>
//! <text class="pg-lab" x="594" y="57">4 · TAKE BACK</text>
//! <text class="pg-h" x="594" y="79">Undo the undone</text>
//! <text class="pg-d" x="594" y="99">as many times as the</text>
//! <text class="pg-d" x="594" y="115">record says</text>
//! <line class="pg-ln" x1="750" y1="92" x2="765" y2="92" marker-end="url(#pg-m11)"/>
//! <rect class="pg-bx" x="768" y="36" width="168" height="112" rx="3"/>
//! <text class="pg-lab" x="780" y="57">5 · ASK AGAIN</text>
//! <text class="pg-h" x="780" y="79">Answer each check</text>
//! <text class="pg-d" x="780" y="99">afresh, from the</text>
//! <text class="pg-d" x="780" y="115">solution just found</text>
//! <text class="pg-lab pg-rt" x="36" y="170">REFUSES WITH</text>
//! <text class="pg-c pg-rt" x="36" y="188">TooManyUndone</text>
//! <text class="pg-c pg-rt" x="36" y="204">ChecksWithoutMoves</text>
//! <text class="pg-lab pg-rt" x="222" y="170">REFUSES WITH</text>
//! <text class="pg-c pg-rt" x="222" y="188">Givens</text>
//! <text class="pg-lab pg-rt" x="408" y="170">REFUSES WITH</text>
//! <text class="pg-c pg-rt" x="408" y="188">MoveAfterSolved</text>
//! <text class="pg-c pg-rt" x="408" y="204">Move</text>
//! <text class="pg-lab pg-rt" x="594" y="170">REFUSES WITH</text>
//! <text class="pg-c pg-rt" x="594" y="188">UndoneWhenSolved</text>
//! <text class="pg-lab pg-rt" x="780" y="170">REFUSES WITH</text>
//! <text class="pg-c pg-rt" x="780" y="188">Check</text>
//! <text class="pg-c pg-rt" x="780" y="204">CheckAfterNoMove</text>
//! <line class="pg-ln" x1="920" y1="148" x2="920" y2="245" marker-end="url(#pg-m11)"/>
//! <text class="pg-d" x="908" y="232" text-anchor="end">nothing refused</text>
//! <rect class="pg-built" x="24" y="248" width="912" height="40" rx="3"/>
//! <text x="480" y="273" text-anchor="middle">The same board: the same digits and notes, the same moves with the same ones undone, the same checks with the same answers.</text>
//! </svg>
//! </div>
//!
//! *The two counts are asked before the search, so a record that fails them costs no
//! search. The eight names are the kinds of [`ReopenError`].*
//!
//! ## Where this comes from
//!
//! The handbook explains why the engine is built this way, on GitHub:
//! [Architecture], [Layering and dependency direction][layering page] and
//! [Purpose and scope].
//!
//! [layering page]: https://github.com/steven-cutting/libpawdoku/blob/main/docs/explanation/layering.md
//! [Architecture]: https://github.com/steven-cutting/libpawdoku/blob/main/docs/explanation/architecture.md
//! [Purpose and scope]: https://github.com/steven-cutting/libpawdoku/blob/main/docs/project/purpose-and-scope.md
//! [`Board`]: crate::board::Board
//! [`Board::open`]: crate::board::Board::open
//! [`Board::reopen`]: crate::board::Board::reopen
//! [`Board::check`]: crate::board::Board::check
//! [`Board::moves`]: crate::board::Board::moves
//! [`place`]: crate::board::Board::place
//! [`erase`]: crate::board::Board::erase
//! [`write_mark`]: crate::board::Board::write_mark
//! [`strike_mark`]: crate::board::Board::strike_mark
//! [`check`]: crate::board::Board::check
//! [`undo`]: crate::board::Board::undo
//! [`redo`]: crate::board::Board::redo
//! [`Record`]: crate::board::Record
//! [`PlayError`]: crate::board::PlayError
//! [`ReopenError`]: crate::board::ReopenError
//! [`Puzzle`]: crate::sudoku::Puzzle
//! [`WellPosed`]: crate::sudoku::WellPosed
//! [`Position`]: crate::sudoku::Position
//! [`Given`]: crate::sudoku::Given
//! [`Cell`]: crate::sudoku::Cell
//! [`search`]: crate::solver::search
//! [`solve`]: crate::solver::solve
//! [`SearchResult`]: crate::solver::SearchResult
//! [`generate`]: crate::generation::generate
//! [`Tier`]: crate::generation::Tier
//! [`SeededStream`]: crate::random::SeededStream
//! [`ReplayStream`]: crate::random::ReplayStream
//! [`RandomError`]: crate::random::RandomError
