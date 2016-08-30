# Changelog

## [Unreleased]

### Added

- Add principal variation search
- Add internal iterative deepening
- Add late move reduction
- Add killer heuristic
- Add basic null move pruning
- Add basic mobility evaluation


## [0.2.0] - 2016-08-22

### Added
- Add basic quiescence search
- Add basic transpositions table
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


## [0.1.0] - 2016-08-10

### Changed
- Improve time management

### Fixed
- Fix compiler errors and warnings
- Fix bug in search function
- Fix promotion parsing bug


## [0.0.1] - 2015-06-09

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


## [0.0.0] - 2014-12-23

### Added
- Initial commit
