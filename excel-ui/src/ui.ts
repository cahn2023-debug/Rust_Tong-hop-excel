import { state } from "./state";

export const selectors = {
    runBtn: "#run-btn",
    inputDir: "#input-dir",
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
    previewHeader: "#preview-header",
    detailBody: "#detail-body",
    detailPlaceholder: "#detail-placeholder",
    detailSection: "#detail-section",
    currentDetailFile: "#current-detail-file"
};

export function switchTab(tabId: string) {
    const tabs = document.querySelectorAll(selectors.tabs);
    const panes = document.querySelectorAll(selectors.panes);

    tabs.forEach(t => t.classList.remove('active'));
    panes.forEach(p => (p as HTMLElement).classList.remove('active'));

    const activeTab = document.querySelector(`.tab[data-tab="${tabId}"]`);
    const activePane = document.getElementById(`tab-${tabId}`);

    if (activeTab) activeTab.classList.add('active');
    if (activePane) activePane.classList.add('active');
}

export function addLog(msg: string, type: "system" | "success" | "error" = "system") {
    const logContainer = document.querySelector(selectors.errorLog);
    if (!logContainer) return;
    const entry = document.createElement("div");
    entry.className = `log-entry ${type}`;
    entry.innerText = `> [${new Date().toLocaleTimeString()}] ${msg}`;
    logContainer.prepend(entry);
}

export function updateConfigSummary() {
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

export function renderPreviewTable(records: any[]) {
    const previewBody = document.querySelector(selectors.previewBody);
    if (!previewBody) return;

    previewBody.innerHTML = records.map(row => `
    <tr data-sources="${row.sources || ''}" data-name="${row.ten_cong_viec || ''}">
      <td>${row.stt || '-'}</td>
      <td>${row.ten_cong_viec}</td>
      <td>${row.don_vi}</td>
      <td>${(row.khoi_luong || 0).toLocaleString()}</td>
    </tr>
  `).join('');
}

export function renderDetailTable(records: any[]) {
    const detailBody = document.querySelector(selectors.detailBody);
    if (!detailBody) return;

    if (records.length === 0) {
        detailBody.innerHTML = '<tr><td colspan="4" style="text-align:center; padding: 20px; color: var(--text-dim);">No valid records found in this file.</td></tr>';
        return;
    }

    detailBody.innerHTML = records.map(row => {
        const isActive = state.selectedJobName && row.ten_cong_viec === state.selectedJobName;
        const hasContent = (row.khoi_luong || 0) > 0;
        const classes = [
            isActive ? 'active-row' : '',
            hasContent ? 'retail-highlight' : ''
        ].filter(Boolean).join(' ');

        return `
      <tr data-name="${row.ten_cong_viec || ''}" class="${classes}">
        <td>${row.stt || '-'}</td>
        <td>${row.ten_cong_viec}</td>
        <td>${row.don_vi}</td>
        <td>${(row.khoi_luong || 0).toLocaleString()}</td>
      </tr>
    `;
    }).join('');
}

export function setupResizer(resizer: HTMLElement, onMove: (x: number, y: number) => void, cursor: string = 'col-resize') {
    let isDragging = false;

    resizer.addEventListener('mousedown', (e) => {
        isDragging = true;
        resizer.classList.add('active');
        document.body.style.cursor = cursor;
        document.body.style.userSelect = 'none';
        e.preventDefault();
    });

    document.addEventListener('mousemove', (e) => {
        if (!isDragging) return;
        onMove(e.clientX, e.clientY);
    });

    document.addEventListener('mouseup', () => {
        if (!isDragging) return;
        isDragging = false;
        resizer.classList.remove('active');
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
    });
}
