Little Wing
===========

[![Travis](https://img.shields.io/travis/vinc/littlewing/master.svg)](https://travis-ci.org/vinc/littlewing/branches)
[![Crates.io](https://img.shields.io/crates/v/littlewing.svg)](https://crates.io/crates/littlewing)

A bitboard chess engine written in Rust.

A work in progress since December 2014.

- Board representation
  - Bitboard with LLVM CTPOP and CTTZ
  - FEN support
  - Zobrist hashing
  - Staged moves generation
  - MVV/LVA and SEE moves ordering with insertion sort
- Search
  - Principal variation search
  - Quiescence search
  - Transpositions table
  - Null move pruning
  - Internal iterative deepening
  - Futility pruning
  - Late move reduction
  - Killer heuristic
- Evaluation
  - Piece square table evaluation
  - Mobility evaluation
  - Static exchange evaluation
- Interface
  - CLI with play and debug commands
  - XBoard and UCI communication protocol
  - Public API with documented library

Tested on GNU/Linux 32 and 64 bits, should run anywhere.


Usage
-----

First you need to install Rust:

    $ curl https://sh.rustup.rs -sSf | sh

Then you can install the latest stable version of the engine with cargo:

    $ cargo install littlewing

Or the development version by fetching the git repository:

    $ git clone https://github.com/vinc/littlewing.git
    $ cd littlewing
    $ LITTLEWING_VERSION=$(git describe) cargo build --release
    $ sudo cp target/release/littlewing /usr/local/bin

Little Wing is compatible with XBoard and UCI communication protocols,
in addition it has its own text-based user interface:

    $ littlewing --color --debug
    Little Wing v0.3.0

    > show board
    +---+---+---+---+---+---+---+---+
    | r | n | b | q | k | b | n | r |
    +---+---+---+---+---+---+---+---+
    | p | p | p | p | p | p | p | p |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   |   |   |   |   |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   |   |   |   |   |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   |   |   |   |   |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   |   |   |   |   |
    +---+---+---+---+---+---+---+---+
    | P | P | P | P | P | P | P | P |
    +---+---+---+---+---+---+---+---+
    | R | N | B | Q | K | B | N | R |
    +---+---+---+---+---+---+---+---+
    > move e2e4
    +---+---+---+---+---+---+---+---+
    | r | n | b | q | k | b | n | r |
    +---+---+---+---+---+---+---+---+
    | p | p | p | p | p | p | p | p |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   |   |   |   |   |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   |   |   |   |   |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   | P |   |   |   |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   |   |   |   |   |
    +---+---+---+---+---+---+---+---+
    | P | P | P | P |   | P | P | P |
    +---+---+---+---+---+---+---+---+
    | R | N | B | Q | K | B | N | R |
    +---+---+---+---+---+---+---+---+
    > show think
    > time 1 10
    > play
    # using 0 threads
    # FEN rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
    # allocating 10000 ms to move
    # starting search at depth 1
     ply   score   time     nodes  pv
       1     -46      0         1  1. ... a6
       1     -45      0         3  1. ... c6
       1     -22      0         4  1. ... d6
       1     -20      0         5  1. ... e6
       1      -1      0        14  1. ... d5
       1       0      0        15  1. ... e5
       1       9      0        20  1. ... Nc6
       2     -47      0        53  1. ... Nc6 2. Nc3
       3       9      0       278  1. ... Nc6 2. Nc3 Nf6
       4     -45      1       860  1. ... Nc6 2. Nc3 Nf6 3. Nf3
       4     -31      1      2435  1. ... d5 2. exd5 Qxd5 3. Nc3
       5     -32      1      4559  1. ... d5 2. exd5 Qxd5 3. Nc3 Qd4
       5     -21      2      7708  1. ... d6 2. Qe2 Nf6 3. Nc3 Nc6
       5      -3      3     11522  1. ... e5 2. Qh5 d6 3. d3 Nc6
       5       1      3     13090  1. ... Nc6 2. Nc3 Nf6 3. Nf3 d5
       6     -21      3     14191  1. ... Nc6 2. Nc3 Nf6 3. Nf3 d5 4. d3
       7      -7     10     42915  1. ... Nc6 2. Nf3 Nf6 3. e5 Ng4 4. d4 d5
       8     -30     15     69004  1. ... Nc6 2. Nf3 Nf6 3. Nc3 e6 4. d4 d5 5. e5
       9     -18     41    203279  1. ... Nc6 2. d4 d5 3. exd5 Qxd5 4. Nf3 Qe4+ 5. Be3 e5
      10     -43     67    337609  1. ... Nc6 2. d4 e6 3. Nc3 d5 4. Nf3 dxe4 5. Nxe4 Nf6 6. Bg5
      10     -38    112    571330  1. ... d5 2. exd5 Qxd5 3. Nc3 Qe6+ 4. Ne2 Nf6 5. d4 Nc6 6. Bf4
      10     -21    135    689624  1. ... e5 2. Nc3 Nf6 3. Nf3 Nc6 4. Bb5 Qe7 5. d3 Qe6 6. a3
      11      -9    236   1203094  1. ... e5 2. c4 Bd6 3. Nf3 Nc6 4. Nc3 Ne7 5. d4 O-O 6. Be3 exd4
      12     -22    518   2721242  1. ... e5 2. Nf3 Nf6 3. Nc3 Nc6 4. d4 exd4 5. Nxd4 d5 6. exd5 Nxd5 7. Bc4
      13     -13    952   5228761  1. ... e5 2. Nf3 Nc6 3. Nc3 Nf6 4. d4 exd4 5. Nxd4 d5 6. exd5 Nxd5 7. Bc4 Qe7+
    # score:               -22
    # time:               9951 ms
    # nodes:           5495966 (5.52e5 nps)
    # tt size:          524288 (8 MB)
    #  - lower:         380565 (72.59 %)
    #  - upper:          52120 (9.94 %)
    #  - exact:            290 (0.06 %)
    # tt inserts:       906503
    # tt lookups:      3675353
    #  - miss:         1502646 (40.88 %)
    #  - hits:          448500 (12.20 %)
    #  - collisions:   1724207 (46.91 %)
    < move e7e5
    +---+---+---+---+---+---+---+---+
    | r | n | b | q | k | b | n | r |
    +---+---+---+---+---+---+---+---+
    | p | p | p | p |   | p | p | p |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   |   |   |   |   |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   | p |   |   |   |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   | P |   |   |   |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   |   |   |   |   |
    +---+---+---+---+---+---+---+---+
    | P | P | P | P |   | P | P | P |
    +---+---+---+---+---+---+---+---+
    | R | N | B | Q | K | B | N | R |
    +---+---+---+---+---+---+---+---+
    > help
    quit                      Exit this program
    help                      Display this screen
    play                      Search and play a move
    undo                      Undo the last move
    move <move>               Play <move> on the board
    show <feature>            Show <feature>
    hide <feature>            Hide <feature>
    time <moves> <time>       Set clock to <moves> in <time> (in seconds)
    setboard <fen>            Set the board to <fen>
    threads <number>          Set the <number> of threads
    perft                     Count the nodes at each depth
    perftsuite <epd>          Compare perft results to each position of <epd>
    testsuite <epd> [<time>]  Search each position of <epd> [for <time>]
    divide <depth>            Count the nodes at <depth> for each moves
    uci                       Start UCI mode
    xboard                    Start XBoard mode
    > quit


Test
----

Unit testing in Rust is wonderful. You can run the test suite directly
from Cargo:

    $ cargo test

Little Wing also have a `perft` command for counting the number of nodes at
each depth from the starting position.

    $ cargo run
    Little Wing v0.3.0

    > perft
    perft(1) -> 20 (0.00 s, 5.83e4 nps)
    perft(2) -> 400 (0.00 s, 7.62e5 nps)
    perft(3) -> 8902 (0.01 s, 7.44e5 nps)
    perft(4) -> 197281 (0.16 s, 1.26e6 nps)
    perft(5) -> 4865609 (3.82 s, 1.27e6 nps)


And a `perftsuite` command for comparing the results of a perft calculation
with the given EPD file.

    $ cargo run -- --color
    Little Wing v0.3.0

    > perftsuite tests/perftsuite.epd
    rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 -> ......
    r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 -> ......
    4k3/8/8/8/8/8/8/4K2R w K - 0 1 -> ......
    4k3/8/8/8/8/8/8/R3K3 w Q - 0 1 -> ......
    4k2r/8/8/8/8/8/8/4K3 w k - 0 1 -> ......
    r3k3/8/8/8/8/8/8/4K3 w q - 0 1 -> ......
    4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1 -> ......
    r3k2r/8/8/8/8/8/8/4K3 w kq - 0 1 -> ......
    8/8/8/8/8/8/6k1/4K2R w K - 0 1 -> ......
    8/8/8/8/8/8/1k6/R3K3 w Q - 0 1 -> ......
    4k2r/6K1/8/8/8/8/8/8 w k - 0 1 -> ......
    r3k3/1K6/8/8/8/8/8/8 w q - 0 1 -> ......


And the usual others like `divide`, `setboard` or `testsuite`:

    $ cargo run -- --color
    Little Wing v0.3.0

    > testsuite tests/wac.epd 1
    2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - bm Qg6 -> Qg6
    8/7p/5k2/5p2/p1p2P2/Pr1pPK2/1P1R3P/8 b - - bm Rxb2 -> c3
    5rk1/1ppb3p/p1pb4/6q1/3P1p1r/2P1R2P/PP1BQ1P1/5RKN w - - bm Rg3 -> Rg3
    r1bq2rk/pp3pbp/2p1p1pQ/7P/3P4/2PB1N2/PP3PPR/2KR4 w - - bm Qxh7+ -> Qxh7+
    5k2/6pp/p1qN4/1p1p4/3P4/2PKP2Q/PP3r2/3R4 b - - bm Qc4+ -> Qc4+
    7k/p7/1R5K/6r1/6p1/6P1/8/8 w - - bm Rb7 -> Rb7
    rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - bm Ne3 -> Ne3
    r4q1k/p2bR1rp/2p2Q1N/5p2/5p2/2P5/PP3PPP/R5K1 w - - bm Rf7 -> Rf7
    3q1rk1/p4pp1/2pb3p/3p4/6Pr/1PNQ4/P1PB1PP1/4RRK1 b - - bm Bh2+ -> Bh2+
    2br2k1/2q3rn/p2NppQ1/2p1P3/Pp5R/4P3/1P3PPP/3R2K1 w - - bm Rxh7 -> Rxh7
    r1b1kb1r/3q1ppp/pBp1pn2/8/Np3P2/5B2/PPP3PP/R2Q1RK1 w kq - bm Bxc6 -> Bxc6

Here we used `cargo run` to run the engine in debug mode, but you can invoke
it from `littlewing` if you installed it to make it run (much) faster.


License
-------

Copyright (c) 2014-2017 Vincent Ollivier. Released under GNU GPL License v3.
