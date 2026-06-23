import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

// State
let daemonIsOurs = false;
let homeDir = "";

// DOM
const foldersList = document.getElementById("folders");
const emptyState = document.getElementById("empty");
const addBtn = document.getElementById("add-item");
const statusDot = document.getElementById("status-dot");
const statusLabel = document.getElementById("status-label");
const statusAction = document.getElementById("status-action");
const copiedFlash = document.getElementById("copied-flash");

// Check if we own the daemon
async function checkOwnership() {
  try { daemonIsOurs = await invoke("daemon_is_ours"); } catch (_) { daemonIsOurs = false; }
}

async function loadHomeDir() {
  try { homeDir = await invoke("home_dir"); } catch (_) {}
}

// Refresh folder list from daemon
async function refreshFolders() {
  try {
    const resp = await invoke("status");
    try { daemonIsOurs = await invoke("daemon_is_ours"); } catch (_) {}
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
      li.className = "menu-item";

      const label = document.createElement("span");
      label.className = "label";
      label.textContent = basename(f.path);

      const count = document.createElement("span");
      count.className = "meta";
      count.textContent = f.count + "\u00D7";

      li.appendChild(label);
      li.appendChild(count);

      li.addEventListener("click", async () => {
        try {
          const text = await invoke("copy_folder", { path: f.path });
          await writeText(text);
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
  if (homeDir && (path === homeDir || path.startsWith(homeDir + "/"))) return "~" + path.slice(homeDir.length);
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
  }
  updateActionButton(running);
}

function updateActionButton(running) {
  if (running) {
    if (daemonIsOurs) {
      statusAction.textContent = "Stop";
      statusAction.style.display = "";
    } else {
      statusAction.style.display = "none";
    }
  } else {
    statusAction.textContent = "Start";
    statusAction.style.display = "";
  }
}

function setConnecting() {
  statusDot.className = "spinning";
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
      setRunning(false);
      statusLabel.textContent = "Failed to start";
    } catch (e) {
      setRunning(false);
      statusLabel.textContent = "Failed to start";
      console.error("spawn failed:", e);
    }
  }
});

// Init
loadHomeDir().then(async () => {
  setConnecting();
  for (let i = 0; i < 20; i++) {
    try {
      await invoke("status");
      await checkOwnership();
      await refreshFolders();
      return;
    } catch (_) {}
    await new Promise(r => setTimeout(r, 500));
  }
  setRunning(false);
});

// Periodic polling
setInterval(refreshFolders, 3000);
