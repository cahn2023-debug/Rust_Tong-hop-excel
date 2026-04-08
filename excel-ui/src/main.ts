import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, save as tauriSave } from "@tauri-apps/plugin-dialog";
import { state, normalizePath, DiscoveredFile } from "./state";
import * as api from "./api";
import * as ui from "./ui";

const appWindow = getCurrentWindow();

// --- Core Logic ---

function openMappingSidebar(path: string) {
  const sidebar = document.querySelector(ui.selectors.sidebar);
  if (!sidebar) return;
  state.currentMappingPath = path;
  const shortName = path.split(/[\\/]/).pop();

  const filenameEl = document.querySelector(ui.selectors.sidebarFilename);
  if (filenameEl) filenameEl.textContent = shortName || path;

  const override = state.mappingOverrides[path];
  (document.querySelector(ui.selectors.mapStt) as HTMLSelectElement).value = override?.stt?.toString() || "";
  (document.querySelector(ui.selectors.mapName) as HTMLSelectElement).value = override?.name?.toString() || "";
  (document.querySelector(ui.selectors.mapUnit) as HTMLSelectElement).value = override?.unit?.toString() || "";
  (document.querySelector(ui.selectors.mapQty) as HTMLSelectElement).value = override?.qty?.toString() || "";

  sidebar.classList.add('active');
}

async function runConsolidation() {
  const inputDir = (document.querySelector(ui.selectors.inputDir) as HTMLInputElement)?.value;
  const templateFile = (document.querySelector(ui.selectors.templateFile) as HTMLInputElement)?.value;
  const skipRows = parseInt((document.querySelector(ui.selectors.skipRows) as HTMLInputElement)?.value || "0");

  if (!inputDir || state.isScanning) {
    ui.addLog("Input directory is required.", "error");
    return;
  }

  state.isScanning = true;
  state.totalRecords = 0;
  state.fileCount = 0;
  state.allSummaryRecords = [];

  // Clear UI
  const fileListBody = document.querySelector(ui.selectors.fileListBody);
  const previewBody = document.querySelector(ui.selectors.previewBody);
  if (fileListBody) fileListBody.innerHTML = '';
  if (previewBody) previewBody.innerHTML = '';
  document.querySelector(ui.selectors.previewPlaceholder)?.classList.remove('hidden');

  ui.addLog(`Starting consolidation for: ${inputDir}`, "system");
  ui.switchTab("logs");

  try {
    await api.callRunConsolidation(inputDir, templateFile, skipRows);
    ui.addLog("Consolidation process successful.", "success");
  } catch (error) {
    ui.addLog(`Engine Failure: ${error}`, "error");
  } finally {
    state.isScanning = false;
    (document.querySelector(ui.selectors.exportBtn) as HTMLButtonElement).disabled = false;
  }
}

// --- Event Handlers & Listeners ---

function applyFiltersAndSort() {
  let data = [...state.allSummaryRecords];

  Object.keys(state.tableFilters).forEach(key => {
    const term = state.tableFilters[key].toLowerCase();
    if (term) {
      data = data.filter(row => String(row[key] || '').toLowerCase().includes(term));
    }
  });

  if (state.tableSort) {
    const { key, dir } = state.tableSort;
    data.sort((a, b) => {
      let valA = a[key];
      let valB = b[key];
      if (key === 'khoi_luong' || key === 'stt') {
        const numA = parseFloat(valA) || 0;
        const numB = parseFloat(valB) || 0;
        return dir === 'asc' ? numA - numB : numB - numA;
      }
      valA = String(valA || '').toLowerCase();
      valB = String(valB || '').toLowerCase();
      return dir === 'asc' ? valA.localeCompare(valB) : valB.localeCompare(valA);
    });
  }

  ui.renderPreviewTable(data);
}

