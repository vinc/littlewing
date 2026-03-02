# Changelog

## Unreleased

## 0.8.0 (2026-03-03)
### Fixed
- Fix insertion sort in movegen (#143)
- Do check before search in root (#131)
- Fix internal iterative deepening (#130)
- Fix UCI issue with PV after draw (#112)
- Fix invalid claim (#103)
- Fix std (#100)
- Fix clippy warnings (#99)
### Added
- Add TT prefetch (#145)
- Add check extension (#142)
- Add history heuristic (#128)
- Add improving heuristic (#139)
- Add libm to compute log in no_std (#146)
- Add noisy/quiet moves (#144)
- Add reverse futility pruning (#135)
- Add tempo to eval (#127)
- Add automated tuning (#122)
- Add depth command (#101)
### Changed
- Reduce futility pruning margin (#138)
- Refactor transposition table (#137)
- Update LMR (#140)
- Update eval params (#133)
- Replace Bencher by Criterion (#123)
- Improve UCI support (#114)
- Use safe lockless transposition table (#113)
- Set default number of moves for UCI go command (#104)
- Migrate from 2015 edition to 2018 (#98)
- Change license from GPL to MIT (#97)
### Bumped
- Update criterion requirement from 0.7 to 0.8 (#126)
- Update rand crates (#120)
- Update rustyline requirement from 11.0.0 to 17.0.2 (#119)
- Update dirs requirement from 5.0.1 to 6.0.0 (#118)
- Upgrade GitHub Actions cache (#111)
- Update dirs requirement from 4.0.0 to 5.0.1 (#110)
- Update rustyline requirement from 10.0.0 to 11.0.0 (#105)
- Update rustyline-derive requirement from 0.7.0 to 0.8.0 (#106)
- Update crates (#96)

## 0.7.0 (2021-08-21)
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
