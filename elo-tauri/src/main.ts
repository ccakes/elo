import { invoke } from "@tauri-apps/api/core";

interface LineResult {
  input: string;
  display: string;
  is_empty: boolean;
  is_error: boolean;
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
      html += `<div class="result-line error" title="${escapeHtml(r.display)}">${escapeHtml(r.display)}</div>`;
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
        `<span class="md-hash">${headerMatch[1]}</span><span class="md-header">${headerMatch[2]}</span>`,
      );
      continue;
    }

    // Comments
    if (trimmed.startsWith("//")) {
      result.push(`<span class="md-comment">${escaped}</span>`);
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

  return result.join("\n") + "\n";
}

function applyInlineFormatting(escaped: string): string {
  // Single-pass regex matching inline code, bold, underline, italic
  // Order matters: longer delimiters first to avoid partial matches
  return escaped.replace(
    /(`[^`]+`)|(\*\*[^*]+\*\*)|(__[^_]+__)|(\*[^*]+\*)|(_[^_]+_)/g,
    (match, code, bold, underline, italicStar, italicUnderscore) => {
      if (code) return `<span class="md-inline-code">${match}</span>`;
      if (bold) return `<span class="md-bold">${match}</span>`;
      if (underline) return `<span class="md-underline">${match}</span>`;
      if (italicStar) return `<span class="md-italic">${match}</span>`;
      if (italicUnderscore) return `<span class="md-italic">${match}</span>`;
      return match;
    },
  );
}

// Sync scroll between editor, highlight, and results
function syncScroll() {
  resultsEl.scrollTop = editor.scrollTop;
  highlightEl.scrollTop = editor.scrollTop;
  highlightEl.scrollLeft = editor.scrollLeft;
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

async function openDocument() {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const { readTextFile } = await import("@tauri-apps/plugin-fs");
    const path = await open({
      filters: [
        { name: "Elo Notes", extensions: ["elo", "txt", "md"] },
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
  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const { writeTextFile } = await import("@tauri-apps/plugin-fs");

    let path = currentFilePath;
    if (!path) {
      const chosen = await save({
        filters: [{ name: "Elo Notes", extensions: ["elo", "txt"] }],
        defaultPath: "untitled.elo",
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