function handleFileDiscovered(file: DiscoveredFile) {
  state.fileCount++;
  const shortName = file.path.split(/[\\/]/).pop() || file.path;
  const item = document.createElement("div");
  item.className = "file-list-item";
  const normPath = normalizePath(file.path);
  item.dataset.path = file.path;
  item.dataset.normPath = normPath;

  const fileId = file.path.replace(/[^a-zA-Z0-9]/g, '-');
  item.innerHTML = `
    <div class="file-context-wrapper">
      <div class="file-icon"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg></div>
      <div class="file-info">
        <div style="display: flex; align-items: center; gap: 6px;">
          <span class="file-name" title="${file.path}">${shortName}</span>
          <span class="lech-badge hidden" id="badge-${fileId}" style="background: #ef4444; color: white; font-size: 8px; padding: 1px 4px; border-radius: 4px; font-weight: bold;">LỆCH</span>
        </div>
      </div>
      <div class="contribution-badge" id="contrib-${fileId}">0</div>
    </div>
    <div class="file-actions">
      <button class="secondary-btn small" style="padding: 2px 6px; font-size: 9px;" data-path="${file.path}">Map</button>
    </div>
  `;

  item.querySelector('.file-actions button')?.addEventListener('click', (e) => {
    e.stopPropagation();
    openMappingSidebar(file.path);
  });

  item.addEventListener("contextmenu", (e) => showFileContextMenu(e, file.path));

  item.addEventListener('click', async () => {
    document.querySelectorAll('.file-list-item').forEach(i => i.classList.remove('selected'));
    item.classList.add('selected');

    // Clear previous source highlights from summary table
    document.querySelector(ui.selectors.previewBody)?.querySelectorAll('tr').forEach(r => {
      r.classList.remove('source-highlight');
    });

    const fileNameEl = document.querySelector(ui.selectors.currentDetailFile);
    if (fileNameEl) fileNameEl.textContent = shortName;
    document.querySelector(ui.selectors.detailPlaceholder)?.classList.add('hidden');

    const detailBody = document.querySelector(ui.selectors.detailBody) as HTMLElement;
    detailBody.innerHTML = '<tr><td colspan="4" style="text-align:center; padding: 20px;">Loading data...</td></tr>';
    // ... rest of preview logic

    try {
      const skipRows = parseInt((document.querySelector(ui.selectors.skipRows) as HTMLInputElement)?.value || "0");
      const data = await api.callGetFileData(file.path, skipRows);
      ui.renderDetailTable(data);
    } catch (err) {
      ui.addLog(`Failed to load file preview: ${err}`, "error");
      detailBody.innerHTML = '<tr><td colspan="4" style="text-align:center; color: var(--error);">Error loading data</td></tr>';
    }
  });

  document.querySelector(ui.selectors.fileListBody)?.prepend(item);
  ui.addLog(`Scanned: ${shortName}`, "system");
}

function handleFileParsed(payload: any) {
  const { path, records, progress, summary, analysis, message } = payload;
  const shortName = path.split(/[\\/]/).pop();

  const progressBar = document.querySelector(ui.selectors.progressBar) as HTMLElement;
  const progressPercent = document.querySelector(ui.selectors.progressPercent) as HTMLElement;
  const progressStatus = document.querySelector(ui.selectors.progressStatus) as HTMLElement;

  if (progressBar) progressBar.style.width = `${progress}%`;
  if (progressPercent) progressPercent.innerText = `${Math.round(progress)}%`;
  if (progressStatus) progressStatus.innerText = `${Math.round(progress * state.fileCount / 100)} / ${state.fileCount}`;

  state.totalRecords += records;
  const recordsEl = document.querySelector(ui.selectors.recordsValue);
  if (recordsEl) recordsEl.textContent = `${state.totalRecords} Records`;

  const fileItem = document.querySelector(`.file-list-item[data-path="${path.replace(/\\/g, '\\\\')}"]`) as HTMLElement;
  if (fileItem) {
    fileItem.classList.remove('valid-data-status', 'warning-status');
    fileItem.classList.add('processed-status');

    if (analysis) {
      if (analysis.has_valid_data && !analysis.has_zero_data) {
        fileItem.classList.add('valid-data-status');
      } else if (analysis.has_zero_data) {
        fileItem.classList.add('warning-status');
        ui.addLog(`[Warning] File ${shortName} contains zero/null data rows.`, "error");
      }

      if (analysis.is_deviant) {
        const fileId = path.replace(/[^a-zA-Z0-9]/g, '-');
        document.getElementById(`badge-${fileId}`)?.classList.remove('hidden');
        ui.addLog(`[Analysis] File ${shortName} is deviant: ${analysis.reason}`, "error");
      }

      if (analysis.detected_columns) {
        const cols = analysis.detected_columns;
        const msg = `[Mapping] ${shortName} -> ${cols.name || '?'} | ${cols.qty || '?'}`;
        ui.addLog(msg, analysis.has_valid_data ? "system" : "error");
      }
    }
  }

  if (records > 0) {
    ui.addLog(`Extracted ${records} records from ${shortName}`, "success");
  } else {
    ui.addLog(`Skipped ${shortName}: ${message || 'No records extracted'}`, "error");
  }

  document.querySelector(ui.selectors.previewPlaceholder)?.classList.add('hidden');
  state.allSummaryRecords = summary || [];
  applyFiltersAndSort();
}

