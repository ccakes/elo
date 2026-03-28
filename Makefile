.PHONY: build build-cli build-macos build-linux test clean

build: build-cli build-macos build-linux

build-cli:
	mkdir -p build
	cargo build --release -p elo-cli
	cp target/release/elo-cli build/

build-macos:
	mkdir -p build
	cd elo-tauri && pnpm install && cd .. && cargo tauri build --bundles app
	cp -r target/release/bundle/macos/Elo.app build/
	cp target/release/bundle/dmg/*.dmg build/ 2>/dev/null || true

build-linux:
	mkdir -p build
	cd elo-tauri && pnpm install && cd .. && cargo tauri build --bundles deb,rpm
	cp -r target/release/bundle/deb/*.deb build/
	cp -r target/release/bundle/rpm/*.rpm build/
	cp -r target/release/bundle/appimage/*.AppImage build/ 2>/dev/null || true

test:
	cargo test --workspace

clean:
	rm -rf build
