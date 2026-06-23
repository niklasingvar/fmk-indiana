import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

// State
let _$daemonIsOurs = false;

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
  try {
    focusInput.value = await invoke("read_focus");
  } catch (_) {}
}

async function saveFocus() {
  try {
    await invoke("save_focus", { text: focusInput.value });
  } catch (_) {}
}

focusInput.addEventListener("blur", saveFocus);
focusInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    focusInput.blur();
    saveFocus();
  }
});

// Refresh folder list from daemon
async function refreshFolders() {
  try {
    const resp = await invoke("status");
    const folders = resp.folders || [];
    renderFolders(folders);
    setRunning(true);
  } catch (_) {
    // Daemon not running
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
      count.textContent = f.count + "×";

      li.appendChild(name);
      li.appendChild(count);

      // Click to copy
      li.addEventListener("click", async () => {
        try {
          await invoke("copy_folder", { path: f.path });
          flashCopied();
        } catch (e) {
          console.error("copy failed:", e);
        }
      });

      // Right-click to remove
      li.addEventListener("contextmenu", async (e) => {
        e.preventDefault();
        try {
          await invoke("remove_folder", { path: f.path });
          await refreshFolders();
        } catch (err) {
          console.error("remove failed:", err);
        }
      });

      foldersList.appendChild(li);
    }
  }
}

function basename(path) {
  // Show ~/ for home dir, else basename
  const home = "/Users/" + (new URLSearchParams(window.location.search).get("user") || "");
  if (home && path.startsWith(home)) {
    return "~" + path.slice(home.length);
  }
  const parts = path.split("/");
  return parts[parts.length - 1] || path;
}

function flashCopied() {
  copiedFlash.classList.add("show");
  setTimeout(() => copiedFlash.classList.remove("show"), 1200);
}

// Server status display
function setRunning(running) {
  if (running) {
    statusDot.className = "running";
    statusLabel.textContent = "Server running";
  } else {
    statusDot.className = "stopped";
    statusLabel.textContent = "Server stopped";
    // Grey out folder list
    renderFolders([]);
    emptyState.hidden = true;
  }
}

function setConnecting() {
  statusDot.className = "";
  statusDot.innerHTML = '<span class="spinner"></span>';
  statusLabel.textContent = "Starting…";
}

// Add folder
addBtn.addEventListener("click", async () => {
  try {
    const dir = await open({ directory: true, multiple: false, title: "Monitor a folder" });
    if (dir) {
      await invoke("add_folder", { path: dir });
      await refreshFolders();
    }
  } catch (e) {
    console.error("add folder failed:", e);
  }
});

emptyState.addEventListener("click", () => addBtn.click());

// Start/stop button
statusAction.addEventListener("click", async () => {
  const isRunning = statusDot.className === "running";
  if (isRunning) {
    try {
      await invoke("shutdown");
      setRunning(false);
    } catch (_) {}
  } else {
    setConnecting();
    try {
      await invoke("spawn_sidecar");
      // Poll for daemon to come up
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
refreshFolders();
