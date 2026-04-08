use crate::domain::models::ExcelRecord;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum PipelineMessage {
    // Scanner -> Filter
    FileDiscovered { path: PathBuf, last_modified: u64 },
}
