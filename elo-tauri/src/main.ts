import { invoke } from "@tauri-apps/api/core";

interface LineResult {
  input: string;
  display: string;
  is_empty: boolean;
  is_error: boolean;
  // True when the line is plain text / markdown prose rather than a formula.
  is_text: boolean;
  error: string | null;
}

let editor: HTMLTextAreaElement;
let highlightEl: HTMLElement;
let resultsEl: HTMLElement;
let statusLeft: HTMLElement;
let statusRight: HTMLElement;
let currentFilePath: string | null = null;
let isDirty = false;
let evalTimeout: number | null = null;

// Debounced evaluation
function scheduleEval() {
  if (evalTimeout !== null) clearTimeout(evalTimeout);
  evalTimeout = window.setTimeout(evaluateDocument, 30);
}

async function evaluateDocument() {
  const text = editor.value;
  try {
    const results: LineResult[] = await invoke("evaluate_document", { text });
    renderResults(results);
    updateStatus(text, results);
    layoutPortraitResults();
  } catch (e) {
    console.error("Evaluation error:", e);
  }
}

function renderResults(results: LineResult[]) {
  const lines = editor.value.split("\n");
  let html = "";

  for (let i = 0; i < lines.length; i++) {
    const r = results[i];
    const inputLine = lines[i];

    if (!r || r.is_empty) {
      // Classify the input line for styling
      if (inputLine.trimStart().startsWith("#")) {
        const label = inputLine.replace(/^#\s*/, "").trim();
        html += `<div class="result-line header">${escapeHtml(label)}</div>`;
      } else if (inputLine.trimStart().startsWith("//")) {
        html += `<div class="result-line comment">comment</div>`;
      } else if (inputLine.trimStart().match(/^[-*]\s/)) {
        html += `<div class="result-line empty">&nbsp;</div>`;
      } else {
        html += `<div class="result-line empty">&nbsp;</div>`;
      }
    } else if (r.is_error) {
      const tooltip = r.error ? escapeHtml(r.error) : escapeHtml(r.display);
      html += `<div class="result-line error" title="${tooltip}">${escapeHtml(r.display)}</div>`;
    } else {
      const cls = classifyResult(r.display, inputLine);
      html += `<div class="result-line ${cls}" data-value="${escapeHtml(r.display)}" title="Click to copy">${escapeHtml(r.display)}</div>`;
    }
  }

  resultsEl.innerHTML = html;

  // Add click-to-copy handlers
  resultsEl.querySelectorAll(".result-line[data-value]").forEach((el) => {
    el.addEventListener("click", () => {
      const value = (el as HTMLElement).dataset.value;
      if (value) copyToClipboard(value);
    });
  });
}

function classifyResult(display: string, _input: string): string {
  if (display.match(/^\d{4}-\d{2}-\d{2}/)) return "datetime";
  return "";
}

function updateStatus(text: string, results: LineResult[]) {
  const lineCount = text.split("\n").length;
  const resultCount = results.filter((r) => !r.is_empty && !r.is_error).length;
  statusLeft.textContent = `${lineCount} line${lineCount !== 1 ? "s" : ""}`;
  statusRight.textContent = `${resultCount} result${resultCount !== 1 ? "s" : ""}`;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

// --- Markdown highlighting for the editor overlay ---

function updateHighlight() {
  highlightEl.innerHTML = highlightMarkdown(editor.value);
}

function highlightMarkdown(text: string): string {
  const lines = text.split("\n");
  let inCodeFence = false;
  const result: string[] = [];

  for (const line of lines) {
    const escaped = escapeHtml(line);
    const trimmed = escaped.trimStart();

    // Code fence toggle
    if (trimmed.startsWith("```")) {
      inCodeFence = !inCodeFence;
      result.push(`<span class="md-code-fence">${escaped}</span>`);
      continue;
    }

    // Lines inside code fences
    if (inCodeFence) {
      result.push(`<span class="md-code-block">${escaped}</span>`);
      continue;
    }

    // Headers
    const headerMatch = escaped.match(/^(#{1,3}\s)(.*)/);
    if (headerMatch) {
      result.push(
        `<span class="md-hash">${headerMatch[1]}</span><span class="md-header">${applyOperators(headerMatch[2])}</span>`,
      );
      continue;
    }

    // Comments
    if (trimmed.startsWith("//")) {
      result.push(`<span class="md-comment">${applyOperators(escaped)}</span>`);
      continue;
    }

    // List items: marker styled, rest gets inline formatting
    const listMatch = escaped.match(/^(\s*)([-*]\s)(.*)/);
    if (listMatch && listMatch[3].match(/^[A-Za-z\u00C0-\u024F]/)) {
      result.push(
        `${listMatch[1]}<span class="md-list-marker">${listMatch[2]}</span>${applyInlineFormatting(listMatch[3])}`,
      );
      continue;
    }

    // Regular line with inline formatting
    result.push(applyInlineFormatting(escaped));
  }

  // Wrap each logical line in a measurable span. Because #editor-highlight
  // wraps text identically to the textarea, each .hl-line's offsetTop/
  // offsetHeight is the true rendered position of that logical line
  // (including soft-wrapping) — the primitive layoutPortraitResults() uses.
  return (
    result
      .map((html, i) => `<span class="hl-line" data-line="${i}">${html}</span>`)
      .join("\n") + "\n"
  );
}

function applyInlineFormatting(escaped: string): string {
  // Walk the markdown matches manually so inline code/bold/italic/underline are
  // emitted verbatim, while only the plain-text gaps between them receive
  // operator substitution. This protects a markdown `*` (italic/bold) from
  // becoming `×` while still converting a math `*` (e.g. `2 * 3`, `(a+b)*c`),
  // which lives in a gap.
  // Order matters: longer delimiters first to avoid partial matches.
  const md = /(`[^`]+`)|(\*\*[^*]+\*\*)|(__[^_]+__)|(\*[^*]+\*)|(_[^_]+_)/g;
  let out = "";
  let last = 0;
  let m: RegExpExecArray | null;
  while ((m = md.exec(escaped)) !== null) {
    out += applyOperators(escaped.slice(last, m.index));
    const [match, code, bold, underline, italicStar] = m;
    if (code) out += `<span class="md-inline-code">${match}</span>`;
    else if (bold) out += `<span class="md-bold">${match}</span>`;
    else if (underline) out += `<span class="md-underline">${match}</span>`;
    else if (italicStar) out += `<span class="md-italic">${match}</span>`;
    else out += `<span class="md-italic">${match}</span>`;
    last = m.index + match.length;
  }
  out += applyOperators(escaped.slice(last));
  return out;
}

// Display-only operator glyphs. Runs on already-HTML-escaped text (so `<`/`>`
// are `&lt;`/`&gt;`). The underlying textarea value is never changed, so the
// Rust parser still receives raw ASCII. Two-char ligatures use a 2ch-wide span
// so the glyph occupies exactly the same character grid as the source chars,
// keeping the textarea caret and soft-wrapping aligned with the overlay.
function applyOperators(s: string): string {
  return s
    .replace(/-&gt;/g, `<span class="op-lig">→</span>`)
    .replace(/=&gt;/g, `<span class="op-lig">⇒</span>`)
    .replace(/!=/g, `<span class="op-lig">≠</span>`)
    .replace(/&lt;=/g, `<span class="op-lig">≤</span>`)
    .replace(/&gt;=/g, `<span class="op-lig">≥</span>`)
    // Division: only when surrounded by whitespace, so dates (06/28),
    // rates/units (km/h), and paths stay literal. Single cell.
    .replace(/(?<=\s)\/(?=\s)/g, `<span class="op-div">÷</span>`)
    // Multiplication: any remaining `*` (markdown `*` is already consumed on
    // inline-formatted lines before this runs). Single cell.
    .replace(/\*/g, `<span class="op-mul">×</span>`);
}

// Sync scroll between editor, highlight, and results
function syncScroll() {
  resultsEl.scrollTop = editor.scrollTop;
  highlightEl.scrollTop = editor.scrollTop;
  highlightEl.scrollLeft = editor.scrollLeft;
  // In portrait the results gutter is a non-scrolling overlay; re-offset chips.
  if (isPortrait()) layoutPortraitResults();
}

// --- Portrait floating-result layout (Option A) ---
//
// Keep matching the CSS media query in styles.css.
function isPortrait(): boolean {
  return window.matchMedia("(orientation: portrait) and (max-width: 600px)")
    .matches;
}

// Reusable canvas for measuring rendered text width in the editor font.
let measureCanvas: HTMLCanvasElement | null = null;
let measureFont = "";

function measureTextWidth(text: string): number {
  if (!measureCanvas) measureCanvas = document.createElement("canvas");
  const ctx = measureCanvas.getContext("2d");
  if (!ctx) return 0;
  if (!measureFont) {
    const cs = getComputedStyle(editor);
    measureFont = `${cs.fontSize} ${cs.fontFamily}`;
  }
  ctx.font = measureFont;
  return ctx.measureText(text).width;
}

// Position each result chip relative to its logical line: right-aligned beside
// short lines, dropped onto its own row beneath long lines (only when the next
// line is blank, so it can't land on top of real text).
//
// NOTE: this is the v1 heuristic the IOS-PORT guide flags for Simulator tuning.
// It is correct-by-construction (no overlap when dropped) but the "is the line
// long enough to crowd the chip" threshold is the knob to turn against real
// notes on a device.
function layoutPortraitResults() {
  if (!isPortrait()) {
    // Landscape/desktop: clear any inline positioning we set previously.
    resultsEl
      .querySelectorAll<HTMLElement>(".result-line")
      .forEach((el) => (el.style.top = ""));
    return;
  }

  const spans = highlightEl.querySelectorAll<HTMLElement>(".hl-line");
  const chips = resultsEl.querySelectorAll<HTMLElement>(".result-line");
  const lines = editor.value.split("\n");
  const scrollTop = editor.scrollTop;

  // Width available for text before it would collide with a right chip.
  // editor content width minus its horizontal padding.
  const cs = getComputedStyle(editor);
  const padX =
    parseFloat(cs.paddingLeft || "0") + parseFloat(cs.paddingRight || "0");
  const contentWidth = editor.clientWidth - padX;

  spans.forEach((span, i) => {
    const chip = chips[i];
    if (!chip) return;

    const chipWidth = chip.offsetWidth || 0;
    const gap = 16;
    const roomForText = contentWidth - chipWidth - gap;

    const lineWidth = measureTextWidth(lines[i] ?? "");
    const nextBlank = (lines[i + 1] ?? "").trim() === "";
    // Drop below only when the line crowds the chip AND there's a blank line
    // beneath to land in — otherwise keep it right-aligned on the line's row.
    const dropBelow = lineWidth > roomForText && nextBlank;

    const top = dropBelow
      ? span.offsetTop + span.offsetHeight
      : span.offsetTop;
    chip.style.top = `${top - scrollTop}px`;
  });
}

// Copy to clipboard with toast
async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    showToast("Copied!");
  } catch {
    // Fallback
    const ta = document.createElement("textarea");
    ta.value = text;
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    document.body.removeChild(ta);
    showToast("Copied!");
  }
}

function showToast(message: string) {
  let toast = document.querySelector(".toast") as HTMLElement;
  if (!toast) {
    toast = document.createElement("div");
    toast.className = "toast";
    document.body.appendChild(toast);
  }
  toast.textContent = message;
  toast.classList.add("show");
  setTimeout(() => toast.classList.remove("show"), 1200);
}

// Copy all results
function copyAllResults() {
  const resultDivs = resultsEl.querySelectorAll(".result-line[data-value]");
  const lines: string[] = [];
  resultDivs.forEach((el) => {
    const v = (el as HTMLElement).dataset.value;
    if (v) lines.push(v);
  });
  if (lines.length > 0) {
    copyToClipboard(lines.join("\n"));
  }
}

// File operations
async function newDocument() {
  editor.value = "";
  currentFilePath = null;
  isDirty = false;
  document.title = "Elo";
  updateHighlight();
  evaluateDocument();
}

// --- Platform-aware document storage ---
//
// On iOS we route Open/Save through the app's iCloud Documents folder
// (icloud_* commands in the Rust backend) and an in-app document list,
// since the desktop file dialog + absolute paths don't apply on sandboxed iOS.
function isIOS(): boolean {
  return (
    /iPad|iPhone|iPod/.test(navigator.userAgent) ||
    // iPadOS 13+ reports as desktop Safari; disambiguate by touch.
    (navigator.platform === "MacIntel" && navigator.maxTouchPoints > 1)
  );
}

// Lightweight modal: renders a titled panel with arbitrary children and a
// cancel affordance. Returns a teardown function.
function showModal(title: string, build: (close: () => void) => HTMLElement) {
  const overlay = document.createElement("div");
  overlay.className = "modal-overlay";
  const panel = document.createElement("div");
  panel.className = "modal-panel";
  const heading = document.createElement("div");
  heading.className = "modal-title";
  heading.textContent = title;
  panel.appendChild(heading);
  const close = () => overlay.remove();
  panel.appendChild(build(close));
  overlay.appendChild(panel);
  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) close();
  });
  document.body.appendChild(overlay);
  return close;
}

async function openDocumentIOS() {
  try {
    const names: string[] = await invoke("icloud_list_documents");
    showModal("Open from iCloud", (close) => {
      const list = document.createElement("div");
      list.className = "modal-list";
      if (names.length === 0) {
        const empty = document.createElement("div");
        empty.className = "modal-empty";
        empty.textContent = "No notes in iCloud yet.";
        list.appendChild(empty);
      }
      for (const name of names) {
        const item = document.createElement("button");
        item.className = "modal-item";
        item.textContent = name;
        item.addEventListener("click", async () => {
          try {
            const content: string = await invoke("icloud_read_document", {
              name,
            });
            editor.value = content;
            currentFilePath = name;
            isDirty = false;
            document.title = `Elo — ${name}`;
            updateHighlight();
            evaluateDocument();
          } catch (e) {
            console.error("iCloud read error:", e);
            showToast("Couldn't open");
          }
          close();
        });
        list.appendChild(item);
      }
      return list;
    });
  } catch (e) {
    console.error("iCloud list error:", e);
    showToast("iCloud unavailable");
  }
}

async function saveDocumentIOS() {
  const write = async (name: string) => {
    if (!name.trim()) return;
    // Default an extension so the file is recognised on re-list.
    if (!/\.(elo|txt|md)$/.test(name)) name += ".md";
    try {
      await invoke("icloud_write_document", { name, contents: editor.value });
      currentFilePath = name;
      isDirty = false;
      document.title = `Elo — ${name}`;
      showToast("Saved!");
    } catch (e) {
      console.error("iCloud write error:", e);
      showToast("Couldn't save");
    }
  };

  if (currentFilePath) {
    await write(currentFilePath);
    return;
  }

  showModal("Save to iCloud", (close) => {
    const form = document.createElement("form");
    form.className = "modal-form";
    const input = document.createElement("input");
    input.className = "modal-input";
    input.type = "text";
    input.placeholder = "note name";
    input.value = "untitled.md";
    const ok = document.createElement("button");
    ok.type = "submit";
    ok.className = "modal-item";
    ok.textContent = "Save";
    form.appendChild(input);
    form.appendChild(ok);
    form.addEventListener("submit", async (e) => {
      e.preventDefault();
      const name = input.value;
      close();
      await write(name);
    });
    setTimeout(() => input.focus(), 0);
    return form;
  });
}

async function openDocument() {
  if (isIOS()) return openDocumentIOS();
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const { readTextFile } = await import("@tauri-apps/plugin-fs");
    const path = await open({
      filters: [
        { name: "Elo Notes", extensions: ["md", "txt", "elo"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });
    if (path) {
      const content = await readTextFile(path as string);
      editor.value = content;
      currentFilePath = path as string;
      isDirty = false;
      document.title = `Elo — ${fileName(currentFilePath)}`;
      updateHighlight();
      evaluateDocument();
    }
  } catch (e) {
    console.error("Open error:", e);
  }
}

async function saveDocument() {
  if (isIOS()) return saveDocumentIOS();
  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const { writeTextFile } = await import("@tauri-apps/plugin-fs");

    let path = currentFilePath;
    if (!path) {
      const chosen = await save({
        filters: [{ name: "Elo Notes", extensions: ["md", "txt", "elo"] }],
        defaultPath: "untitled.md",
      });
      if (!chosen) return;
      path = chosen;
    }

    await writeTextFile(path, editor.value);
    currentFilePath = path;
    isDirty = false;
    document.title = `Elo — ${fileName(currentFilePath)}`;
    showToast("Saved!");
  } catch (e) {
    console.error("Save error:", e);
  }
}

async function exportDocument() {
  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const { writeTextFile } = await import("@tauri-apps/plugin-fs");

    const results: LineResult[] = await invoke("evaluate_document", {
      text: editor.value,
    });

    const lines = editor.value.split("\n");
    let exported = "";
    for (let i = 0; i < lines.length; i++) {
      const r = results[i];
      const line = lines[i];
      if (r && !r.is_empty && !r.is_error) {
        // Pad result to right-align
        const pad = Math.max(1, 50 - line.length);
        exported += line + " ".repeat(pad) + "= " + r.display + "\n";
      } else {
        exported += line + "\n";
      }
    }

    const path = await save({
      filters: [
        { name: "Text", extensions: ["txt"] },
        { name: "Markdown", extensions: ["md"] },
      ],
      defaultPath: "export.txt",
    });
    if (path) {
      await writeTextFile(path, exported);
      showToast("Exported!");
    }
  } catch (e) {
    console.error("Export error:", e);
  }
}

function fileName(path: string): string {
  return path.split("/").pop() || path.split("\\").pop() || path;
}

// Keyboard shortcuts
function handleKeyboard(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey;
  if (mod && e.key === "n") {
    e.preventDefault();
    newDocument();
  } else if (mod && e.key === "o") {
    e.preventDefault();
    openDocument();
  } else if (mod && e.key === "s") {
    e.preventDefault();
    saveDocument();
  } else if (mod && e.shiftKey && e.key === "c") {
    e.preventDefault();
    copyAllResults();
  }
}

// Init
window.addEventListener("DOMContentLoaded", () => {
  editor = document.getElementById("editor") as HTMLTextAreaElement;
  highlightEl = document.getElementById("editor-highlight") as HTMLElement;
  resultsEl = document.getElementById("results") as HTMLElement;
  statusLeft = document.getElementById("status-left") as HTMLElement;
  statusRight = document.getElementById("status-right") as HTMLElement;

  editor.addEventListener("input", () => {
    isDirty = true;
    updateHighlight();
    scheduleEval();
  });

  editor.addEventListener("scroll", syncScroll);
  document.addEventListener("keydown", handleKeyboard);

  // Re-lay-out floating results on geometry changes.
  window.addEventListener("resize", layoutPortraitResults);
  window.addEventListener("orientationchange", layoutPortraitResults);
  // The on-screen keyboard resizes the visual viewport, not the layout
  // viewport — track it so chips stay aligned and the caret stays visible.
  if (window.visualViewport) {
    window.visualViewport.addEventListener("resize", layoutPortraitResults);
  }

  document.getElementById("btn-new")!.addEventListener("click", newDocument);
  document.getElementById("btn-open")!.addEventListener("click", openDocument);
  document.getElementById("btn-save")!.addEventListener("click", saveDocument);
  document
    .getElementById("btn-export")!
    .addEventListener("click", exportDocument);
  document
    .getElementById("btn-copy-all")!
    .addEventListener("click", copyAllResults);

  // Initial highlight and evaluation
  updateHighlight();
  evaluateDocument();
});
