import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { open, save as tauriSave } from "@tauri-apps/plugin-dialog";

const appWindow = getCurrentWindow();

interface DiscoveredFile {
  path: string;
  size: number;
  cached: boolean;
}

interface MappingOverride {
  stt: number | null;
  name: number | null;
  unit: number | null;
  qty: number | null;
}

// State
let isScanning = false;
let fileCount = 0;
let mappingOverrides: Record<string, MappingOverride> = {};
let currentMappingPath: string | null = null;
let totalRecords = 0;
let allSummaryRecords: any[] = [];
let tableFilters: Record<string, string> = { stt: '', ten_cong_viec: '', don_vi: '', khoi_luong: '' };
let tableSort: { key: string, dir: 'asc' | 'desc' } | null = null;

// UI Selectors
const selectors = {
  runBtn: "#run-btn",
  inputDir: "#input-dir",
  outputFile: "#output-file",
  templateFile: "#template-file",
  skipRows: "#skip-rows",
  progressBar: "#progress-bar",
  progressPercent: "#progress-percent",
  progressStatus: "#progress-status",
  fileListBody: "#file-list-items",
  recordsValue: "#stats-records-badge",
  tabs: ".tab[data-tab]",
  panes: ".tab-pane",
  sidebar: "#mapping-sidebar",
  sidebarClose: "#close-sidebar",
  sidebarFilename: "#sidebar-filename",
  saveMappingBtn: "#save-mapping",
  mapStt: "#map-stt",
  mapName: "#map-name",
  mapUnit: "#map-unit",
  mapQty: "#map-qty",
  errorLog: "#error-log",
  summaryStt: "#summary-stt",
  summaryName: "#summary-name",
  summaryQty: "#summary-qty",
  previewBody: "#preview-body",
  previewPlaceholder: "#preview-placeholder",
  exportBtn: "#export-btn",
  previewHeader: "#preview-header"
};

function switchTab(tabId: string) {
  const tabs = document.querySelectorAll(selectors.tabs);
  const panes = document.querySelectorAll(selectors.panes);

  tabs.forEach(t => t.classList.remove('active'));
  panes.forEach(p => (p as HTMLElement).classList.remove('active'));

  const activeTab = document.querySelector(`.tab[data-tab="${tabId}"]`);
  const activePane = document.getElementById(`tab-${tabId}`);

  if (activeTab) activeTab.classList.add('active');
  if (activePane) activePane.classList.add('active');
}

function generateColOptions() {
  const selects = [selectors.mapStt, selectors.mapName, selectors.mapUnit, selectors.mapQty];
  const alph = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".split("");

  selects.forEach(sel => {
    const el = document.querySelector(sel) as HTMLSelectElement;
    if (!el) return;
    el.innerHTML = '<option value="">(Auto-Detect)</option>';
    alph.forEach((letter, idx) => {
      const opt = document.createElement("option");
      opt.value = idx.toString();
      opt.textContent = `Column ${letter}`;
      el.appendChild(opt);
    });
  });
}

function updateConfigSummary() {
  const stt = (document.querySelector(selectors.mapStt) as HTMLSelectElement)?.value || "Auto";
  const name = (document.querySelector(selectors.mapName) as HTMLSelectElement)?.value || "Auto";
  const qty = (document.querySelector(selectors.mapQty) as HTMLSelectElement)?.value || "Auto";

  const sttEl = document.querySelector(selectors.summaryStt);
  const nameEl = document.querySelector(selectors.summaryName);
  const qtyEl = document.querySelector(selectors.summaryQty);

  if (sttEl) sttEl.textContent = stt === "" ? "Auto" : `Col ${stt}`;
  if (nameEl) nameEl.textContent = name === "" ? "Auto" : `Col ${name}`;
  if (qtyEl) qtyEl.textContent = qty === "" ? "Auto" : `Col ${qty}`;
}

function addLog(msg: string, type: "system" | "success" | "error" = "system") {
  const logContainer = document.querySelector(selectors.errorLog);
  if (!logContainer) return;
  const entry = document.createElement("div");
  entry.className = `log-entry ${type}`;
  entry.innerText = `> [${new Date().toLocaleTimeString()}] ${msg}`;
  logContainer.prepend(entry);
}

