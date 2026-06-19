# Elo iOS Port — Implementation Guide

This document is a complete, step-by-step plan for porting **Elo** (the Tauri
desktop calculator) to **iOS**. It is written to be executed on a macOS machine
with Xcode, where the build can be run and tested in the iOS Simulator / on a
device as you go.

The work is divided into independently verifiable phases. Do them in order —
each phase ends in a state you can build and run.

---

## 0. Context: how Elo is wired today

Read this before touching anything; the layout work in Phase 3 depends on it.

- **Workspace** — Rust 2024 workspace. The Tauri crate lives at
  `crates/elo-tauri` (the JS/Vite frontend lives separately at `elo-tauri/`).
  Note the split: `crates/elo-tauri/tauri.conf.json` points
  `frontendDist` at `../../elo-tauri/dist`.
- **Tauri v2** already. Both telltale mobile hooks are present:
  - `crates/elo-tauri/Cargo.toml` declares
    `crate-type = ["staticlib", "cdylib", "rlib"]` (iOS needs the static/cdylib).
  - `crates/elo-tauri/src/lib.rs` has
    `#[cfg_attr(mobile, tauri::mobile_entry_point)]` on `run()`.
- **Rust commands** (`src/lib.rs`): `evaluate_document`, `evaluate_line`,
  `reset_session`. These are pure compute over `elo-core` and port to iOS
  unchanged.
- **Frontend** (`elo-tauri/`):
  - `index.html` — toolbar (New/Open/Save/Export/Copy Results), an
    editor area, a results gutter, a status bar.
  - `src/main.ts` — a single `<textarea id="editor">` holds **all** lines.
    A `<pre id="editor-highlight">` behind it renders syntax highlighting,
    kept pixel-aligned with the textarea (same font, padding, `line-height:
    1.6`, `white-space: pre-wrap`). A separate `<div id="results">` (a fixed
    **260px right gutter**) renders one right-aligned `.result-line`
    (height `22.4px` = `14px × 1.6`) per document line. Scroll is synced
    across all three. **Understanding this dual-pane/overlay model is critical:
    a `<textarea>` cannot interleave non-text rows, which is the central
    constraint for the portrait layout in Phase 3.**
  - File I/O uses `@tauri-apps/plugin-dialog` + `@tauri-apps/plugin-fs`
    (`openDocument`/`saveDocument`/`exportDocument`) with absolute filesystem
    paths — a desktop assumption that does not hold on sandboxed iOS.
- **Desktop-only Rust features currently always-on**:
  - `tauri = { features = ["tray-icon"] }` — no system tray on iOS.
  - `tauri-plugin-global-shortcut` — no global hotkeys on iOS.
  (Both are declared/registered but lightly used: there is no tray code in
  `lib.rs`, and the global-shortcut plugin is initialised but registers no
  shortcuts from Rust; the JS uses plain `keydown` handlers. So gating them is
  low-risk.)

### Goals for this port

1. Produce a buildable, runnable iOS app from the existing Tauri project.
2. **Portrait reflow** — on the narrow portrait viewport, the input line uses
   the full width (extending across where the desktop result gutter sits) and
   each line's result drops *below* the line instead of sitting in a side
   column.
3. **iCloud** — create an app-specific Documents folder in the user's iCloud
   and support saving/loading Elo notes there.
4. **Feature-gate** tray-icon and global-shortcut so the mobile build compiles
   and the desktop build is unchanged.

---

## 1. Prerequisites (macOS)

These cannot be done on Linux — iOS toolchains are macOS-only.

```sh
# Xcode (full app, not just CLT) + license + simulators
xcode-select --install         # if needed
sudo xcodebuild -license accept
xcodebuild -runFirstLaunch

# Rust iOS targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
#   aarch64-apple-ios      → physical devices
#   aarch64-apple-ios-sim  → Apple-silicon simulator
#   x86_64-apple-ios       → Intel simulator (optional)

# CocoaPods (cargo-mobile2 uses it for the generated Xcode project)
brew install cocoapods

# Tauri CLI v2 — the repo already depends on @tauri-apps/cli ^2 in
# elo-tauri/package.json, so `pnpm tauri` works. A global install also works:
cargo install tauri-cli --version '^2.0.0' --locked
```

