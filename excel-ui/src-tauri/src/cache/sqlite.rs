use crate::domain::models::{FileMetadata, FileStatus};
use rusqlite::{Connection, Result, params};
use std::path::Path;

pub struct CacheManager {
    conn: Connection,
}

impl CacheManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                last_modified INTEGER NOT NULL,
                file_hash TEXT,
                last_processed_at INTEGER,
                status TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS records_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                ten_cong_viec TEXT,
                don_vi TEXT,
                khoi_luong REAL,
                FOREIGN KEY(file_path) REFERENCES files(path)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_records_path ON records_cache(file_path)",
            [],
        )?;
        Ok(())
    }

    pub fn get_file_metadata(&self, path: &str) -> Result<Option<FileMetadata>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, last_modified, file_hash, status FROM files WHERE path = ?")?;
        let mut rows = stmt.query(params![path])?;

        if let Some(row) = rows.next()? {
            Ok(Some(FileMetadata {
                path: row.get(0)?,
                last_modified: row.get(1)?,
                file_hash: row.get(2)?,
                status: match row.get::<_, String>(3)?.as_str() {
                    "Processed" => FileStatus::Processed,
                    "Failed" => FileStatus::Failed,
                    "Skipped" => FileStatus::Skipped,
                    _ => FileStatus::Pending,
                },
            }))
        } else {
            Ok(None)
        }
    }
}
