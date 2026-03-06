import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import "./styles.css";

type PtyDataPayload = { session_id: string; data: string };
type PtyExitPayload = { session_id: string; status: number };

type SessionHistoryItem = {
  record_id: string;
  started_at_ms: number;
  ended_at_ms: number;
  status: string;
  exit_code: number | null;
  initial_cwd: string | null;
  last_known_cwd: string | null;
  preview: string;
};

type SessionTranscript = {
  record_id: string;
  transcript: string;
};

type HistorySettings = {
  history_enabled: boolean;
};

type ResumeSessionResponse = {
  session_id: string;
  used_fallback: boolean;
  notice: string | null;
};

let sessionId: string | null = null;
let starting = false;
let initialized = false;
let selectedHistoryId: string | null = null;
let historyItems: SessionHistoryItem[] = [];

let statusEl: HTMLElement | null = null;
let restartBtn: HTMLButtonElement | null = null;
let terminalEl: HTMLElement | null = null;
let historyBtn: HTMLButtonElement | null = null;
let historyPanel: HTMLElement | null = null;
let historyCloseBtn: HTMLButtonElement | null = null;
let historySearchInput: HTMLInputElement | null = null;
let historyListEl: HTMLUListElement | null = null;
let historyTranscriptEl: HTMLElement | null = null;
let historyResumeBtn: HTMLButtonElement | null = null;
let historyOpenTranscriptBtn: HTMLButtonElement | null = null;
let historyEnabledInput: HTMLInputElement | null = null;
let historyClearBtn: HTMLButtonElement | null = null;

const appWindow = getCurrentWindow();

const terminal = new Terminal({
  cursorBlink: true,
  convertEol: true,
  fontFamily: "'JetBrains Mono', 'SFMono-Regular', Menlo, Monaco, Consolas, monospace",
  scrollback: 2000,
  theme: {
    background: "#0c0c0c",
    foreground: "#e5e5e5",
    cursor: "#7bd88f",
  },
});
const fitAddon = new FitAddon();
terminal.loadAddon(fitAddon);

function setStatus(text: string) {
  if (statusEl) {
    statusEl.textContent = text;
  }
}

function errorToMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}

function formatTime(ms: number): string {
  const date = new Date(ms);
  if (Number.isNaN(date.getTime())) return "Unknown time";
  return date.toLocaleString();
}

async function terminateActiveSession() {
  if (!sessionId) return;
  const activeSessionId = sessionId;
  sessionId = null;
  try {
    await invoke("terminate_session", { sessionId: activeSessionId });
  } catch (err) {
    console.error("terminate_session failed", err);
  }
}

async function startPopper(options?: { recordId?: string }) {
  if (starting) return;
  starting = true;
  setStatus("Starting Popper shell…");

  try {
    if (options?.recordId) {
      const result = await Promise.race([
        invoke<ResumeSessionResponse>("start_session_from_history", {
          recordId: options.recordId,
          cols: terminal.cols,
          rows: terminal.rows,
        }),
        new Promise<ResumeSessionResponse>((_, reject) =>
          setTimeout(() => reject(new Error("start_session_from_history timed out")), 8000)
        ),
      ]);
      sessionId = result.session_id;
      setStatus(result.notice ?? "Resumed shell context");
    } else {
      sessionId = await Promise.race([
        invoke<string>("start_session", {
          cols: terminal.cols,
          rows: terminal.rows,
        }),
        new Promise<string>((_, reject) =>
          setTimeout(() => reject(new Error("start_session timed out")), 8000)
        ),
      ]);
      setStatus("Popper shell running");
    }
  } catch (err) {
    console.error("failed to start session", err);
    setStatus(`Failed to start Popper: ${errorToMessage(err)}`);
  } finally {
    starting = false;
  }
}

function setHistoryPanelOpen(open: boolean) {
  if (!historyPanel) return;
  historyPanel.classList.toggle("history--hidden", !open);
  historyPanel.setAttribute("aria-hidden", String(!open));
}

