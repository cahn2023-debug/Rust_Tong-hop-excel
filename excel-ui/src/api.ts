import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { state } from "./state";

export async function callRunConsolidation(inputDir: string, templateFile: string, skipRows: number) {
    return await invoke("run_consolidation", {
        inputDir,
        outputFile: "latest_result.xlsx",
        templatePath: templateFile || null,
        overrides: state.mappingOverrides,
        skipRows
    });
}

export async function callGetTemplateHeaders(path: string, skipRows: number) {
    return await invoke("get_template_headers", { path, skip_rows: skipRows }) as string[];
}

export async function callGetFileData(path: string, skipRows: number) {
    return await invoke("get_file_data", { path, skipRows }) as any[];
}

export async function callOpenFile(path: string) {
    return await invoke("open_file", { path });
}

export async function callOpenFolder(path: string) {
    return await invoke("open_folder", { path });
}

export async function callOpenResultFolder(path: string) {
    return await invoke("open_result_folder", { path });
}

export async function callFinalizeExport(source: string, destination: string) {
    return await invoke("finalize_export", { source, destination });
}

export async function callClearCache() {
    return await invoke("clear_cache");
}

export function setupBackendListeners(callbacks: {
    onFileDiscovered: (file: any) => void,
    onFileParsed: (payload: any) => void,
    onError: (msg: string) => void
}) {
    listen("file_discovered", (event) => callbacks.onFileDiscovered(event.payload));
    listen("file_parsed", (event) => callbacks.onFileParsed(event.payload));
    listen<string>("process_error", (event) => callbacks.onError(event.payload));
}
