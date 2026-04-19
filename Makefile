.PHONY: build build-cli build-macos build-linux test clean

build: build-cli build-macos build-linux

build-cli:
	mkdir -p build
	cargo build --release -p elo-cli
	cp target/release/elo-cli build/

build-macos:
	mkdir -p build
	rustup target add aarch64-apple-darwin x86_64-apple-darwin
	cd elo-tauri && pnpm install && cd .. && cargo tauri build --target universal-apple-darwin --bundles app,dmg
	cp -r target/universal-apple-darwin/release/bundle/macos/Elo.app build/
	cp target/universal-apple-darwin/release/bundle/dmg/*.dmg build/

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