You will also need, for anything beyond the Simulator:

- An **Apple Developer account** (free account works for Simulator + limited
  on-device; **iCloud requires a paid account** to create a container).
- A **development team ID** for code signing.

> **Important — iCloud needs a paid Apple Developer account.** The iCloud
> container and entitlements in Phase 4 cannot be provisioned on a free
> account. The Simulator can exercise the *code paths*, but a real signed
> build is required to validate end-to-end sync.

---

## 2. Phase 1 — Feature-gate desktop-only deps (`Cargo.toml` + `lib.rs`)

Do this first: it is required for the mobile target to compile, and it is a
pure refactor that leaves desktop behaviour identical.

### 2.1 `crates/elo-tauri/Cargo.toml`

Move `tray-icon` and `global-shortcut` into a **desktop-only** target table.
Cargo merges features additively, so the base `tauri` dep stays minimal and the
desktop target adds `tray-icon`.

```toml
[dependencies]
elo-core = { path = "../elo-core" }
tauri = { version = "2" }                  # base, no tray-icon
tauri-plugin-opener = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Desktop-only: tray + global shortcuts (no equivalent on iOS/Android)
[target.'cfg(not(any(target_os = "ios", target_os = "android")))'.dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2"

# iOS-only deps used by the iCloud bridge (Phase 4)
[target.'cfg(target_os = "ios")'.dependencies]
objc2 = "0.5"
objc2-foundation = { version = "0.2", features = [
  "NSFileManager", "NSURL", "NSString", "NSArray", "NSError",
  "NSFileCoordinator",
] }
```

> Note: in `Cargo.toml` target tables you must use **real built-in cfgs**
> (`target_os`), *not* Tauri's `desktop`/`mobile` cfgs — those are emitted by a
> build script and are only available in Rust source, not to Cargo's manifest
> evaluation.

### 2.2 `crates/elo-tauri/src/lib.rs`

In Rust source you *can* use Tauri's `desktop`/`mobile` cfgs. Gate the
global-shortcut plugin registration so it only compiles/loads on desktop. The
idiomatic Tauri v2 pattern registers it inside `.setup()`:

```rust
pub fn run() {
    let rates = RateStore::load();
    let builder = tauri::Builder::default()
        .manage(AppState {
            session: Mutex::new(Session::with_rates(rates.clone())),
            rates,
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init());

    let builder = builder.setup(|_app| {
        #[cfg(desktop)]
        {
            _app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new().build(),
            )?;
        }
        Ok(())
    });

    builder
        .invoke_handler(tauri::generate_handler![
            evaluate_document,
            evaluate_line,
            reset_session,
            // iCloud commands added in Phase 4:
            // icloud_documents_dir,
            // icloud_list_documents,
            // icloud_read_document,
            // icloud_write_document,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

If any `tray-icon`-specific code exists elsewhere, wrap it in `#[cfg(desktop)]`
too. (Currently there is none.)

### 2.3 Verify desktop still builds

```sh
cargo build -p elo-tauri
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

---

## 3. Phase 2 — Initialize the iOS project

```sh
cd /path/to/elo
# Build the web assets at least once so frontendDist exists:
cd elo-tauri && pnpm install && pnpm build && cd ..