function openMappingSidebar(path: string) {
  const sidebar = document.querySelector(selectors.sidebar);
  if (!sidebar) return;
  currentMappingPath = path;
  const shortName = path.split(/[\\/]/).pop();

  const filenameEl = document.querySelector(selectors.sidebarFilename);
  if (filenameEl) filenameEl.textContent = shortName || path;

  // Load existing override if any
  const override = mappingOverrides[path];
  (document.querySelector(selectors.mapStt) as HTMLSelectElement).value = override?.stt?.toString() || "";
  (document.querySelector(selectors.mapName) as HTMLSelectElement).value = override?.name?.toString() || "";
  (document.querySelector(selectors.mapUnit) as HTMLSelectElement).value = override?.unit?.toString() || "";
  (document.querySelector(selectors.mapQty) as HTMLSelectElement).value = override?.qty?.toString() || "";

  sidebar.classList.add('active');
}

async function runConsolidation() {
  const inputDirInput = document.querySelector(selectors.inputDir) as HTMLInputElement;
  const outputFileInput = document.querySelector(selectors.outputFile) as HTMLInputElement;
  const templateFileInput = document.querySelector(selectors.templateFile) as HTMLInputElement;
  const skipRowsInput = document.querySelector(selectors.skipRows) as HTMLInputElement;

  const inputDir = inputDirInput?.value;
  const outputFile = outputFileInput?.value;
  const templateFile = templateFileInput?.value;
  const skipRows = parseInt(skipRowsInput?.value || "0");

  if (!inputDir || isScanning) {
    addLog("Input directory is required.", "error");
    return;
  }

  isScanning = true;
  totalRecords = 0;
  fileCount = 0;

  // Clear lists
  const fileListBody = document.querySelector(selectors.fileListBody);
  const previewBody = document.querySelector(selectors.previewBody);
  if (fileListBody) fileListBody.innerHTML = '';
  if (previewBody) previewBody.innerHTML = '';
  const placeholder = document.querySelector(selectors.previewPlaceholder);
  if (placeholder) placeholder.classList.remove('hidden');

  addLog(`Starting consolidation for: ${inputDir}`, "system");
  switchTab("logs");

  try {
    await invoke("run_consolidation", {
      inputDir,
      outputFile: outputFile || "consolidated_output.xlsx",
      templatePath: templateFile || null,
      overrides: mappingOverrides,
      skipRows
    });
    addLog("Consolidation process successful.", "success");
  } catch (error) {
    addLog(`Engine Failure: ${error}`, "error");
  } finally {
    isScanning = false;
    const exportBtn = document.querySelector(selectors.exportBtn) as HTMLButtonElement;
    if (exportBtn) exportBtn.disabled = false;
  }
}

// Context Menu Logic
let currentContextPath: string | null = null;

function setupContextMenu() {
  const menu = document.getElementById("file-context-menu");
  const openFileBtn = document.getElementById("menu-open-file");
  const openFolderBtn = document.getElementById("menu-open-folder");

  if (!menu) return;

  // Hide menu on click elsewhere
  window.addEventListener("click", () => {
    menu.style.display = "none";
  });

  // Prevent context menu on the menu itself
  menu.addEventListener("contextmenu", (e) => e.preventDefault());

  openFileBtn?.addEventListener("click", async () => {
    if (currentContextPath) {
      try {
        await invoke("open_file", { path: currentContextPath });
      } catch (err) {
        addLog(`Failed to open file: ${err}`, "error");
      }
    }
  });

  openFolderBtn?.addEventListener("click", async () => {
    if (currentContextPath) {
      try {
        await invoke("open_folder", { path: currentContextPath });
      } catch (err) {
        addLog(`Failed to open folder: ${err}`, "error");
      }
    }
  });
}

