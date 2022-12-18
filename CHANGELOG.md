# Changelog

## Unreleased

### 0.7.0 (2021-08-21)
### Fixed
- Fix getopts parsing (#75)
### Added
- Add `no_std` support to lib (#81)
- Add chess prelude to lib (#82)
### Changed
- Increase search reductions (#47)
- Speed up large transposition table creations (#48)
- Used std::time instead of time (#49)
- Upgrade dependencies (#58)
- Migrate from TravisCI to GitHub Actions (#66)
- Upgrade to GitHub-native Dependabot (#72)
- Update colored requirement from 1.9.3 to 2.0.0 (#60)

## 0.6.0 (2019-12-22)
### Fixed
- Fix castling right update bug
- Fix pawn move disambiguation in SAN
- Fix white pieces color with black on white terminals in CLI
- Avoid panicking in CLI
### Added
- Add makefile
- Parse moves in SAN format (#41)
- Add `go movetime` command to UCI (#40)
- Add `load pgn` and `save pgn` commands to CLI (#42)
- Add `play none` subcommand to CLI
- Add `init` command to CLI
- Add command history file to CLI (#38)
- Add autocompletion to CLI (#36)
### Changed
- Increase futility pruning depth (#44)
- Avoid TT cutoff on PV-nodes
- Split PV over multiple lines in CLI mode (#45)
- Add error propagation to CLI commands (#43)
- Improve colors in CLI
- Refactor board drawing
- Update dependencies

## 0.5.0 (2018-07-18)
### Added
- Use Hyperbola Quintessence and First Rank Attacks for sliding piece attacks
- Use Xorshift random number generator for Zobrist hashing
- Add depth parameter to `perft` CLI command
- Add `save fen` and `save pgn` CLI commands
- Add `--silent` flag to executable
### Changed
- Improve CLI output
- Rename `load` CLI command to `load fen`
- Rename `Direction` to `Shift` and introduce a new (compass) `Direction`
- Rename `Move*` to `PieceMove*`
- Store castling rights in a u8 in `Position`
- Use fail-soft instead of fail-hard in search
- Refactor many parts of the code

## 0.4.0 (2017-11-20)
### Added
- Add library with public API and documentation
- Add getopt to parse program options
- Add readline to user interface
- Add piece square tables to evaluation
- Add upper and lower bounds in transposition table
- Add age field in transposition table
- Add delta pruning to quiescence search
- Add transposition table to quiescence search
- Add basic UCI support
- Add very basic parallel search in threads with shared transposition table
### Changed
- Improve CLI
- Improve statistics debug output
- Refactor many parts of the code
- Allow NMP, IID, and LMR at shallower depth
- Set NMP R to 3

## 0.3.0 (2017-10-11)
### Added
- Add principal variation search
- Add internal iterative deepening
- Add late move reduction
- Add killer heuristic
- Add basic null move pruning
- Add basic futility pruning
- Add basic mobility evaluation
- Add static exchange evaluation
### Changed
- Use static exchange evaluation in moves ordering
- Skip bad captures in quiescence search

## 0.2.0 (2016-08-22)
### Added
- Add basic quiescence search
- Add basic transposition table
- Add MVV/LVA moves ordering by insertion sort
- Add staged moves generation
- Add fullmoves and halfmoves counting
- Add draw detection
- Add mate pruning
- Add XBoard `memory` command
- Add `color` and `debug` command line flag
### Changed
- Improve user interface
- Display game result in XBoard
- Save best move during iterative deepening
- Print principal variation from transpositions table
### Fixed
- Fix compiler warnings
- Fix castling bug
- Fix bug when undoing promotions

## 0.1.0 (2016-08-10)
### Changed
- Improve time management
### Fixed
- Fix compiler errors and warnings
- Fix bug in search function
- Fix promotion parsing bug

## 0.0.1 (2015-06-09)
### Added
- Add bitboard moves generation with De Bruijn sequence
- Add board array representation
- Add basic evaluation
- Add alpha beta pruning
- Add iterative deepening
- Add basic time management
- Add support of XBoard protocol
- Add Zobrist hashing
- Add FEN support
- Add `perft`, `perftsuite`, and `divide` commands in user interface
- Add Travis CI

## 0.0.0 (2014-12-23)
### Added
- Initial commit
