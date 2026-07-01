import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { getVersion } from "@tauri-apps/api/app";

// State
let homeDir = "";
let openFolderMenu = null;
let connecting = false;

// DOM
const foldersList = document.getElementById("folders");
const addBtn = document.getElementById("add-item");
const statusDot = document.getElementById("status-dot");
const statusLabel = document.getElementById("status-label");
const statusAction = document.getElementById("status-action");
const versionEl = document.getElementById("version");
const copiedFlash = document.getElementById("copied-flash");

async function loadHomeDir() {
  try { homeDir = await invoke("home_dir"); } catch (_) {}
}

// Refresh folder list from daemon. Always renders — the `connecting` guard
// lives on the background poll (the only caller that must not clobber a
// spawn-in-progress); explicit callers want the result immediately.
async function refreshFolders() {
  try {
    const resp = await invoke("status");
    renderFolders(resp.folders || []);
    setRunning(true, resp.stoppable);
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

function flashText(msg, ms = 1200) {
  copiedFlash.textContent = msg;
  copiedFlash.classList.add("show");
  setTimeout(() => copiedFlash.classList.remove("show"), ms);
}

function firstLine(err) {
  return String(err).split("\n")[0].slice(0, 48);
}
function renderFolders(folders) {
  foldersList.innerHTML = "";
  if (folders.length === 0) { foldersList.hidden = true; return; }
  foldersList.hidden = false;
  for (const f of folders) {
      const li = document.createElement("li");
      li.className = "folder";

      const name = document.createElement("span");
      name.className = "name";
      name.textContent = basename(f.path);

      const count = document.createElement("span");
      count.className = "count";
      count.textContent = f.count + " ::";

      const runBtn = document.createElement("button");
      runBtn.className = "copy";
      runBtn.textContent = "run";
      runBtn.addEventListener("click", async (e) => {
        e.stopPropagation();
        try {
          const text = await invoke("copy_folder", { path: f.path });
          await writeText(text);
          flashText("copied");
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
          const ok = await invoke("refresh_templates", { path: f.path });
          flashText(ok ? "updated" : "failed");
        } catch (err) { console.error("refresh failed:", err); flashText("failed: " + firstLine(err), 3500); }
      });
      menu.appendChild(refreshItem);

      const replaceItem = document.createElement("button");
      replaceItem.textContent = "replace indiana commands";
      replaceItem.addEventListener("click", async (e) => {
        e.stopPropagation();
        closeFolderMenu();
        try {
          const ok = await invoke("replace_templates", { path: f.path });
          flashText(ok ? "replaced" : "failed", ok ? 1200 : 3500);
        } catch (err) { console.error("replace failed:", err); flashText("failed: " + firstLine(err), 3500); }
      });
      menu.appendChild(replaceItem);

      const copyActionsItem = document.createElement("button");
      copyActionsItem.textContent = "copy actions";
      copyActionsItem.addEventListener("click", async (e) => {
        e.stopPropagation();
        closeFolderMenu();
        try {
          const text = await invoke("copy_folder", { path: f.path, kind: "action" });
          await writeText(text);
          flashText("copied");
        } catch (err) { console.error("copy actions failed:", err); }
      });
      menu.appendChild(copyActionsItem);

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
      li.appendChild(runBtn);
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

function setRunning(running, stoppable = false) {
  connecting = false;
  statusDot.className = running ? "running" : "stopped";
  statusLabel.textContent = running ? "server running" : "server stopped";
  if (!running) {
    foldersList.innerHTML = "";
    foldersList.hidden = true;
  }
  updateActionButton(running, stoppable);
}

function updateActionButton(running, stoppable) {
  // The daemon tells us whether it can be cleanly stopped (StatusResponse.
  // stoppable). A supervised daemon (launchd KeepAlive) reports false, since a
  // Shutdown would just be restarted — hide stop and let launchctl manage it.
  if (running && !stoppable) {
    statusAction.style.display = "none";
    return;
  }
  statusAction.textContent = running ? "stop" : "start";
  statusAction.style.display = "";
}
function setConnecting() {
  connecting = true;
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
  if (isRunning) {
    try {
      await invoke("shutdown");
      setRunning(false);
    } catch (_) {}
  } else {
    setConnecting();
    // spawn_sidecar blocks until the daemon answers or errors, so we don't poll
    // here — one source of truth. On failure it returns the real reason.
    try {
      await invoke("spawn_sidecar");
      await refreshFolders();
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
  let connected = false;
  for (let i = 0; i < 20; i++) {
    try {
      await invoke("status");
      await refreshFolders();
      connected = true;
      break;
    } catch (_) {}
    await new Promise(r => setTimeout(r, 500));
  }
  // Only fall to the stopped state if we never reached the daemon — otherwise
  // refreshFolders() already rendered "running" and we'd clobber it.
  if (!connected) setRunning(false);
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

// Periodic polling — skipped while a spawn is in flight so it can't overwrite
// the "starting…" state with a transient "stopped".
setInterval(() => { if (!connecting) refreshFolders(); }, 3000);
