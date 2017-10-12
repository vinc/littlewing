Little Wing
===========

[![Travis](https://img.shields.io/travis/vinc/littlewing.svg)]()
[![Crates.io](https://img.shields.io/crates/v/littlewing.svg)]()

A bitboard chess engine written in Rust.

A work in progress since December 2014.

- [x] XBoard protocol
- [x] Bitboard moves generation with De Bruijn sequence
- [x] Quiescence search
- [x] Principal variation search
- [x] MVV/LVA moves ordering by insertion sort
- [x] Staged moves generation
- [x] Static exchange evaluation
- [x] Transpositions table
- [x] Null move pruning
- [x] Internal iterative deepening
- [x] Futility pruning
- [x] Late move reduction
- [x] Killer heuristic
- [x] Mobility evaluation
- [x] Zobrist hashing
- [x] FEN support

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

Little Wing is compatible with the XBoard protocol, and has its own text-based
user interface:

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
    # FEN rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
    # allocating 10000 ms to move
     ply   score   time     nodes  pv
       1      -6      0         1  1. ... a6
       1      -5      0         3  1. ... c6
       1      -2      0         4  1. ... d6
       1       0      0         5  1. ... e6
       2     -10      0        51  1. ... e6 2. Qf3
       3      -1      0       484  1. ... e6 2. Qh5 Qf6
       3       0      0       944  1. ... e5 2. Qf3 Qf6
       4      -4      0      2259  1. ... e5 2. Qf3 Qf6 3. Nc3
       5       0      1     10414  1. ... e5 2. Qf3 Qf6 3. Nc3 Nc6
       6      -4      9     60123  1. ... e5 2. Qf3 Qf6 3. Nc3 Nc6 4. Bc4
       6      -3     14     94638  1. ... c6 2. Qh5 Nf6 3. Qe5 d6 4. Qd4
       7      -5     51    318377  1. ... c6 2. d4 Qa5+ 3. Qd2 Qxd2+ 4. Bxd2 Nf6
       7      -1     90    562244  1. ... d6 2. Qf3 Nf6 3. Bc4 Nc6 4. Qb3 e6
       8      -5    168   1129349  1. ... d6 2. Nc3 Nf6 3. d4 Nc6 4. Nf3 Be6 5. Bf4
       8      -2    249   1658850  1. ... e6 2. Nc3 Nc6 3. d4 Qf6
       8       0    292   1954015  1. ... d5 2. Qf3 dxe4 3. Qxe4 Nf6 4. Qa4+ Nc6 5. Nc3
       9       3    393   2721094  1. ... d5 2. d3 Nc6 3. Nc3 Nf6
      10       3    762   5572125  1. ... d5 2. exd5 Qxd5 3. Qf3 Nf6 4. Nc3 Qd4
      11       0    995   7245726  1. ... d5 2. exd5 Qxd5 3. Nc3 Qe6+ 4. Qe2 Nc6 5. Nf3 Nf6 6. Qe3 Qg4
    # 9951 ms used in search
    # 7245726 nodes visited (7.28e5 nps)
    # tt size:       524288
    # tt inserts:    1431
    # tt lookups:    1042442
    # tt hits:       1487
    # tt collisions: 1249
    move d7d5
    +---+---+---+---+---+---+---+---+
    | r | n | b | q | k | b | n | r |
    +---+---+---+---+---+---+---+---+
    | p | p | p |   | p | p | p | p |
    +---+---+---+---+---+---+---+---+
    |   |   |   |   |   |   |   |   |
    +---+---+---+---+---+---+---+---+
    |   |   |   | p |   |   |   |   |
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
    perft                     Count the nodes at each depth
    perftsuite <epd>          Compare perft results to each position of <epd>
    testsuite <epd> [<time>]  Search each position of <epd> [for <time>]
    divide <depth>            Count the nodes at <depth> for each moves
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
    rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - bm Ne3 -> Nxe5
    r4q1k/p2bR1rp/2p2Q1N/5p2/5p2/2P5/PP3PPP/R5K1 w - - bm Rf7 -> Rf7
    3q1rk1/p4pp1/2pb3p/3p4/6Pr/1PNQ4/P1PB1PP1/4RRK1 b - - bm Bh2+ -> f6
    2br2k1/2q3rn/p2NppQ1/2p1P3/Pp5R/4P3/1P3PPP/3R2K1 w - - bm Rxh7 -> Rxh7
    r1b1kb1r/3q1ppp/pBp1pn2/8/Np3P2/5B2/PPP3PP/R2Q1RK1 w kq - bm Bxc6 -> Bxc6

Here we used `cargo run` to run the engine in debug mode, but you can invoke
it from `littlewing` if you installed it to make it run faster.


License
-------

Copyright (C) 2014-2017 Vincent Ollivier. Released under GNU GPL License v3.
