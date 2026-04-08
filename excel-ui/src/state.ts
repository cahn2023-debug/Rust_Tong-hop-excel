export interface DiscoveredFile {
    path: string;
    size: number;
    cached: boolean;
}

export interface MappingOverride {
    stt: number | null;
    name: number | null;
    unit: number | null;
    qty: number | null;
}

export interface AppState {
    isScanning: boolean;
    fileCount: number;
    mappingOverrides: Record<string, MappingOverride>;
    currentMappingPath: string | null;
    totalRecords: number;
    allSummaryRecords: any[];
    selectedJobName: string | null;
    tableFilters: Record<string, string>;
    tableSort: { key: string, dir: 'asc' | 'desc' } | null;
}

export const state: AppState = {
    isScanning: false,
    fileCount: 0,
    mappingOverrides: {},
    currentMappingPath: null,
    totalRecords: 0,
    allSummaryRecords: [],
    selectedJobName: null,
    tableFilters: { stt: '', ten_cong_viec: '', don_vi: '', khoi_luong: '' },
    tableSort: { key: 'stt', dir: 'asc' }
};

export function normalizePath(p: string): string {
    if (!p) return "";
    return p.replace(/\\/g, '/').toLowerCase().replace(/\/$/, '');
}
