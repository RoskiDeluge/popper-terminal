import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import "./styles.css";

let sessionId: string | null = null;
let starting = false;
let statusEl: HTMLElement | null = null;
let restartBtn: HTMLElement | null = null;
let terminalEl: HTMLElement | null = null;
let initialized = false;
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
  if (statusEl) statusEl.textContent = text;
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

async function startPopper() {
  if (starting) return;
  starting = true;
  setStatus("Starting Popper shell…");
  console.log("[popper] startPopper invoked");
  try {
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
    console.log("[popper] session started", sessionId);
  } catch (err) {
    console.error("[popper] failed to start", err);
    setStatus(`Failed to start Popper: ${errorToMessage(err)}`);
  } finally {
    starting = false;
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

  listen("pty-data", (event) => {
    const payload = event.payload as { session_id: string; data: string };
    if (!payload || payload.session_id !== sessionId) return;
    terminal.write(payload.data);
  });

  listen("pty-exit", (event) => {
    const payload = event.payload as { session_id: string; status: number };
    if (!payload || payload.session_id !== sessionId) return;
    sessionId = null;
    if (payload.status === 0) {
      setStatus("Popper exited. Closing app…");
      appWindow.close();
    } else {
      setStatus(
        `Popper exited unexpectedly (code ${payload.status}). Click restart to relaunch.`
      );
    }
  });

  restartBtn?.addEventListener("click", () => {
    terminal.reset();
    startPopper();
  });
}

function init() {
  if (initialized) return;
  initialized = true;
  console.log("[popper] init");
  statusEl = document.getElementById("status");
  restartBtn = document.getElementById("restart-btn");
  terminalEl = document.getElementById("terminal");

  if (terminalEl) {
    terminal.open(terminalEl);
    fitAddon.fit();
  } else {
    console.error("Terminal container not found");
    setStatus("Terminal container missing");
  }

  attachEvents();
  startPopper();
}

window.addEventListener("DOMContentLoaded", init);
window.addEventListener("load", init);
init();
