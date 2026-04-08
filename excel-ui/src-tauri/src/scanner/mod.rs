use crate::pipeline::messages::PipelineMessage;
use anyhow::Result;
use crossbeam_channel::Sender;
use std::fs;
use std::path::Path;

pub struct Scanner {
    sender: Sender<PipelineMessage>,
}

impl Scanner {
    pub fn new(sender: Sender<PipelineMessage>) -> Self {
        Self { sender }
    }

    pub fn scan<P: AsRef<Path>>(&self, root: P) -> Result<()> {
        self.scan_recursive(root.as_ref())?;
        Ok(())
    }

    fn scan_recursive(&self, dir: &Path) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.scan_recursive(&path)?;
                } else if self.is_excel_file(&path) {
                    let metadata = entry.metadata()?;
                    let last_modified = metadata
                        .modified()?
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs();

                    self.sender
                        .send(PipelineMessage::FileDiscovered {
                            path: path.to_path_buf(),
                            last_modified,
                        })
                        .map_err(|e| anyhow::anyhow!("Send error: {}", e))?;
                }
            }
        }
        Ok(())
    }

    fn is_excel_file(&self, path: &Path) -> bool {
        path.extension()
            .map_or(false, |ext| ext == "xlsx" || ext == "xls")
    }
}