// --- Context Menu Management ---
let currentContextPath: string | null = null;

export function getCurrentContextPath() { return currentContextPath; }

function showFileContextMenu(e: MouseEvent, path: string) {
  const menu = document.getElementById("file-context-menu");
  if (!menu) return;
  e.preventDefault(); e.stopPropagation();
  currentContextPath = path;
  menu.style.display = "flex";
  menu.style.visibility = "hidden";
  const menuWidth = menu.offsetWidth, menuHeight = menu.offsetHeight;
  const windowWidth = window.innerWidth, windowHeight = window.innerHeight;
  let x = Math.min(e.clientX, windowWidth - menuWidth - 10);
  let y = Math.min(e.clientY, windowHeight - menuHeight - 10);
  menu.style.visibility = "visible";
  menu.style.top = `${y}px`; menu.style.left = `${x}px`;
}

function initApp() {
  window.addEventListener("contextmenu", (e) => e.preventDefault());

  api.setupBackendListeners({
    onFileDiscovered: handleFileDiscovered,
    onFileParsed: handleFileParsed,
    onError: (msg) => ui.addLog(msg, "error")
  });

  // Context Menu Item Listeners
  document.getElementById("menu-open-file")?.addEventListener("click", () => {
    if (currentContextPath) {
      api.callOpenFile(currentContextPath);
      const menu = document.getElementById("file-context-menu");
      if (menu) menu.style.display = "none";
    }
  });

  document.getElementById("menu-open-folder")?.addEventListener("click", () => {
    if (currentContextPath) {
      api.callOpenFolder(currentContextPath);
      const menu = document.getElementById("file-context-menu");
      if (menu) menu.style.display = "none";
    }
  });

  // Hide context menu on global click
  window.addEventListener("click", () => {
    const menu = document.getElementById("file-context-menu");
    if (menu) menu.style.display = "none";
  });

  // UI Element Listeners
  document.getElementById("browse-input-btn")?.addEventListener("click", async () => {
    const selected = await open({ directory: true, multiple: false, title: "Chọn thư mục chứa file Excel" });
    if (selected) (document.querySelector(ui.selectors.inputDir) as HTMLInputElement).value = selected as string;
  });

  document.getElementById("browse-template-btn")?.addEventListener("click", async () => {
    const selected = await open({
      directory: false, multiple: false, title: "Chọn file mẫu",
      filters: [{ name: "Excel", extensions: ["xlsx", "xls"] }]
    });
    if (selected) {
      (document.querySelector(ui.selectors.templateFile) as HTMLInputElement).value = selected as string;
      const skipRows = parseInt((document.querySelector(ui.selectors.skipRows) as HTMLInputElement)?.value || "0");
      try {
        const headers = await api.callGetTemplateHeaders(selected as string, skipRows);
        const headerRow = document.querySelector(`${ui.selectors.previewHeader} tr`);
        if (headerRow && headers?.length > 0) {
          headerRow.innerHTML = headers.map(h => `<th>${h}</th>`).join("");
          ui.addLog("Table headers updated from template.", "system");
        }
      } catch (e) {
        ui.addLog(`Failed to load template headers: ${e}`, "error");
      }
    }
  });

  document.getElementById("export-btn")?.addEventListener("click", async () => {
    try {
      const filePath = await tauriSave({
        filters: [{ name: "Excel", extensions: ["xlsx"] }],
        title: "Xuất kết quả tổng hợp",
        defaultPath: "Tonghop_Ketqua.xlsx"
      });
      if (filePath) {
        ui.addLog(`Exporting to: ${filePath.split(/[\\/]/).pop()}`, "system");
        await api.callFinalizeExport("latest_result.xlsx", filePath);
        ui.addLog("Export successful!", "success");
        await api.callOpenResultFolder(filePath);
      }
    } catch (e) {
      ui.addLog(`Export failed: ${e}`, "error");
    }
  });

  document.querySelector(ui.selectors.runBtn)?.addEventListener("click", () => runConsolidation());

  // Window Controls
  document.getElementById('titlebar-minimize')?.addEventListener('click', () => appWindow.minimize());
  document.getElementById('titlebar-maximize')?.addEventListener('click', () => appWindow.toggleMaximize());
  document.getElementById('titlebar-close')?.addEventListener('click', () => appWindow.close());

  // Clear Cache Action
  document.getElementById("clear-cache-btn")?.addEventListener("click", async () => {
    if (confirm("Xóa toàn bộ cache hiệu năng để quét lại từ đầu?")) {
      try {
        await api.callClearCache();
        ui.addLog("Cache cleared.", "system");
        alert("Đã xóa cache thành công!");
      } catch (err: any) {
        ui.addLog(`Error clearing cache: ${err}`, "error");
      }
    }
  });

  // Tabs & Sidebar
  document.querySelectorAll(ui.selectors.tabs).forEach(tab => {
    tab.addEventListener('click', (e) => {
      const tabId = (e.currentTarget as HTMLElement).getAttribute('data-tab');
      if (tabId) ui.switchTab(tabId);
    });
  });

  document.querySelector(ui.selectors.sidebarClose)?.addEventListener('click', () => {
    document.querySelector(ui.selectors.sidebar)?.classList.remove('active');
  });

  document.querySelector(ui.selectors.saveMappingBtn)?.addEventListener('click', () => {
    if (!state.currentMappingPath) return;
    const getVal = (sel: string) => (document.querySelector(sel) as HTMLSelectElement).value;
    state.mappingOverrides[state.currentMappingPath] = {
      stt: getVal(ui.selectors.mapStt) === "" ? null : parseInt(getVal(ui.selectors.mapStt), 10),
      name: getVal(ui.selectors.mapName) === "" ? null : parseInt(getVal(ui.selectors.mapName), 10),
      unit: getVal(ui.selectors.mapUnit) === "" ? null : parseInt(getVal(ui.selectors.mapUnit), 10),
      qty: getVal(ui.selectors.mapQty) === "" ? null : parseInt(getVal(ui.selectors.mapQty), 10)
    };
    document.querySelector(ui.selectors.sidebar)?.classList.remove('active');
    ui.addLog(`Mapping saved for ${state.currentMappingPath.split(/[\\/]/).pop()}`, "success");
  });

  // Table Interaction (Filter/Sort)
  const filterMenu = document.getElementById('column-filter-menu') as HTMLElement;
  const searchInput = document.getElementById('filter-search-input') as HTMLInputElement;
  let activeCol: string | null = null;

  document.querySelectorAll('.filter-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const col = (btn as HTMLElement).getAttribute('data-col');
      if (!col) return;
      activeCol = col;
      const rect = (btn as HTMLElement).getBoundingClientRect();
      filterMenu.style.top = `${rect.bottom + 5}px`;
      filterMenu.style.left = `${Math.min(window.innerWidth - 210, rect.left)}px`;
      filterMenu.classList.add('active');
      searchInput.value = state.tableFilters[activeCol] || '';
      searchInput.focus();
    });
  });

  document.getElementById('sort-asc')?.addEventListener('click', () => {
    if (activeCol) { state.tableSort = { key: activeCol, dir: 'asc' }; applyFiltersAndSort(); }
    filterMenu.classList.remove('active');
  });
  document.getElementById('sort-desc')?.addEventListener('click', () => {
    if (activeCol) { state.tableSort = { key: activeCol, dir: 'desc' }; applyFiltersAndSort(); }
    filterMenu.classList.remove('active');
  });
  searchInput.addEventListener('input', () => {
    if (activeCol) { state.tableFilters[activeCol] = searchInput.value; applyFiltersAndSort(); }
  });

  // Resizers
  ui.setupResizer(document.getElementById('resizer-left')!, (x) => {
    (document.querySelector('.left-pane') as HTMLElement).style.width = `${Math.max(250, Math.min(600, x))}px`;
  });
  ui.setupResizer(document.getElementById('resizer-right')!, (x) => {
    (document.querySelector('.right-pane') as HTMLElement).style.width = `${Math.max(200, Math.min(500, window.innerWidth - x))}px`;
  });
  // Highlight Logic from Summary Table
  const previewBody = document.querySelector(ui.selectors.previewBody);
  previewBody?.addEventListener('click', (e) => {
    const row = (e.target as HTMLElement).closest('tr');
    if (!row) return;

    // Toggle active row class
    previewBody.querySelectorAll('tr').forEach(r => r.classList.remove('active-row'));
    row.classList.add('active-row');

    const jobName = row.getAttribute('data-name');
    state.selectedJobName = jobName;

    // Update active-row in detail table (File Detail Preview)
    const detailBody = document.querySelector(ui.selectors.detailBody);
    if (detailBody && jobName) {
      detailBody.querySelectorAll('tr').forEach(r => {
        if (r.getAttribute('data-name') === jobName) {
          r.classList.add('active-row');
          r.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        } else {
          r.classList.remove('active-row');
        }
      });
    }

    const sourcesStr = row.getAttribute('data-sources') || '';
    const sourcesData = sourcesStr.split(';')
      .map((s: string) => {
        const trimmed = s.trim();
        const match = trimmed.match(/^(.*)\s*\(([\d.]+)\)$/);
        if (match) {
          return { path: normalizePath(match[1].trim()), qty: parseFloat(match[2]) };
        }
        return { path: normalizePath(trimmed), qty: 0 };
      })
      .filter((item: { path: string; qty: number }) => item.qty > 0);

    const sourcePaths = sourcesData.map((item: { path: string; qty: number }) => item.path);

    // Highlight source files
    let firstHighlighted: HTMLElement | null = null;
    document.querySelectorAll('.file-list-item').forEach(el => {
      const item = el as HTMLElement;
      const itemPath = item.dataset.normPath || '';
      const badge = document.getElementById(`contrib-${item.dataset.path?.replace(/[^a-zA-Z0-9]/g, '-')}`);

      if (sourcePaths.includes(itemPath)) {
        item.classList.add('highlighted');
        if (!firstHighlighted) firstHighlighted = item;

        const source = sourcesData.find((s: { path: string; qty: number }) => s.path === itemPath);
        if (badge && source) {
          badge.textContent = source.qty.toLocaleString();
          badge.style.display = 'flex';
        }
      } else {
        item.classList.remove('highlighted');
        if (badge) badge.style.display = 'none';
      }
    });

    // Auto-scroll to first highlighted file
    if (firstHighlighted) {
      (firstHighlighted as HTMLElement).scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }

    ui.addLog(`Viewing sources for: ${row.getAttribute('data-name') || 'unknown'}`, "system");
  });
}

window.addEventListener("DOMContentLoaded", initApp);
