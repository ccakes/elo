.PHONY: build build-cli build-macos build-linux test clean

build: build-cli build-macos build-linux

build-cli:
	mkdir -p build
	cargo build --release -p elo-cli
	cp target/release/elo-cli build/

build-macos:
	mkdir -p build
	cd elo-tauri && pnpm install && cargo tauri build --bundles app
	cp -r target/release/bundle/macos/Elo.app build/

build-linux:
	mkdir -p build
	cd elo-tauri && pnpm install && cargo tauri build --bundles deb,appimage
	cp -r target/release/bundle/deb build/
	cp -r target/release/bundle/appimage build/

test:
	cargo test --workspace

clean:
	rm -rf build
