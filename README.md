Little Wing
===========

A bitboard chess engine written in Rust.

A work in progress since December 2014.

Tested on GNU/Linux 32 and 64 bits, should run anywhere.


Usage
-----

First you need to install Rust:

    $ curl https://sh.rustup.rs -sSf | sh

Then you can compile and install the engine:

    $ git clone https://github.com/vinc/littlewing.git
    $ cd littlewing
    $ LITTLEWING_VERSION=$(git describe) cargo build --release
    $ sudo cp target/release/littlewing /usr/local/bin

Little Wing is compatible with the XBoard protocol, and it has its own
text-based user interface:

    $ littlewing --color --debug
    Little Wing v0.3.0

    > time 1 10
    > show think
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
    > play
    # FEN rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
    # allocating 10000 ms to move
     ply   score   time     nodes  pv
       1       0      0         3  1. ... a6
       2       0      0        56  1. ... a6 2. a3
       3       0      0       205  1. ... a6 2. a3 a5
       4       0      1      1821  1. ... a6 2. a3 a5 3. b3
       5       0     10      7970  1. ... a6 2. a3 a5 3. b3 a4
       6       0     31     59151  1. ... a6 2. a3 a5 3. b3 a4 4. bxa4
       7       0    404    377742  1. ... a6 2. a3 a5 3. b3 a4 4. bxa4 Rxa4
       8       0    930   2170288  1. ... a6 2. a3 a5 3. b3 a4 4. bxa4 Rxa4 5. d3
    # 9951 ms used in search
    # 2239407 nodes visited (2.25e5 nps)
    # tt size:       524288
    # tt inserts:    6371
    # tt lookups:    213735
    # tt hits:       14459
    # tt collisions: 1001
    move a7a6
    +---+---+---+---+---+---+---+---+
    | r | n | b | q | k | b | n | r |
    +---+---+---+---+---+---+---+---+
    |   | p | p | p | p | p | p | p |
    +---+---+---+---+---+---+---+---+
    | p |   |   |   |   |   |   |   |
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

As you can see, it's still pretty weak at the moment.


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
