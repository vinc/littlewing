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

    $ cargo run
    Little Wing v0.0.1

    > perft
    perft(1) -> 20 (0.00 s, 5.83e4 nps)
    perft(2) -> 400 (0.00 s, 7.62e5 nps)
    perft(3) -> 8902 (0.01 s, 7.44e5 nps)
    perft(4) -> 197281 (0.16 s, 1.26e6 nps)
    perft(5) -> 4865609 (3.82 s, 1.27e6 nps)


And a `perftsuite` command for comparing the results of a perft calculation
with the given EPD file.

    $ cargo run
    Little Wing v0.0.1

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


Bench
-----

    $ cargo bench
