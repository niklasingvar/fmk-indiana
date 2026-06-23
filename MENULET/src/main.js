import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

// State
let daemonIsOurs = false;

// DOM
const focusInput = document.getElementById("focus");
const foldersList = document.getElementById("folders");
const emptyState = document.getElementById("empty-state");
const addBtn = document.getElementById("add-btn");
const statusDot = document.getElementById("status-dot");
const statusLabel = document.getElementById("status-label");
const statusAction = document.getElementById("status-action");
const copiedFlash = document.getElementById("copied-flash");

// Focus persistence
async function loadFocus() {
  try { focusInput.value = await invoke("read_focus"); } catch (_) {}
}
async function saveFocus() {
  try { await invoke("save_focus", { text: focusInput.value }); } catch (_) {}
}
focusInput.addEventListener("blur", saveFocus);
focusInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") { focusInput.blur(); saveFocus(); }
});

// Check if we own the daemon
async function checkOwnership() {
  try { daemonIsOurs = await invoke("daemon_is_ours"); } catch (_) { daemonIsOurs = false; }
}

// Refresh folder list from daemon
async function refreshFolders() {
  try {
    const resp = await invoke("status");
    renderFolders(resp.folders || []);
    setRunning(true);
  } catch (_) {
    setRunning(false);
  }
}

function renderFolders(folders) {
  foldersList.innerHTML = "";
  if (folders.length === 0) {
    emptyState.hidden = false;
    foldersList.hidden = true;
  } else {
    emptyState.hidden = true;
    foldersList.hidden = false;
    for (const f of folders) {
      const li = document.createElement("li");
      li.className = "folder";

      const name = document.createElement("span");
      name.className = "basename";
      name.textContent = basename(f.path);

      const count = document.createElement("span");
      count.className = "count";
      count.textContent = f.count + "\u00D7";

      li.appendChild(name);
      li.appendChild(count);

      li.addEventListener("click", async () => {
        try {
          const text = await invoke("copy_folder", { path: f.path });
          // Write to clipboard via the web API
          await navigator.clipboard.writeText(text);
          flashCopied();
        } catch (e) { console.error("copy failed:", e); }
      });

      li.addEventListener("contextmenu", async (e) => {
        e.preventDefault();
        try {
          await invoke("remove_folder", { path: f.path });
          await refreshFolders();
        } catch (err) { console.error("remove failed:", err); }
      });

      foldersList.appendChild(li);
    }
  }
}

function basename(path) {
  try {
    const home = Deno?.env?.get("HOME") || "";
    if (home && path.startsWith(home)) return "~" + path.slice(home.length);
  } catch (_) {}
  const parts = path.split("/");
  return parts[parts.length - 1] || path;
}

function flashCopied() {
  copiedFlash.classList.add("show");
  setTimeout(() => copiedFlash.classList.remove("show"), 1200);
}

function setRunning(running) {
  statusDot.className = running ? "running" : "stopped";
  statusLabel.textContent = running ? "Server running" : "Server stopped";
  if (!running) {
    foldersList.innerHTML = "";
    emptyState.hidden = true;
    daemonIsOurs = false;
  }
  updateActionButton(running);
}

function updateActionButton(running) {
  if (running) {
    if (daemonIsOurs) {
      statusAction.textContent = "\u23F9";
      statusAction.title = "Stop server";
      statusAction.style.display = "";
    } else {
      statusAction.style.display = "none";
    }
  } else {
    statusAction.textContent = "\u25B6";
    statusAction.title = "Start server";
    statusAction.style.display = "";
  }
}

function setConnecting() {
  statusDot.className = "";
  statusDot.innerHTML = '<span class="spinner"></span>';
  statusLabel.textContent = "Starting\u2026";
  statusAction.style.display = "none";
}

// Add folder
addBtn.addEventListener("click", async () => {
  try {
    const dir = await open({ directory: true, multiple: false, title: "Monitor a folder" });
    if (dir) {
      await invoke("add_folder", { path: dir });
      await refreshFolders();
    }
  } catch (e) { console.error("add folder failed:", e); }
});

emptyState.addEventListener("click", () => addBtn.click());

// Start/stop button
statusAction.addEventListener("click", async () => {
  const isRunning = statusDot.className === "running";
  if (isRunning && daemonIsOurs) {
    try {
      await invoke("shutdown");
      daemonIsOurs = false;
      setRunning(false);
    } catch (_) {}
  } else if (!isRunning) {
    setConnecting();
    try {
      await invoke("spawn_sidecar");
      daemonIsOurs = true;
      for (let i = 0; i < 20; i++) {
        await new Promise((r) => setTimeout(r, 500));
        try {
          await invoke("status");
          await refreshFolders();
          return;
        } catch (_) {}
      }
      statusLabel.textContent = "Failed to start";
    } catch (e) {
      statusLabel.textContent = "Failed to start";
      console.error("spawn failed:", e);
    }
  }
});

// Init
loadFocus();
checkOwnership().then(() => refreshFolders());