# Generate the Xcode project under crates/elo-tauri/gen/apple
pnpm --dir elo-tauri tauri ios init
#   (or: cargo tauri ios init  — run from repo root)
```

What this creates: `crates/elo-tauri/gen/apple/` containing an Xcode project,
an `Info.plist`, an `<App>_iOS.entitlements` file, asset catalogs, and the
`project.yml` cargo-mobile2 uses to regenerate. The app identifier comes from
`tauri.conf.json` → `identifier` (`com.elo.calculator`).

Run it in the Simulator to confirm the baseline works **before** the layout and
iCloud work:

```sh
pnpm --dir elo-tauri tauri ios dev
# pick a simulator when prompted, e.g. "iPhone 15 Pro"
```

At this point the existing desktop UI will render in the Simulator (cramped,
desktop layout). That is expected — Phase 3 fixes it.

> **Source control for `gen/apple`.** cargo-mobile2 can regenerate this
> directory. Tauri's generated `.gitignore` ignores most of it. You **must**
> commit the files we hand-edit so they survive (`Info.plist`, the
> `.entitlements`, and any `project.yml` change) — force-add them if ignored:
> `git add -f crates/elo-tauri/gen/apple/elo_iOS/Info.plist
> crates/elo-tauri/gen/apple/elo_iOS/elo_iOS.entitlements`. Re-running
> `tauri ios init` later can overwrite these — re-apply Phase 4 edits if so.

### Mobile capabilities

The current `crates/elo-tauri/capabilities/default.json` targets the desktop
schema and includes `global-shortcut:allow-register`, which is invalid on
mobile. Split capabilities by platform:

- Restrict the existing file to desktop by adding a `platforms` key:

  ```jsonc
  // capabilities/default.json
  {
    "$schema": "../gen/schemas/desktop-schema.json",
    "identifier": "default",
    "description": "Capability for the main desktop window",
    "platforms": ["macOS", "windows", "linux"],
    "windows": ["main"],
    "permissions": [
      "core:default", "opener:default",
      "dialog:default", "fs:default",
      "fs:allow-read-text-file", "fs:allow-write-text-file",
      "global-shortcut:allow-register"
    ]
  }
  ```

- Add a mobile capability without global-shortcut:

  ```jsonc
  // capabilities/mobile.json
  {
    "$schema": "../gen/schemas/mobile-schema.json",
    "identifier": "mobile",
    "description": "Capability for the iOS/Android app",
    "platforms": ["iOS", "android"],
    "windows": ["main"],
    "permissions": [
      "core:default", "opener:default",
      "dialog:default", "fs:default",
      "fs:allow-read-text-file", "fs:allow-write-text-file"
    ]
  }
  ```

(The `mobile-schema.json` is generated into `gen/schemas/` after the first
mobile build. If your editor flags the `$schema` before then, build once.)

---

## 4. Phase 3 — Responsive & portrait layout

### 4.1 The requirement

Desktop / landscape keeps today's look: editor on the left, results in the
right gutter. **Portrait** (narrow): the input line uses the full width, and
each line's result is shown **below** that line rather than in a side column —
"the line extends over where the result was, and the result moves down".

### 4.2 The architectural constraint (read this)

Today all lines live in **one `<textarea>`**. You cannot interleave
non-editable result rows *between* text rows inside a textarea, and you cannot
make the textarea reflow around floating elements. So a result placed "below a
line" must either (a) float as an overlay that does not reflow text, or (b)
require moving away from the single-textarea model. Two viable designs follow;
**implement Option A first** (lowest risk, fully reuses the current evaluation
pipeline) and treat Option B as a later enhancement.

#### Option A — Measured floating-result overlay (recommended for v1)

Keep the textarea + highlight overlay exactly as-is for input. Change only how
results are positioned in portrait:

1. **Make highlight lines measurable.** In `highlightMarkdown()` wrap each
   line's content in a span that carries its index:

   ```ts
   // instead of result.push(html), push:
   result.push(`<span class="hl-line" data-line="${i}">${html}</span>`);
   ```

   Because `#editor-highlight` wraps text identically to the textarea
   (`pre-wrap`, same width/font/padding), each `.hl-line` span's
   `offsetTop`/`offsetHeight` is the *true rendered position* of that logical
   line, including any soft-wrapping. This is the key measurement primitive.