function showFileContextMenu(e: MouseEvent, path: string) {
  const menu = document.getElementById("file-context-menu");
  if (!menu) return;

  e.preventDefault();
  e.stopPropagation();

  currentContextPath = path;

  // Show menu briefly to calculate dimensions
  menu.style.display = "flex";
  menu.style.visibility = "hidden";

  const menuWidth = menu.offsetWidth;
  const menuHeight = menu.offsetHeight;
  const windowWidth = window.innerWidth;
  const windowHeight = window.innerHeight;

  let x = e.clientX;
  let y = e.clientY;

  // Prevent menu from going off-screen
  if (x + menuWidth > windowWidth) {
    x = windowWidth - menuWidth - 10;
  }
  if (y + menuHeight > windowHeight) {
    y = windowHeight - menuHeight - 10;
  }

  menu.style.visibility = "visible";
  menu.style.top = `${y}px`;
  menu.style.left = `${x}px`;
}

async function setupEventListeners() {
  const fileListBody = document.querySelector(selectors.fileListBody);

  await listen<DiscoveredFile>("file_discovered", (event) => {
    fileCount++;
    const file = event.payload;
    const shortName = file.path.split(/[\\/]/).pop() || file.path;

    const item = document.createElement("div");
    item.className = "file-list-item";
    item.dataset.path = file.path;

    item.innerHTML = `
        <div class="file-context-wrapper">
            <div class="file-icon">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                    <polyline points="14 2 14 8 20 8"></polyline>
                    <line x1="9" y1="13" x2="15" y2="13"></line>
                    <line x1="9" y1="17" x2="15" y2="17"></line>
                    <line x1="9" y1="9" x2="10" y2="9"></line>
                </svg>
            </div>
            <div class="file-info">
                <div style="display: flex; align-items: center; gap: 6px;">
                    <span class="file-name" title="${file.path}">${shortName}</span>
                    <span class="lech-badge hidden" id="badge-${file.path.replace(/[^a-zA-Z0-9]/g, '-')}" 
                          style="background: #ef4444; color: white; font-size: 8px; padding: 1px 4px; border-radius: 4px; font-weight: bold;">LỆCH</span>
                </div>
            </div>
        </div>
        <div class="file-actions">
            <button class="secondary-btn small" style="padding: 2px 6px; font-size: 9px;" data-path="${file.path}">Map</button>
        </div>
    `;

    const mapBtn = item.querySelector('.file-actions button');
    mapBtn?.addEventListener('click', (e) => {
      e.stopPropagation();
      openMappingSidebar(file.path);
    });

    // Custom Context Menu
    item.addEventListener("contextmenu", (e) => {
      showFileContextMenu(e, file.path);
    });

    fileListBody?.prepend(item);
    addLog(`Scanned: ${shortName}`, "system");
  });

  await listen<{ path: string, records: number, progress: number, summary: any[], analysis: any, message?: string }>("file_parsed", (event) => {
    const { path, records, progress, summary, analysis, message } = event.payload;
    const shortName = path.split(/[\\/]/).pop();

    const progressBar = document.querySelector(selectors.progressBar) as HTMLElement;
    const progressPercent = document.querySelector(selectors.progressPercent) as HTMLElement;
    const progressStatus = document.querySelector(selectors.progressStatus) as HTMLElement;

    if (progressBar) progressBar.style.width = `${progress}%`;
    if (progressPercent) progressPercent.innerText = `${Math.round(progress)}%`;
    if (progressStatus) progressStatus.innerText = `${Math.round(progress)}%`;

    totalRecords += records;
    const recordsEl = document.querySelector(selectors.recordsValue);
    if (recordsEl) recordsEl.textContent = `${totalRecords} Records`;

    // Show "Lech" Badge if analysis says so
    const fileId = path.replace(/[^a-zA-Z0-9]/g, '-');
    if (analysis && analysis.is_deviant) {
      const badge = document.getElementById(`badge-${fileId}`);
      if (badge) badge.classList.remove('hidden');
      addLog(`[Analysis] File ${shortName} is deviant: ${analysis.reason}`, "error");
    } else if (analysis) {
      addLog(`[Analysis] File ${shortName} matches standard structure (${Math.round(analysis.confidence * 100)}%)`, "success");
    }

    // New: Highlight files with null/zero data
    const fileItem = document.querySelector(`.file-list-item[data-path="${path.replace(/\\/g, '\\\\')}"]`);
    if (fileItem && analysis && analysis.has_zero_data) {
      fileItem.classList.add('warning-status');
      addLog(`[Warning] File ${shortName} contains zero/null data rows.`, "error");
    }

    if (records > 0) {
      addLog(`Extracted ${records} records from ${shortName}`, "success");
    } else {
      addLog(`Skipped ${shortName}: ${message || 'No records extracted'}`, "error");
    }

    // Live Preview Logic (Aggregated Summary)
    const placeholder = document.querySelector(selectors.previewPlaceholder);
    if (placeholder) placeholder.classList.add('hidden');

    allSummaryRecords = summary || [];
    applyFiltersAndSort();
  });

  await listen<string>("process_error", (event) => {
    addLog(event.payload, "error");
  });

  // Highlight Logic
  const previewBody = document.querySelector(selectors.previewBody);
  previewBody?.addEventListener('click', (e) => {
    const row = (e.target as HTMLElement).closest('tr');
    if (!row) return;

    // Clear active row highlight if any
    document.querySelectorAll(`${selectors.previewBody} tr`).forEach(r => r.classList.remove('active-row'));
    row.classList.add('active-row');

    const sourcesStr = row.getAttribute('data-sources') || '';
    // Improved extraction: Extract everything before the last " (" in each source info
    const filenames = sourcesStr.split(';').map(s => {
      const part = s.trim();
      const lastIndex = part.lastIndexOf(' (');
      return lastIndex > -1 ? part.substring(0, lastIndex) : part;
    });

    // Clear all file highlights
    document.querySelectorAll('.file-list-item').forEach(item => item.classList.remove('highlighted'));

    // Highlight matching files
    filenames.forEach(name => {
      // Use double backslashes for querySelector and escape quotes
      const escapedPath = name.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
      const item = document.querySelector(`.file-list-item[data-path="${escapedPath}"]`);
      if (item) {
        item.classList.add('highlighted');
        item.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }
    });
  });
}

