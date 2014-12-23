Little Wing
===========

A chess engine written in Rust.


Usage
-----

    $ git clone https://github.com/vinc/littlewing.git
    $ cd littlewing
    $ rustc --crate-type=lib src/littlewing.rs
    $ rustc -L . -o littlewing src/main.rs
    $ ./littlewing


Test
----

    $ rustc -L . --test test/littlewing_test.rs
    $ ./littlewing_test