2. **Portrait mode toggle.** Drive layout from orientation:

   ```css
   @media (orientation: portrait) and (max-width: 600px) {
     #results { display: none; }          /* hide the fixed gutter */
     #editor-wrapper { flex: 1 1 100%; }  /* editor takes full width  */
     #results.portrait { display: block; position: absolute; inset: 0;
                          pointer-events: none; /* taps fall through except on chips */ }
     .result-line.portrait-chip {
       position: absolute; right: 12px; pointer-events: auto;
       max-width: 60%; height: 22.4px; text-align: right;
     }
   }
   ```

3. **Position each result under the last visual row of its line.** After every
   `renderResults()` + `updateHighlight()`, in portrait mode:

   ```ts
   function layoutPortraitResults() {
     const spans = highlightEl.querySelectorAll<HTMLElement>('.hl-line');
     const chips = resultsEl.querySelectorAll<HTMLElement>('.result-line');
     spans.forEach((span, i) => {
       const chip = chips[i];
       if (!chip) return;
       const top = span.offsetTop - editor.scrollTop;
       const lineBottom = span.offsetTop + span.offsetHeight;
       // crude collision test: does the line's last row leave room on the right?
       const lastRowText = /* measure trailing text width via a hidden canvas */;
       const crowded = lastRowText > span.clientWidth * 0.5;
       chip.classList.add('portrait-chip');
       // if crowded, drop the result onto its own row beneath the line
       chip.style.top = (crowded ? lineBottom : span.offsetTop) - editor.scrollTop + 'px';
     });
   }
   ```

   Recompute on `input`, on `scroll` (cheap — just re-offset by `scrollTop`),
   and on `resize`/`orientationchange`.

   **Known limitation to tune in the Simulator:** a result dropped below a long
   line floats over the *next* line (the textarea doesn't reflow to make room).
   Mitigations, in order of preference: (i) only drop the result below when the
   next logical line is blank; (ii) right-align the result on the line's last
   wrapped row when it fits, dropping it under only when it truly doesn't; (iii)
   render the result as a faint trailing chip. This visual tuning is exactly
   what the Mac session should iterate on with the Simulator open.

#### Option B — Line-based interleaved editor (best UX, larger change)

Replace the single textarea with a list of per-line rows, each row = an
editable input + its result, stacked in portrait and side-by-side in landscape.
This reflows naturally (results genuinely push content down) and matches apps
like Soulver. Cost: re-implementing caret/selection/multiline editing, the
markdown highlight overlay per row, paste handling, and keyboard navigation —
a substantial rewrite of `main.ts`. Defer unless Option A's overlap tuning
proves unacceptable. If pursued, keep the Rust `evaluate_document` contract;
only the frontend changes.

> **Decision point for the Mac session:** build Option A, evaluate it in the
> Simulator against real notes, and only escalate to Option B if the floating
> overlay can't be made to look right. Flag back to the user before committing
> to Option B (it's a big rewrite).

### 4.3 Viewport, safe areas, and touch

`index.html` — opt into the safe-area model:

```html
<meta name="viewport"
      content="width=device-width, initial-scale=1.0, viewport-fit=cover, maximum-scale=1.0" />
```

`src/styles.css` — pad the chrome by the safe-area insets so the toolbar/status
bar clear the notch and home indicator:

```css
#toolbar   { padding-top: env(safe-area-inset-top); }
#statusbar { padding-bottom: env(safe-area-inset-bottom); }
#app       { padding-left: env(safe-area-inset-left);
             padding-right: env(safe-area-inset-right); }
```

Touch/input niceties on the textarea (`index.html`):

```html
<textarea id="editor" spellcheck="false"
          autocapitalize="off" autocorrect="off" autocomplete="off"
          inputmode="text" ...></textarea>
```

Also: bump toolbar button tap targets to ≥ 44×44 pt in portrait, remove
`-webkit-app-region: drag` effects on mobile (harmless but pointless), and
verify the on-screen keyboard appearing doesn't hide the active line (listen to
`visualViewport` resize and scroll the caret into view).

---

## 5. Phase 4 — iCloud Documents

Goal: an app-owned folder in iCloud Drive (visible in the Files app as "Elo"),
with save/load from inside the app.

### 5.1 Provision the iCloud container (Apple Developer portal + Xcode)

1. In the Apple Developer portal, under your App ID `com.elo.calculator`,
   enable **iCloud** and create an **iCloud container**
   `iCloud.com.elo.calculator`.
2. Open the generated Xcode project
   (`crates/elo-tauri/gen/apple/*.xcodeproj`), select the `_iOS` target →
   **Signing & Capabilities** → set your Team → **+ Capability → iCloud** →
   check **iCloud Documents** → add the container
   `iCloud.com.elo.calculator`. Xcode writes the entitlements file.

The resulting `crates/elo-tauri/gen/apple/elo_iOS/elo_iOS.entitlements` should
contain:

```xml
<key>com.apple.developer.icloud-container-identifiers</key>
<array><string>iCloud.com.elo.calculator</string></array>
<key>com.apple.developer.icloud-services</key>
<array><string>CloudDocuments</string></array>
<key>com.apple.developer.ubiquity-container-identifiers</key>
<array><string>iCloud.com.elo.calculator</string></array>
```

### 5.2 Expose the folder in the Files app (`Info.plist`)

Add to `crates/elo-tauri/gen/apple/elo_iOS/Info.plist` so the container's
Documents folder is user-visible and named:

```xml
<key>NSUbiquitousContainers</key>
<dict>
  <key>iCloud.com.elo.calculator</key>
  <dict>
    <key>NSUbiquitousContainerIsDocumentScopePublic</key><true/>
    <key>NSUbiquitousContainerName</key><string>Elo</string>
    <key>NSUbiquitousContainerSupportedFolderLevels</key><string>Any</string>
  </dict>
</dict>
```

> **Gotcha:** changes to `NSUbiquitousContainers` only take effect when
> **`CFBundleVersion` is incremented**. Bump the build number whenever you
> change this dict, or the Files-app entry won't update.

### 5.3 Rust bridge to the ubiquity container (`src/lib.rs`)

`tauri-plugin-fs` can't reach the iCloud ubiquity container directly, so add
iOS-only commands that resolve the container URL via Foundation and do the I/O.
Uses the `objc2`/`objc2-foundation` deps added in Phase 1.

```rust
#[cfg(target_os = "ios")]
mod icloud {
    use objc2_foundation::{NSFileManager, NSString, NSURL};
    use std::path::PathBuf;

    const CONTAINER: &str = "iCloud.com.elo.calculator";

    /// Resolve <ubiquity-container>/Documents, creating it if needed.
    /// NOTE: URLForUbiquityContainerIdentifier may block — call off the main
    /// thread (these commands run on Tauri's async runtime, which is fine).
    pub fn documents_dir() -> Result<PathBuf, String> {
        unsafe {
            let fm = NSFileManager::defaultManager();
            let ident = NSString::from_str(CONTAINER);
            let url: Option<objc2::rc::Retained<NSURL>> =
                fm.URLForUbiquityContainerIdentifier(Some(&ident));
            let base = url.ok_or("iCloud is not available (not signed in?)")?;
            let docs = base.URLByAppendingPathComponent(&NSString::from_str("Documents"))
                .ok_or("could not derive Documents URL")?;
            let path = docs.path().ok_or("no filesystem path")?.to_string();
            let pb = PathBuf::from(path);
            std::fs::create_dir_all(&pb).map_err(|e| e.to_string())?;
            Ok(pb)
        }
    }
}

#[cfg(target_os = "ios")]
#[tauri::command]
fn icloud_documents_dir() -> Result<String, String> {
    Ok(icloud::documents_dir()?.to_string_lossy().into_owned())
}

#[cfg(target_os = "ios")]
#[tauri::command]
fn icloud_list_documents() -> Result<Vec<String>, String> {
    let dir = icloud::documents_dir()?;
    let mut names = vec![];
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        // iCloud may show not-yet-downloaded files as ".<name>.icloud" stubs.
        let clean = name.strip_prefix('.').and_then(|n| n.strip_suffix(".icloud"))
            .map(str::to_owned).unwrap_or(name);
        if clean.ends_with(".elo") || clean.ends_with(".txt") || clean.ends_with(".md") {
            names.push(clean);
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}

#[cfg(target_os = "ios")]
#[tauri::command]
fn icloud_read_document(name: String) -> Result<String, String> {
    let path = icloud::documents_dir()?.join(&name);
    // If the file is an undownloaded placeholder, request download first.
    // For v1, std::fs::read_to_string works once the item is materialised;
    // wire startDownloadingUbiquitousItemAtURL + NSFileCoordinator if you hit
    // placeholder reads in testing (see notes below).
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[cfg(target_os = "ios")]
#[tauri::command]
fn icloud_write_document(name: String, contents: String) -> Result<(), String> {
    let path = icloud::documents_dir()?.join(&name);
    std::fs::write(&path, contents).map_err(|e| e.to_string())
}
```

Register the commands in `generate_handler!` (guard with `#[cfg(target_os =
"ios")]` or list them unconditionally behind cfg'd no-op stubs for the other
platforms so the `invoke_handler` array stays consistent — simplest is to build
the handler list with `#[cfg]` attributes inside the macro).

> **iCloud download coordination (deepen if testing surfaces it).** Files that
> exist in iCloud but aren't downloaded locally appear as `*.icloud`
> placeholders. Robust access uses
> `NSFileManager.startDownloadingUbiquitousItemAtURL:error:` then reads under an
> `NSFileCoordinator` (and writes under a coordinator too, to avoid conflicts).
> `objc2-foundation`'s `NSFileCoordinator` feature is enabled in Phase 1 for
> this. For v1 you can ship plain `std::fs` and add coordination once real-world
> placeholder/conflict cases appear in Simulator/device testing.

### 5.4 Frontend wiring (`src/main.ts`)

On iOS, route Open/Save through the iCloud commands and an in-app document
list, instead of the desktop file dialog. Detect platform via
`import { platform } from '@tauri-apps/plugin-os'` (add the `os` plugin) or by
feature-testing for the iCloud command.

- **Save** → if no current name, prompt for one (simple modal), then
  `invoke('icloud_write_document', { name, contents: editor.value })`.
- **Open** → `invoke('icloud_list_documents')` → render a tap list (reuse the
  toast/modal styling) → on tap `invoke('icloud_read_document', { name })` →
  load into the editor.
- Keep the existing desktop dialog path under the desktop branch unchanged.
- The **document picker** (`@tauri-apps/plugin-dialog`'s open/save) still works
  on iOS and surfaces the Files app (including iCloud Drive) for *arbitrary*
  files — keep it as a secondary "Import/Export elsewhere" path if useful, but
  the app-specific iCloud folder above is the primary store per the
  requirement.

---

## 6. Phase 5 — Build, run, iterate

```sh
# Live-reload in the Simulator (fastest dev loop)
pnpm --dir elo-tauri tauri ios dev

# Build a device/release IPA
pnpm --dir elo-tauri tauri ios build
# add --export-method app-store-connect | release-testing | debugging as needed
```

`tauri.conf.json` extras worth setting for iOS:

```jsonc
"bundle": {
  "iOS": {
    "developmentTeam": "YOUR_TEAM_ID",     // or env TAURI_APPLE_DEVELOPMENT_TEAM
    "minimumSystemVersion": "14.0"
  }
}
```

The desktop `app.windows` block (700×500, decorations, etc.) is ignored on iOS
— leave it; it does no harm.

---

## 7. Testing checklist

Run on **Simulator** for layout/logic and a **real device** for iCloud.

- [ ] Desktop build unchanged: `cargo build -p elo-tauri`, app launches with
      tray/shortcuts working; `cargo clippy --workspace --all-targets -- -D
      warnings` and `cargo fmt --all -- --check` pass.
- [ ] `tauri ios dev` launches in the Simulator; math evaluates
      (`evaluate_document` round-trips).
- [ ] Portrait: input lines use full width; results sit below long lines and
      right-aligned beside short ones; no overlap with the next line in the
      common cases; scrolling keeps results aligned.
- [ ] Landscape: reverts to the gutter layout.
- [ ] Safe areas: toolbar clears the notch/Dynamic Island; status bar clears
      the home indicator; rotation re-lays-out cleanly.
- [ ] On-screen keyboard doesn't cover the caret/active line.
- [ ] Tap-to-copy on a result works.
- [ ] iCloud (device, paid account): "Elo" folder appears in Files app; Save
      writes there; Open lists and loads; edits sync to another device;
      undownloaded files load (verify placeholder handling).
- [ ] Cold launch with iCloud signed-out shows a graceful error, not a crash.

---

## 8. File-by-file change map

| File | Change |
|------|--------|
| `crates/elo-tauri/Cargo.toml` | Move `tray-icon`/`global-shortcut` to desktop-only target table; add iOS-only `objc2`/`objc2-foundation`. |
| `crates/elo-tauri/src/lib.rs` | `#[cfg(desktop)]` around global-shortcut registration; add iOS iCloud module + commands; register them in `generate_handler!`. |
| `crates/elo-tauri/capabilities/default.json` | Add `"platforms": ["macOS","windows","linux"]`. |
| `crates/elo-tauri/capabilities/mobile.json` | **New** — iOS/android capability without global-shortcut. |
| `crates/elo-tauri/tauri.conf.json` | Add `bundle.iOS` (team, min version). |
| `crates/elo-tauri/gen/apple/**` | Generated by `tauri ios init`; hand-edit `Info.plist` (NSUbiquitousContainers) + `*.entitlements` (iCloud); commit/force-add the edited files. |
| `elo-tauri/index.html` | `viewport-fit=cover`; textarea `autocapitalize/autocorrect/autocomplete=off`. |
| `elo-tauri/src/styles.css` | `@media (orientation: portrait)` reflow; safe-area-inset padding; larger tap targets. |
| `elo-tauri/src/main.ts` | `.hl-line` spans in highlight; `layoutPortraitResults()`; platform-aware Open/Save routing to iCloud commands + in-app doc list. |
| `elo-tauri/package.json` | Add `@tauri-apps/plugin-os` (platform detection) if used. |

---

## 9. Risks & open decisions

1. **Portrait reflow fidelity (the main one).** Option A floats results and
   can't make the textarea reflow, so a result dropped below a long line can
   overlap the next line. Tune in the Simulator; escalate to Option B (line
   editor rewrite) only if needed — and check with the user first, since it's a
   large change.
2. **iCloud requires a paid account + device** for true validation; the
   Simulator only exercises code paths.
3. **`gen/apple` regeneration** can clobber the hand-edited `Info.plist`/
   entitlements — keep those edits committed and be ready to re-apply after any
   `tauri ios init --reinstall`.
4. **iCloud download/coordination**: plain `std::fs` is fine for v1; add
   `startDownloadingUbiquitousItem` + `NSFileCoordinator` if placeholder or
   conflict cases appear.
5. **`objc2` API drift**: pin the `objc2`/`objc2-foundation` versions; method
   names (`URLForUbiquityContainerIdentifier`, `URLByAppendingPathComponent`)
   may differ slightly across crate versions — let the compiler guide you.

---

## 10. Suggested commit sequence

1. `Cargo.toml` + `lib.rs` feature gating (+ split capabilities). Desktop
   builds unchanged.
2. `tauri ios init` scaffold committed; baseline runs in Simulator.
3. Portrait/responsive CSS + JS (Option A).
4. iCloud entitlements/Info.plist + Rust commands + frontend wiring.
5. `bundle.iOS` config + signing notes.

Each step builds and runs; don't batch them.
