# Elo — Icon & App Identity Design Brief

A prompt for a design agent (e.g. an image/icon generation model or a human designer)
to create a fresh, modern set of iconography for **Elo**, with a primary focus on the
new **iOS app** icon.

---

## The prompt

> You are designing a new icon set for **Elo**, a notepad-style calculator app.
>
> **What Elo is.** Elo is a *natural-language* calculator and scratchpad. Instead of
> tapping buttons on a numeric keypad, you type plain-English math and the answer
> appears live next to it, line by line — like writing in a notebook that does the
> arithmetic for you. It handles things like `5 feet in meters`, `15% of 200`,
> `100 USD in EUR`, `today + 3 days`, `time in Asia/Tokyo`, `sqrt(2) * 10`, running
> totals, and variables. It is calm, fast, keyboard-first, and text-centric. It is a
> faithful re-implementation of the Mac app *Numi*. The vibe is "a quiet, intelligent
> text editor that happens to be a calculator," **not** a chunky button-grid calculator.
>
> **The goal.** Design a modern icon that instantly says *smart text-based math
> notepad* and is distinct from the cliché calculator look (the 4×4 button grid,
> the orange "=" key, the pocket-calculator silhouette). The current icon — two
> interlocking curved strokes forming an abstract figure-8 / infinity — is too
> abstract, reads as a generic logo, and does not communicate what the app does.
> Replace it.
>
> **Concept directions** (explore a few, then commit to the strongest one):
> 1. **The living line.** A single line/row of typed expression with its result —
>    e.g. an abstracted `=` revealing an answer, or a text caret/cursor paired with a
>    result. Conveys "type → instant answer."
> 2. **Notepad + math.** A minimal page/notepad mark fused with a mathematical glyph
>    (`=`, `±`, `≈`, `Σ`, or a tasteful operator stack). Conveys "calculator notebook."
> 3. **The monogram.** A confident, modern letterform — an "E" for Elo — constructed
>    from math/UI primitives (the three bars of "E" doubling as `=` plus a baseline,
>    or built from an operator rhythm). Works beautifully tiny.
> 4. **The equals glyph as hero.** A bold, geometric `=` (or `≈`, nodding to unit
>    conversion and approximation) as the central, ownable mark.
>
> Pick the idea that is most legible at 29×29 px and most ownable. Favor a single
> strong idea over a busy composition.
>
> **Visual style.** Modern, geometric, flat-with-subtle-depth (think contemporary
> Apple/Linear/Raycast app icons): a confident solid or smooth-gradient background
> with one clean foreground mark. Crisp, slightly rounded geometry. Optional very
> subtle gradient or soft inner light for depth — avoid skeuomorphism, heavy bevels,
> long shadows, gloss, and 2010-era glassy buttons. The mark should have generous
> optical margins and read at a glance.
>
> **Palette.** Draw from Elo's existing Catppuccin-Mocha-based UI theme so the icon
> feels native to the app:
> - Background base / dark: `#1e1e2e` (with `#181825` for deeper shades)
> - Accent blue (primary): `#89b4fa`
> - Result green: `#a6e3a1`
> - Label yellow: `#f9e2af`
> - Mauve/violet: `#cba6f7`
> - Accent red/pink: `#f38ba8`
> - Light text / foreground: `#cdd6f4`
>
> Recommended treatment: a dark base (`#1e1e2e` / deep blue-violet gradient) with the
> mark in the accent blue→mauve→green range, OR a vivid single-hue gradient background
> with a light (`#cdd6f4` / white) mark. The app uses a **monospace** typeface
> (JetBrains Mono); if any glyph/letterform appears in the icon, echo that geometric,
> even-width, slightly technical character. Do not literally render long strings of
> text — keep it to a symbol or a single character.
>
> **Deliverables.**
> 1. **iOS app icon** — a single **1024×1024 px** master, full-bleed, **square, no
>    transparency, no rounded corners** (iOS applies the rounded-rect mask itself), no
>    alpha channel, sRGB. Keep all important content within a safe area (~10% margin)
>    so nothing is clipped by the mask. Must remain crisp and legible when scaled down
>    to 180, 120, 87, 80, 60, 58, 40, and **29 px** — avoid hairline strokes and fine
>    detail that disappears at small sizes.
> 2. **iOS light / dark / tinted variants** (iOS 18+): provide a **light** version, a
>    **dark** version (transparent-friendly mark intended to sit on the system's dark
>    backdrop), and a single-color **tinted/monochrome** version (grayscale mark that
>    the system recolors). Design the mark so it survives being reduced to one color.
> 3. **Master logomark** as scalable **SVG** (vector), plus PNG exports.
> 4. **Cross-platform set** to refresh the existing desktop assets, exported at:
>    `32×32`, `128×128`, `128×128@2x (256)`, `512×512`, `1024×1024`, plus a Windows
>    `.ico` and macOS `.icns`. (These replace the files in
>    `crates/elo-tauri/icons/`.)
> 5. Optional: a **monochrome/glyph** version for the macOS menu-bar tray and favicons.
>
> **Constraints & don'ts.**
> - Don't use the generic calculator button-grid or pocket-calculator shape.
> - Don't reuse the current figure-8 / infinity mark.
> - No transparency, gloss, drop shadows, or rounded corners baked into the iOS master.
> - No literal screenshots of UI text; no more than a single character/glyph of type.
> - Must be instantly recognizable in the iOS Home Screen, App Library, Settings list,
>   Spotlight, and notifications at small sizes.
>
> **Output.** Provide: (a) 3–4 quick concept thumbnails, (b) one recommended direction
> rendered as the 1024×1024 iOS master, (c) the light/dark/tinted iOS variants, and
> (d) the source SVG. Briefly explain the chosen concept and how it maps to "natural-
> language math notepad."

---

## Reference facts (for whoever runs this prompt)

| Item | Value |
| --- | --- |
| App name | Elo |
| Bundle identifier | `com.elo.calculator` |
| What it is | Notepad-style natural-language calculator (Numi re-implementation) |
| Platforms | CLI, Tauri desktop (macOS/Linux/Windows), iOS (in progress) |
| UI theme | Catppuccin Mocha (dark), monospace (JetBrains Mono) |
| Existing icon assets | `crates/elo-tauri/icons/` (tauri-generated set) |
| iOS asset target | Xcode asset catalog `AppIcon.appiconset` (1024 master + light/dark/tinted) |
| Current icon | Abstract orange/cyan interlocking figure-8 on transparent bg — to be replaced |