window.addEventListener("DOMContentLoaded", () => {
  // Disable context menu globally
  window.addEventListener("contextmenu", (e) => e.preventDefault());

  setupEventListeners();
  setupContextMenu(); // Initialize custom menu
  generateColOptions();

  // Native Dialogs
  document.getElementById("browse-input-btn")?.addEventListener("click", async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Chọn thư mục chứa file Excel"
    });
    if (selected) {
      (document.querySelector(selectors.inputDir) as HTMLInputElement).value = selected as string;
    }
  });

  document.getElementById("browse-output-btn")?.addEventListener("click", async () => {
    const filePath = await tauriSave({
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
      title: "Lưu file kết quả"
    });
    if (filePath) {
      (document.querySelector(selectors.outputFile) as HTMLInputElement).value = filePath;
    }
  });

  document.getElementById("browse-template-btn")?.addEventListener("click", async () => {
    const selected = await open({
      directory: false,
      multiple: false,
      filters: [{ name: "Excel", extensions: ["xlsx", "xls"] }],
      title: "Chọn file mẫu"
    });
    if (selected) {
      (document.querySelector(selectors.templateFile) as HTMLInputElement).value = selected as string;
      updateTemplateHeaders(selected as string);
    }
  });

  async function updateTemplateHeaders(path: string) {
    const skipRows = parseInt((document.querySelector(selectors.skipRows) as HTMLInputElement)?.value || "0");
    try {
      const headers = await invoke("get_template_headers", { path, skip_rows: skipRows }) as string[];
      if (headers && headers.length > 0) {
        const headerRow = document.querySelector(`${selectors.previewHeader} tr`);
        if (headerRow) {
          headerRow.innerHTML = headers.map(h => `<th>${h}</th>`).join("");
          if (headers.length < 5) {
            // Add padding if template has few columns to keep "Source" info if needed
            // though user said headers from template, so we follow that exactly.
          }
          addLog("Table headers updated from template.", "system");
        }
      }
    } catch (e) {
      addLog(`Failed to load template headers: ${e}`, "error");
    }
  }

  document.getElementById("export-btn")?.addEventListener("click", async () => {
    const outputFile = (document.querySelector(selectors.outputFile) as HTMLInputElement).value;
    if (outputFile) {
      addLog(`Attempting to open result: ${outputFile.split(/[\\/]/).pop()}`, "system");
      try {
        await invoke("open_result_folder", { path: outputFile });
      } catch (e) {
        addLog(`Failed to open result: ${e}`, "error");
      }
    } else {
      addLog("Export failed: No output file path specified.", "error");
    }
  });

  document.querySelector(selectors.runBtn)?.addEventListener("click", () => runConsolidation());

  document.querySelectorAll(selectors.tabs).forEach(tab => {
    tab.addEventListener('click', (e) => {
      const tabId = (e.currentTarget as HTMLElement).getAttribute('data-tab');
      if (tabId) switchTab(tabId);
    });
  });

  document.querySelector(selectors.sidebarClose)?.addEventListener('click', () => {
    document.querySelector(selectors.sidebar)?.classList.remove('active');
  });

  document.querySelector(selectors.saveMappingBtn)?.addEventListener('click', () => {
    if (!currentMappingPath) return;
    const stt = (document.querySelector(selectors.mapStt) as HTMLSelectElement).value;
    const name = (document.querySelector(selectors.mapName) as HTMLSelectElement).value;
    const unit = (document.querySelector(selectors.mapUnit) as HTMLSelectElement).value;
    const qty = (document.querySelector(selectors.mapQty) as HTMLSelectElement).value;

    mappingOverrides[currentMappingPath] = {
      stt: stt === "" ? null : parseInt(stt, 10),
      name: name === "" ? null : parseInt(name, 10),
      unit: unit === "" ? null : parseInt(unit, 10),
      qty: qty === "" ? null : parseInt(qty, 10)
    };

    document.querySelector(selectors.sidebar)?.classList.remove('active');
    addLog(`Mapping saved for ${currentMappingPath.split(/[\\/]/).pop()}`, "success");
  });

  document.querySelector(selectors.skipRows)?.addEventListener('input', () => {
    updateConfigSummary();
    const template = (document.querySelector(selectors.templateFile) as HTMLInputElement).value;
    if (template) updateTemplateHeaders(template);
  });

  // Window Control Listeners
  document.getElementById('titlebar-minimize')?.addEventListener('click', () => appWindow.minimize());
  document.getElementById('titlebar-maximize')?.addEventListener('click', () => appWindow.toggleMaximize());
  document.getElementById('titlebar-close')?.addEventListener('click', () => appWindow.close());

  // Handle window maximized state for CSS fixes
  const updateMaximizedState = async () => {
    const isMaximized = await appWindow.isMaximized();
    if (isMaximized) {
      document.body.classList.add('maximized');
    } else {
      document.body.classList.remove('maximized');
    }
  };

  appWindow.onResized(() => {
    updateMaximizedState();
  });

  // Initial check
  updateMaximizedState();

  initResizers();
  initTableResizers();
  initExcelActions();
});

