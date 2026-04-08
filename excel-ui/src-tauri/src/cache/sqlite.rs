use crate::domain::models::{ExcelRecord, FileAnalysis, FileMetadata, FileStatus};
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct CacheManager {
    conn: Connection,
    path: String,
}

impl CacheManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(Self {
            conn,
            path: path_str,
        })
    }

    pub fn clone_conn(&self) -> Result<Self> {
        let conn = Connection::open(&self.path)?;
        Ok(Self {
            conn,
            path: self.path.clone(),
        })
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                last_modified INTEGER NOT NULL,
                file_hash TEXT,
                last_processed_at INTEGER,
                status TEXT NOT NULL,
                analysis TEXT
            )",
            [],
        )?;

        // Migration: Ensure 'analysis' column exists if the table was created by an older version
        let mut stmt = conn.prepare("PRAGMA table_info(files)")?;
        let exists = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                Ok(name)
            })?
            .any(|r| r.map(|n| n == "analysis").unwrap_or(false));

        if !exists {
            conn.execute("ALTER TABLE files ADD COLUMN analysis TEXT", [])?;
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS records_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                stt TEXT,
                ten_cong_viec TEXT,
                don_vi TEXT,
                khoi_luong REAL,
                source_sheet TEXT,
                FOREIGN KEY(file_path) REFERENCES files(path)
            )",
            [],
        )?;

        // Migration: Ensure 'stt' and 'source_sheet' exist in 'records_cache'
        let mut stmt = conn.prepare("PRAGMA table_info(records_cache)")?;
        let columns: Vec<String> = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                Ok(name)
            })?
            .collect::<Result<Vec<_>>>()?;

        if !columns.contains(&"stt".to_string()) {
            conn.execute("ALTER TABLE records_cache ADD COLUMN stt TEXT", [])?;
        }
        if !columns.contains(&"source_sheet".to_string()) {
            conn.execute("ALTER TABLE records_cache ADD COLUMN source_sheet TEXT", [])?;
        }

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_records_path ON records_cache(file_path)",
            [],
        )?;
        Ok(())
    }

    pub fn get_file_metadata(&self, path: &str) -> Result<Option<FileMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT path, last_modified, file_hash, status, analysis FROM files WHERE path = ?",
        )?;
        let mut rows = stmt.query(params![path])?;

        if let Some(row) = rows.next()? {
            let analysis_json: Option<String> = row.get(4)?;
            let analysis = analysis_json.and_then(|s| serde_json::from_str(&s).ok());

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
                analysis,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn save_file_records(
        &mut self,
        path: &str,
        records: &[ExcelRecord],
        analysis: &FileAnalysis,
        last_modified: i64,
    ) -> Result<()> {
        let analysis_json = serde_json::to_string(analysis).unwrap_or_default();

        // Use a transaction for atomic and FAST batch inserts
        let tx = self.conn.transaction()?;

        // 1. Update/Insert File Metadata
        tx.execute(
            "INSERT OR REPLACE INTO files (path, last_modified, status, analysis) VALUES (?, ?, ?, ?)",
            params![path, last_modified, "Processed", analysis_json],
        )?;

        // 2. Clear old records
        tx.execute(
            "DELETE FROM records_cache WHERE file_path = ?",
            params![path],
        )?;

        // 3. Insert new records
        {
            let mut stmt = tx.prepare(
                "INSERT INTO records_cache (file_path, stt, ten_cong_viec, don_vi, khoi_luong, source_sheet) VALUES (?, ?, ?, ?, ?, ?)"
            )?;
            for record in records {
                stmt.execute(params![
                    path,
                    record.stt,
                    record.ten_cong_viec,
                    record.don_vi,
                    record.khoi_luong,
                    record.source_sheet
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_file_records(&self, path: &str) -> Result<Vec<ExcelRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT stt, ten_cong_viec, don_vi, khoi_luong, source_sheet FROM records_cache WHERE file_path = ?",
        )?;
        let rows = stmt.query_map(params![path], |row| {
            Ok(ExcelRecord {
                stt: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                ten_cong_viec: row.get(1)?,
                don_vi: row.get(2)?,
                khoi_luong: row.get(3)?,
                source_file: path.to_string(),
                source_sheet: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn clear_all(&self) -> Result<()> {
        self.conn.execute("DELETE FROM records_cache", [])?;
        self.conn.execute("DELETE FROM files", [])?;
        Ok(())
    }
}
