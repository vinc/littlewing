Little Wing: a chess engine written in Rust
===========================================

[![Travis](https://img.shields.io/travis/vinc/littlewing/master.svg)](https://travis-ci.org/vinc/littlewing/branches)
[![Crates.io](https://img.shields.io/crates/v/littlewing.svg)](https://crates.io/crates/littlewing)

Little Wing is the successor of [Purple Haze](https://github.com/vinc/purplehaze).

The project started in December 2014 to learn the Rust language and play with
bitboards. Both experiments were conclusive and Little Wing it is still
maintained in 2022.

Currently evaluated at 2000+ ELO on CCRL 40/4 Rating List.

[![asciicast](https://asciinema.org/a/146112.png)](https://asciinema.org/a/146112)


Features
--------

- Interfaces
  - A nice CLI with many commands
  - XBoard and UCI communication protocols
  - Public API with [documented library](https://docs.rs/littlewing)
- Board representation
  - Bitboard with LLVM CTPOP and CTTZ
  - Sliding piece attacks with Hyperbola Quintessence and First Rank Attacks
  - Zobrist hashing with Xorshift RNG
  - Staged moves generation
  - MVV/LVA and SEE moves ordering with insertion sort
  - FEN support
- Search
  - Principal variation search
  - Quiescence search
  - Transposition table
  - Null move pruning
  - Internal iterative deepening
  - Futility pruning
  - Late move reduction
  - Killer heuristic
- Evaluation
  - Piece square table evaluation
  - Mobility evaluation
  - Static exchange evaluation


Install
-------

Binaries for GNU/Linux (x86-64), Android (ARMv7), and Windows (x86-64) are
available: https://vinc.cc/binaries

If you want to compile Little Wing yourself, you must first install Rust:

    $ curl https://sh.rustup.rs -sSf | sh

Then you can install the latest stable version of the engine with Cargo:

    $ cargo install littlewing

Or the development version by fetching the git repository:

    $ git clone https://github.com/vinc/littlewing.git
    $ cd littlewing
    $ export RUSTFLAGS="-C target-cpu=native"
    $ export LITTLEWING_VERSION="$(git describe)"
    $ cargo build --release
    $ sudo cp target/release/littlewing /usr/local/bin


Usage
-----

Little Wing is compatible with XBoard and UCI communication protocols,
and in addition it has its own text-based user interface:

    $ littlewing
                                          _,;
                   ,       .--.       _,-'.-;
                    \`-, <) o  `._ ,-' ,'`_7
                    <_  `-\ _       _,' _.'
                      <_`".| `\    `  _.>
                        <_ ;   \     _>
                         `"     ;  ``
      Little Wing v0.7.0    \   |   \
                             '|-. _  \
      by Vincent Ollivier  _/ /     \ '.
      +---+---+---+---+---+"-"`---+--'\_>
      | r | n | b | q | k | b | n | r | 8
      +---+---+---+---+---+---+---+---+
      | p | p | p | p | p | p | p | p | 7
      +---+---+---+---+---+---+---+---+
      |   |   |   |   |   |   |   |   | 6
      +---+---+---+---+---+---+---+---+
      |   |   |   |   |   |   |   |   | 5
      +---+---+---+---+---+---+---+---+
      |   |   |   |   |   |   |   |   | 4
      +---+---+---+---+---+---+---+---+
      |   |   |   |   |   |   |   |   | 3
      +---+---+---+---+---+---+---+---+
      | P | P | P | P | P | P | P | P | 2
      +---+---+---+---+---+---+---+---+
      | R | N | B | Q | K | B | N | R | 1
      +---+---+---+---+---+---+---+---+
        a   b   c   d   e   f   g   h

    > move e4

      +---+---+---+---+---+---+---+---+
      | r | n | b | q | k | b | n | r | 8
      +---+---+---+---+---+---+---+---+
      | p | p | p | p | p | p | p | p | 7
      +---+---+---+---+---+---+---+---+
      |   |   |   |   |   |   |   |   | 6
      +---+---+---+---+---+---+---+---+
      |   |   |   |   |   |   |   |   | 5
      +---+---+---+---+---+---+---+---+
      |   |   |   |   | P |   |   |   | 4
      +---+---+---+---+---+---+---+---+
      |   |   |   |   |   |   |   |   | 3
      +---+---+---+---+---+---+---+---+
      | P | P | P | P |   | P | P | P | 2
      +---+---+---+---+---+---+---+---+
      | R | N | B | Q | K | B | N | R | 1
      +---+---+---+---+---+---+---+---+
        a   b   c   d   e   f   g   h

    > time 1 10
    > show think
    > play black

      ply  score    time      nodes  pv
        1    -46       0          1  1. ... a6
        1    -45       0          3  1. ... c6
        1    -22       0          4  1. ... d6
        1    -20       0          5  1. ... e6
        1     -1       0         14  1. ... d5
        1      0       0         15  1. ... e5
        1      9       0         20  1. ... Nc6
        2    -47       0         53  1. ... Nc6 2. Nc3
        3      9       0        278  1. ... Nc6 2. Nc3 Nf6
        4    -45       0        860  1. ... Nc6 2. Nc3 Nf6 3. Nf3
        4    -31       1       2437  1. ... d5 2. exd5 Qxd5 3. Nc3
        5    -32       1       4647  1. ... d5 2. exd5 Qxd5 3. Nc3 Qd4
        5    -20       2       7442  1. ... d6 2. Nc3 Nf6 3. Nf3 Nc6
        5      0       2      11381  1. ... e5 2. Nc3 Nf6 3. Nf3 Nc6
        5      1       2      12946  1. ... Nc6 2. Nc3 Nf6 3. Nf3 d5
        6    -21       3      14052  1. ... Nc6 2. Nc3 Nf6 3. Nf3 d5 4. d3
        7     -7       6      34813  1. ... Nc6 2. Nf3 Nf6 3. e5 Ng4 4. d4 d5
        8    -29       9      53113  1. ... Nc6 2. Nf3 Nf6 3. e5 Ng4 4. d4 d6
                                     5. h3
        9    -25      27     181431  1. ... Nc6 2. d4 d5 3. e5 e6 4. Nc3 Bb4
                                     5. Nf3 Nge7
        9    -12      35     241217  1. ... e5 2. Nc3 Nf6 3. Nf3 Nc6 4. d4 exd4
                                     5. Nxd4 d5
       10    -10      50     363028  1. ... e5 2. Nc3 Nf6 3. Nf3 Nc6 4. d4 exd4
                                     5. Nxd4 d5 6. f3
       11     -7     109     815399  1. ... e5 2. Nf3 Nf6 3. Nxe5 d6 4. Nc4 Nxe4
                                     5. d3 Nf6 6. Nc3 Nc6
       12    -23     218    1686098  1. ... e5 2. Nf3 Nf6 3. Nc3 Nc6 4. d4 exd4
                                     5. Nxd4 d5 6. exd5 Nxd5 7. Bc4
       13    -13     467    3791367  1. ... e5 2. Nf3 Nf6 3. Nc3 Nc6 4. d4 exd4
                                     5. Nxd4 d5 6. exd5 Nxd5 7. Bc4

    < move e5

      +---+---+---+---+---+---+---+---+
      | r | n | b | q | k | b | n | r | 8
      +---+---+---+---+---+---+---+---+
      | p | p | p | p |   | p | p | p | 7
      +---+---+---+---+---+---+---+---+
      |   |   |   |   |   |   |   |   | 6
      +---+---+---+---+---+---+---+---+
      |   |   |   |   | p |   |   |   | 5
      +---+---+---+---+---+---+---+---+
      |   |   |   |   | P |   |   |   | 4
      +---+---+---+---+---+---+---+---+
      |   |   |   |   |   |   |   |   | 3
      +---+---+---+---+---+---+---+---+
      | P | P | P | P |   | P | P | P | 2
      +---+---+---+---+---+---+---+---+
      | R | N | B | Q | K | B | N | R | 1
      +---+---+---+---+---+---+---+---+
        a   b   c   d   e   f   g   h

    > help

    Commands:

      quit                      Exit this program
      help                      Display this screen
      load [<options>]          Load game from <options>
      save [<options>]          Save game to <options>
      hint                      Search the best move
      play [<color>]            Search and play [<color>] move[s]
      undo                      Undo the last move
      move <move>               Play <move> on the board

      show <feature>            Show <feature>
      hide <feature>            Hide <feature>
      time <moves> <time>       Set clock to <moves> in <time> (in seconds)
      hash <size>               Set the <size> of the memory (in MB)
      core <number>             Set the <number> of threads

      perft [<depth>]           Count the nodes at each depth
      perftsuite <epd>          Compare perft results to each position of <epd>
      testsuite <epd> [<time>]  Search each position of <epd> [for <time>]
      divide <depth>            Count the nodes at <depth> for each moves

      uci                       Start UCI mode
      xboard                    Start XBoard mode

    Made with <3 in 2014-2022 by Vincent Ollivier <v@vinc.cc>

    Report bugs to https://github.com/vinc/littlewing/issues

    > quit


Tests
-----

Run the test suite with Cargo:

    $ cargo test

Little Wing also have a `perft` command for counting the number of nodes at
each depth from the starting position.

    $ cargo run -- --silent
    > perft
    perft 1 -> 20 (0.00 s, 5.83e4 nps)
    perft 2 -> 400 (0.00 s, 7.62e5 nps)
    perft 3 -> 8902 (0.01 s, 7.44e5 nps)
    perft 4 -> 197281 (0.16 s, 1.26e6 nps)
    perft 5 -> 4865609 (3.82 s, 1.27e6 nps)

And a `perftsuite` command for comparing the results of a perft calculation
with the given EPD file.

    $ cargo run -- --silent
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

And the usual debug commands like `divide` or `testsuite`:

    $ cargo run -- --silent
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

The CLI as been designed to interface nicely with other Unix tools, for example
to test if the times from perft are stable you can do:

    $ for i in $(seq 0 10); do littlewing -s <<< "perft 6"; sleep 1; done

Anyway, have fun with it and send me your feedback at <v@vinc.cc>!


License
-------

Little Wing is released under MIT.
