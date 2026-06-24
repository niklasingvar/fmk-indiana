import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { getVersion } from "@tauri-apps/api/app";

// State
let daemonIsOurs = false;
let homeDir = "";
let openFolderMenu = null;

// DOM
const foldersList = document.getElementById("folders");
const addBtn = document.getElementById("add-item");
const statusDot = document.getElementById("status-dot");
const statusLabel = document.getElementById("status-label");
const statusAction = document.getElementById("status-action");
const versionEl = document.getElementById("version");
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


function closeFolderMenu() {
  if (openFolderMenu) {
    openFolderMenu.hidden = true;
    openFolderMenu = null;
  }
}

function flashText(msg) {
  copiedFlash.textContent = msg;
  copiedFlash.classList.add("show");
  setTimeout(() => copiedFlash.classList.remove("show"), 1200);
}
function renderFolders(folders) {
  foldersList.innerHTML = "";
  if (folders.length === 0) return;
  for (const f of folders) {
      const li = document.createElement("li");
      li.className = "folder";

      const name = document.createElement("span");
      name.className = "name";
      name.textContent = basename(f.path);

      const count = document.createElement("span");
      count.className = "count";
      count.textContent = f.count + " ::";

      const copy = document.createElement("button");
      copy.className = "copy";
      copy.textContent = "copy";
      copy.addEventListener("click", async (e) => {
        e.stopPropagation();
        try {
          const text = await invoke("copy_folder", { path: f.path });
          await writeText(text);
          flashCopied();
        } catch (err) { console.error("copy failed:", err); }
      });

      const menuBtn = document.createElement("button");
      menuBtn.className = "menu-btn";
      menuBtn.textContent = "\u22ef";
      const menu = document.createElement("div");
      menu.className = "folder-menu";
      menu.hidden = true;

      const refreshItem = document.createElement("button");
      refreshItem.textContent = "update indiana commands";
      refreshItem.addEventListener("click", async (e) => {
        e.stopPropagation();
        closeFolderMenu();
        try {
          await invoke("refresh_templates", { path: f.path });
          flashText("updated");
        } catch (err) { console.error("refresh failed:", err); }
      });
      menu.appendChild(refreshItem);

      const removeItem = document.createElement("button");
      removeItem.textContent = "remove folder";
      removeItem.addEventListener("click", async (e) => {
        e.stopPropagation();
        closeFolderMenu();
        try {
          await invoke("remove_folder", { path: f.path });
          await refreshFolders();
        } catch (err) { console.error("remove failed:", err); }
      });
      menu.appendChild(removeItem);

      menuBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        if (openFolderMenu && openFolderMenu !== menu) {
          openFolderMenu.hidden = true;
        }
        menu.hidden = !menu.hidden;
        openFolderMenu = menu.hidden ? null : menu;
      });

      li.appendChild(name);
      li.appendChild(count);
      li.appendChild(copy);
      li.appendChild(menuBtn);
      li.appendChild(menu);
      foldersList.appendChild(li);
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
  statusLabel.textContent = running ? "server running" : "server stopped";
  if (!running) {
    foldersList.innerHTML = "";
  }
  updateActionButton(running);
}

function updateActionButton(running) {
  if (running) {
    if (daemonIsOurs) {
      statusAction.textContent = "stop";
      statusAction.style.display = "";
    } else {
      statusAction.style.display = "none";
    }
  } else {
    statusAction.textContent = "start";
    statusAction.style.display = "";
  }
}

function setConnecting() {
  statusDot.className = "spinning";
  statusLabel.textContent = "starting\u2026";
  statusAction.style.display = "none";
}

// Add folder
addBtn.addEventListener("click", async () => {
  try {
    try { await invoke("set_dialog_open", { open: true }); } catch (_) {}
    const dir = await open({ directory: true, multiple: false, title: "Monitor a folder" });
    if (dir) {
      await invoke("add_folder", { path: dir });
      await refreshFolders();
    }
  } catch (e) { console.error("add folder failed:", e); }
  finally { try { await invoke("set_dialog_open", { open: false }); } catch (_) {} }
});


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
      statusLabel.textContent = "failed to start";
    } catch (e) {
      setRunning(false);
      statusLabel.textContent = "failed to start";
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
      break;
    } catch (_) {}
    await new Promise(r => setTimeout(r, 500));
  }
  setRunning(false);
  try {
    versionEl.textContent = "v" + await getVersion();
  } catch (_) {
    versionEl.textContent = "v0.1.0";
  }
});

// Theme switcher (cogwheel)
const themeCog = document.getElementById("theme-cog");
const themeMenu = document.getElementById("theme-menu");

function applyTheme(choice) {
  document.documentElement.dataset.theme = choice;
  try { localStorage.setItem("indiana.theme", choice); } catch (_) {}
  markTheme(choice);
}

function markTheme(choice) {
  for (const btn of themeMenu.querySelectorAll("[data-theme-choice]")) {
    btn.querySelector(".mark").textContent =
      btn.dataset.themeChoice === choice ? "›" : "";
  }
}

themeCog.addEventListener("click", (e) => {
  e.stopPropagation();
  themeMenu.hidden = !themeMenu.hidden;
});

for (const btn of themeMenu.querySelectorAll("[data-theme-choice]")) {
  btn.addEventListener("click", () => {
    applyTheme(btn.dataset.themeChoice);
    themeMenu.hidden = true;
  });
}

document.addEventListener("click", () => { themeMenu.hidden = true; closeFolderMenu(); });

markTheme(document.documentElement.dataset.theme || "system");

// Periodic polling
setInterval(refreshFolders, 3000);