function applyFiltersAndSort() {
  let data = [...allSummaryRecords];

  // Apply Search Filters
  Object.keys(tableFilters).forEach(key => {
    const term = tableFilters[key].toLowerCase();
    if (term) {
      data = data.filter(row => {
        const val = String(row[key] || '').toLowerCase();
        return val.includes(term);
      });
    }
  });

  // Apply Sort
  if (tableSort) {
    const { key, dir } = tableSort;
    data.sort((a, b) => {
      let valA = a[key];
      let valB = b[key];

      // Handle numeric sorting for 'khoi_luong' and 'stt'
      if (key === 'khoi_luong' || key === 'stt') {
        const numA = parseFloat(valA) || 0;
        const numB = parseFloat(valB) || 0;
        return dir === 'asc' ? numA - numB : numB - numA;
      }

      valA = String(valA || '').toLowerCase();
      valB = String(valB || '').toLowerCase();
      if (valA < valB) return dir === 'asc' ? -1 : 1;
      if (valA > valB) return dir === 'asc' ? 1 : -1;
      return 0;
    });
  }

  renderPreviewTable(data);
}

function renderPreviewTable(records: any[]) {
  const previewBody = document.querySelector(selectors.previewBody);
  if (!previewBody) return;

  previewBody.innerHTML = records.map(row => `
    <tr data-sources="${row.sources || ''}">
      <td>${row.stt || '-'}</td>
      <td>${row.ten_cong_viec}</td>
      <td>${row.don_vi}</td>
      <td>${(row.khoi_luong || 0).toLocaleString()}</td>
    </tr>
  `).join('');
}