function selectedHistoryItem(): SessionHistoryItem | null {
  if (!selectedHistoryId) return null;
  return historyItems.find((item) => item.record_id === selectedHistoryId) ?? null;
}

function renderHistoryList() {
  if (!historyListEl) return;

  historyListEl.innerHTML = "";
  if (historyItems.length === 0) {
    const empty = document.createElement("li");
    empty.className = "history__empty";
    empty.textContent = "No sessions found.";
    historyListEl.appendChild(empty);
    return;
  }

  for (const item of historyItems) {
    const li = document.createElement("li");
    const button = document.createElement("button");
    button.type = "button";
    button.className = "history__item";
    if (item.record_id === selectedHistoryId) {
      button.classList.add("history__item--selected");
    }

    const title = document.createElement("div");
    title.className = "history__item-title";
    const statusLabel = item.status === "terminated" ? "terminated" : `exit ${item.exit_code ?? "?"}`;
    title.textContent = `${formatTime(item.ended_at_ms)} · ${statusLabel}`;

    const cwd = document.createElement("div");
    cwd.className = "history__item-cwd";
    cwd.textContent = item.last_known_cwd ?? item.initial_cwd ?? "(default cwd)";

    const preview = document.createElement("div");
    preview.className = "history__item-preview";
    preview.textContent = item.preview.trim() || "(no transcript preview)";

    button.appendChild(title);
    button.appendChild(cwd);
    button.appendChild(preview);
    button.addEventListener("click", () => {
      selectedHistoryId = item.record_id;
      renderHistoryList();
      showTranscriptPreview();
    });

    li.appendChild(button);
    historyListEl.appendChild(li);
  }
}

async function showTranscriptPreview() {
  if (!historyTranscriptEl) return;

  const item = selectedHistoryItem();
  if (!item) {
    historyTranscriptEl.textContent = "Select a session to view transcript.";
    return;
  }

  historyTranscriptEl.textContent = `Selected: ${formatTime(item.ended_at_ms)}\n${item.preview || "(no preview)"}`;
}

async function loadSessionHistory() {
  const query = historySearchInput?.value?.trim() ?? "";
  try {
    historyItems = await invoke<SessionHistoryItem[]>("list_session_history", { query });
    if (!selectedHistoryId || !historyItems.some((item) => item.record_id === selectedHistoryId)) {
      selectedHistoryId = historyItems[0]?.record_id ?? null;
    }
    renderHistoryList();
    showTranscriptPreview();
  } catch (err) {
    console.error("list_session_history failed", err);
    setStatus(`Failed to load history: ${errorToMessage(err)}`);
  }
}

async function loadHistorySettings() {
  if (!historyEnabledInput) return;

  try {
    const settings = await invoke<HistorySettings>("get_history_settings");
    historyEnabledInput.checked = settings.history_enabled;
  } catch (err) {
    console.error("get_history_settings failed", err);
  }
}

