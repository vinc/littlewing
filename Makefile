.PHONY: setup build install clean
.EXPORT_ALL_VARIABLES:

PREFIX = /usr/local
LITTLEWING_VERSION = $(shell git describe)

build:
	RUSTFLAGS="-C target-cpu=native" cargo build --release

setup:
	curl https://sh.rustup.rs -sSf | sh
	rustup update

install:
	cp target/release/littlewing $(PREFIX)/bin

uninstall:
	rm -f $(PREFIX)/bin/littlewing

clean:
	cargo clean
