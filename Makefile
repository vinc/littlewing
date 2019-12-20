.PHONY: setup build install clean cross
.EXPORT_ALL_VARIABLES:

PREFIX = /usr/local
LITTLEWING_VERSION = $(shell git describe)
build: RUSTFLAGS = -C target-cpu=native

build:
	cargo build --release

setup:
	curl https://sh.rustup.rs -sSf | sh
	rustup update

install:
	cp target/release/littlewing $(PREFIX)/bin

uninstall:
	rm -f $(PREFIX)/bin/littlewing

clean:
	cargo clean

release:
	mkdir -p release
	cp README.md release/README.txt
	cp LICENSE release/LICENSE.txt
	cp CHANGELOG.md release/CHANGELOG.txt
	cross build --release --target x86_64-unknown-linux-gnu
	cross build --release --target x86_64-pc-windows-gnu
	cross build --release --target armv7-linux-androideabi
	cp target/x86_64-unknown-linux-gnu/release/littlewing release/littlewing-$(LITTLEWING_VERSION)-linux-x86
	cp target/armv7-linux-androideabi/release/littlewing release/littlewing-$(LITTLEWING_VERSION)-android-armv7
	cp target/x86_64-pc-windows-gnu/release/littlewing.exe release/littlewing-$(LITTLEWING_VERSION)-windows-x86.exe
	gzip release/littlewing-*
	cd release && shasum littlewing-* > shasums.txt
