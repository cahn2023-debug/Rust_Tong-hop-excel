use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelRecord {
    pub stt: String,
    pub ten_cong_viec: String,
    pub don_vi: String,
    pub khoi_luong: f64,
    pub source_file: String,
    pub source_sheet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub last_modified: i64,
    pub file_hash: Option<String>,
    pub status: FileStatus,
    pub analysis: Option<FileAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColumnMapping {
    pub stt: Option<usize>,
    pub name: Option<usize>,
    pub unit: Option<usize>,
    pub qty: Option<usize>,
    pub detected_names: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatus {
    Pending,
    Processed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileAnalysis {
    pub confidence: f64,
    pub is_deviant: bool,
    pub has_zero_data: bool,
    pub has_valid_data: bool,
    pub reason: String,
    pub header_row: Option<usize>,
    pub column_offsets: std::collections::HashMap<String, i32>,
    pub detected_columns: std::collections::HashMap<String, String>,
}