function initExcelActions() {
  const menu = document.getElementById('column-filter-menu') as HTMLElement;
  const searchInput = document.getElementById('filter-search-input') as HTMLInputElement;
  let activeCol: string | null = null;

  // Toggle Menus
  document.querySelectorAll('.filter-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const col = (btn as HTMLElement).getAttribute('data-col');
      if (!col) return;

      if (activeCol === col && menu.classList.contains('active')) {
        menu.classList.remove('active');
        return;
      }

      activeCol = col;

      // Position menu
      const rect = (btn as HTMLElement).getBoundingClientRect();
      menu.style.top = `${rect.bottom + 5}px`;
      menu.style.left = `${Math.min(window.innerWidth - 210, rect.left)}px`;
      menu.classList.add('active');

      // Update UI state
      searchInput.value = tableFilters[activeCol] || '';
      searchInput.focus();

      // Highlight active buttons
      document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
    });
  });

  // Sort Actions
  document.getElementById('sort-asc')?.addEventListener('click', () => {
    if (!activeCol) return;
    tableSort = { key: activeCol, dir: 'asc' };
    applyFiltersAndSort();
    menu.classList.remove('active');
  });

  document.getElementById('sort-desc')?.addEventListener('click', () => {
    if (!activeCol) return;
    tableSort = { key: activeCol, dir: 'desc' };
    applyFiltersAndSort();
    menu.classList.remove('active');
  });

  // Search Action
  searchInput.addEventListener('input', () => {
    if (!activeCol) return;
    tableFilters[activeCol] = searchInput.value;
    applyFiltersAndSort();
  });

  // Close on click outside
  document.addEventListener('click', (e) => {
    if (!menu.contains(e.target as Node)) {
      menu.classList.remove('active');
      document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
    }
  });
}

function initTableResizers() {
  const table = document.getElementById('preview-table');
  if (!table) return;

  const resizers = table.querySelectorAll('.col-resizer');
  resizers.forEach((resizer) => {
    const th = resizer.parentElement as HTMLElement;
    if (!th) return;

    let startWidth = 0;
    let nextStartWidth = 0;
    let startX = 0;

    setupResizer(resizer as HTMLElement, (clientX) => {
      const nextTh = th.nextElementSibling as HTMLElement;
      if (!th || !nextTh) return;

      if (startX === 0) {
        startWidth = th.offsetWidth;
        nextStartWidth = nextTh.offsetWidth;
        startX = clientX;
      }
      const dx = clientX - startX;

      // Ensure neither column goes below minimum width
      const minWidth = 40;
      if (startWidth + dx > minWidth && nextStartWidth - dx > minWidth) {
        th.style.width = `${startWidth + dx}px`;
        nextTh.style.width = `${nextStartWidth - dx}px`;
      }
    });

    // Reset markers on mouseup (handled inside setupResizer but we need to reset our local startX)
    resizer.addEventListener('mousedown', () => {
      startX = 0;
    });
  });
}

function initResizers() {
  const leftResizer = document.getElementById('resizer-left');
  const rightResizer = document.getElementById('resizer-right');
  const leftPane = document.querySelector('.left-pane') as HTMLElement;
  const rightPane = document.querySelector('.right-pane') as HTMLElement;

  if (leftResizer && leftPane) {
    setupResizer(leftResizer, (x) => {
      const newWidth = Math.max(250, Math.min(600, x));
      leftPane.style.width = `${newWidth}px`;
    });
  }

  if (rightResizer && rightPane) {
    setupResizer(rightResizer, (x) => {
      const newWidth = Math.max(200, Math.min(500, window.innerWidth - x));
      rightPane.style.width = `${newWidth}px`;
    });
  }
}

function setupResizer(resizer: HTMLElement, onMove: (x: number) => void) {
  let isDragging = false;

  resizer.addEventListener('mousedown', (e) => {
    isDragging = true;
    resizer.classList.add('active');
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none'; // Disable text selection while dragging
    e.preventDefault();
  });

  document.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    onMove(e.clientX);
  });

  document.addEventListener('mouseup', () => {
    if (!isDragging) return;
    isDragging = false;
    resizer.classList.remove('active');
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  });
}