function attachEvents() {
  terminal.onData((data) => {
    if (!sessionId) return;
    invoke("write_to_session", { sessionId, data }).catch((err) =>
      console.error("write_to_session failed", err)
    );
  });

  window.addEventListener("resize", () => {
    fitAddon.fit();
    if (!sessionId) return;
    invoke("resize_session", {
      sessionId,
      cols: terminal.cols,
      rows: terminal.rows,
    }).catch((err) => console.error("resize_session failed", err));
  });

  listen<PtyDataPayload>("pty-data", (event) => {
    const payload = event.payload;
    if (!payload || payload.session_id !== sessionId) return;
    terminal.write(payload.data);
  });

  listen<PtyExitPayload>("pty-exit", (event) => {
    const payload = event.payload;
    if (!payload || payload.session_id !== sessionId) return;
    sessionId = null;

    loadSessionHistory().catch((err) => {
      console.error("loadSessionHistory after exit failed", err);
    });

    if (payload.status === 0) {
      setStatus("Popper exited. Closing app…");
      appWindow.close();
    } else {
      setStatus(`Popper exited unexpectedly (code ${payload.status}). Click restart to relaunch.`);
    }
  });

  restartBtn?.addEventListener("click", async () => {
    await terminateActiveSession();
    terminal.reset();
    await startPopper();
  });

  historyBtn?.addEventListener("click", async () => {
    const isHidden = historyPanel?.classList.contains("history--hidden") ?? true;
    setHistoryPanelOpen(isHidden);
    if (isHidden) {
      await loadHistorySettings();
      await loadSessionHistory();
    }
  });

  historyCloseBtn?.addEventListener("click", () => setHistoryPanelOpen(false));

  historySearchInput?.addEventListener("input", () => {
    loadSessionHistory().catch((err) => {
      console.error("history search failed", err);
    });
  });

  historyOpenTranscriptBtn?.addEventListener("click", async () => {
    if (!historyTranscriptEl) return;
    const item = selectedHistoryItem();
    if (!item) {
      historyTranscriptEl.textContent = "Select a session first.";
      return;
    }

    try {
      const payload = await invoke<SessionTranscript>("get_session_transcript", {
        recordId: item.record_id,
      });
      historyTranscriptEl.textContent = payload.transcript || "(empty transcript)";
    } catch (err) {
      historyTranscriptEl.textContent = `Failed to open transcript: ${errorToMessage(err)}`;
    }
  });

  historyResumeBtn?.addEventListener("click", async () => {
    const item = selectedHistoryItem();
    if (!item) {
      setStatus("Select a history session to resume.");
      return;
    }

    await terminateActiveSession();
    terminal.reset();
    await startPopper({ recordId: item.record_id });
    setHistoryPanelOpen(false);
  });

  if (historyEnabledInput) {
    const historyToggle = historyEnabledInput;
    historyToggle.addEventListener("change", async () => {
      const enabled = historyToggle.checked;
      try {
        const updated = await invoke<HistorySettings>("set_history_enabled", { enabled });
        historyToggle.checked = updated.history_enabled;
        setStatus(updated.history_enabled ? "Session history enabled" : "Session history disabled");
      } catch (err) {
        historyToggle.checked = !enabled;
        setStatus(`Failed to update history setting: ${errorToMessage(err)}`);
      }
    });
  }

  historyClearBtn?.addEventListener("click", async () => {
    try {
      await invoke("clear_session_history");
      historyItems = [];
      selectedHistoryId = null;
      renderHistoryList();
      showTranscriptPreview();
      setStatus("Session history cleared");
    } catch (err) {
      setStatus(`Failed to clear history: ${errorToMessage(err)}`);
    }
  });
}

function init() {
  if (initialized) return;
  initialized = true;

  statusEl = document.getElementById("status");
  restartBtn = document.getElementById("restart-btn") as HTMLButtonElement | null;
  historyBtn = document.getElementById("history-btn") as HTMLButtonElement | null;
  terminalEl = document.getElementById("terminal");
  historyPanel = document.getElementById("history-panel");
  historyCloseBtn = document.getElementById("history-close-btn") as HTMLButtonElement | null;
  historySearchInput = document.getElementById("history-search") as HTMLInputElement | null;
  historyListEl = document.getElementById("history-list") as HTMLUListElement | null;
  historyTranscriptEl = document.getElementById("history-transcript");
  historyResumeBtn = document.getElementById("history-resume-btn") as HTMLButtonElement | null;
  historyOpenTranscriptBtn = document.getElementById(
    "history-open-transcript-btn"
  ) as HTMLButtonElement | null;
  historyEnabledInput = document.getElementById("history-enabled") as HTMLInputElement | null;
  historyClearBtn = document.getElementById("history-clear-btn") as HTMLButtonElement | null;

  if (terminalEl) {
    terminal.open(terminalEl);
    fitAddon.fit();
  } else {
    setStatus("Terminal container missing");
  }

  attachEvents();
  loadHistorySettings().catch((err) => {
    console.error("loadHistorySettings failed", err);
  });
  startPopper();
}

window.addEventListener("DOMContentLoaded", init);
window.addEventListener("load", init);
init();
