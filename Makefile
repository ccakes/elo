.PHONY: setup build build-cli build-macos build-linux test clean

# Enable the repo's version-controlled git hooks (e.g. pre-push tag/version check).
setup:
	git config core.hooksPath .githooks
	@echo "git hooks enabled (core.hooksPath=.githooks)"

build: build-cli build-macos build-linux

build-cli:
	mkdir -p build
	cargo build --release -p elo-cli
	cp target/release/elo-cli build/

build-macos:
	mkdir -p build
	cd elo-tauri && pnpm install && cd .. && cargo tauri build --bundles dmg --target universal-apple-darwin
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
