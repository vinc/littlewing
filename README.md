Little Wing
===========

Someday this will be a chess engine written in Rust.


Usage
-----

    $ git clone https://github.com/vinc/littlewing.git
    $ cd littlewing
    $ cargo build --release
    $ sudo cp target/release/littlewing /usr/local/bin


Test
----

    $ cargo test

The program also have a `perft` command for counting the number of nodes at
each depth from the starting position.

    $ littlewing
    Little Wing v0.0.1

    > perft
    perft(1) -> 20 (0.00 s, 6.89e4 nps)
    perft(2) -> 400 (0.00 s, 4.30e5 nps)
    perft(3) -> 8908 (0.01 s, 6.41e5 nps)
    perft(4) -> 197830 (0.26 s, 7.75e5 nps)
    perft(5) -> 4923794 (6.34 s, 7.77e5 nps)

And a `perftsuite` command for comparing the results of a perft calculation
with the given EPD file.

    $ littlewing
    Little Wing v0.0.1

    > perftsuite tests/perftsuite.epd
    rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 -> ..x
    r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 -> x
    4k3/8/8/8/8/8/8/4K2R w K - 0 1 -> x
    4k3/8/8/8/8/8/8/R3K3 w Q - 0 1 -> x
    4k2r/8/8/8/8/8/8/4K3 w k - 0 1 -> .x

One can see that the moves generator is not yet bug free.


Bench
-----

    $ cargo bench
