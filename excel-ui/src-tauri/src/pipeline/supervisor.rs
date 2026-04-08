use crate::aggregator::Aggregator;
use crate::cache::sqlite::CacheManager;
use crate::parser::ExcelParser;
use crate::pipeline::messages::PipelineMessage;
use crate::scanner::Scanner;
use crate::writer::ExcelWriter;
use anyhow::Result;
use calamine::{Data, Reader};
use crossbeam_channel::unbounded;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use tauri::Emitter;

pub struct Supervisor {
    cache: CacheManager,
}

impl Supervisor {
    pub fn new(db_path: &str) -> Result<Self> {
        let cache = CacheManager::new(db_path)?;
        Ok(Self { cache })
    }

    pub fn run(
        &self,
        window: &tauri::Window,
        input_dir: &str,
        output_file: &str,
        template_path: Option<String>,
        skip_rows: usize,
        overrides: std::collections::HashMap<String, crate::domain::models::ColumnMapping>,
    ) -> Result<()> {
        let aggregator = Arc::new(Aggregator::new());
        let overrides = Arc::new(overrides);

        // 1. Scan and gather all files to determine total volume
        let (scan_tx, scan_rx) = unbounded::<PipelineMessage>();
        let scanner = Scanner::new(scan_tx);
        scanner.scan(input_dir)?;

        let mut files_to_process = Vec::new();
        while let Ok(msg) = scan_rx.try_recv() {
            let PipelineMessage::FileDiscovered {
                path,
                last_modified,
            } = msg;
            files_to_process.push((path, last_modified));
        }

        let total_files = files_to_process.len();
        let processed_count = Arc::new(AtomicUsize::new(0));

        if total_files == 0 {
            return Ok(());
        }

        // 1.5. Extract template mapping if available
        let template_mapping = if let Some(ref tp) = template_path {
            match ExcelParser::get_template_mapping(tp, skip_rows) {
                Ok(m) => m,
                Err(e) => {
                    let _ = window.emit("process_error", format!("Template error: {}", e));
                    None
                }
            }
        } else {
            None
        };
        let template_mapping = Arc::new(template_mapping);

        // 2. Process gathered files
        for (path, last_modified) in files_to_process {
            let path_str = path.to_string_lossy().to_string();
            let metadata = std::fs::metadata(&path)?;
            let size = metadata.len();
            let size_mb = size as f64 / (1024.0 * 1024.0);

            // Check Cache
            let cached_meta = self.cache.get_file_metadata(&path_str)?;
            let needs_parsing = match &cached_meta {
                Some(meta) if meta.last_modified == last_modified as i64 => {
                    // Force re-parse if:
                    // 1. Manual override provided
                    // 2. Template provided (to ensure alignment)
                    // 3. Previous result had low confidence or NO valid data
                    overrides.contains_key(&path_str)
                        || template_path.is_some()
                        || meta
                            .analysis
                            .as_ref()
                            .map(|a| a.confidence < 0.1 || !a.has_valid_data)
                            .unwrap_or(true)
                }
                _ => true,
            };

            let _ = window.emit(
                "file_discovered",
                serde_json::json!({
                    "path": path_str,
                    "size": size,
                    "cached": !needs_parsing
                }),
            );

            if needs_parsing {
                let agg_clone = aggregator.clone();
                let window_clone = window.clone();
                let overrides_clone = overrides.clone();
                let processed_clone = processed_count.clone();
                let path_clone = path.clone();
                let cache_clone = self.cache.clone_conn()?;
                let t_map = template_mapping.clone();

                rayon::spawn(move || {
                    let mut cache_clone = cache_clone;
                    let start = std::time::Instant::now();

                    // Priority: 1. Manual Override > 2. Template Mapping > 3. Auto-Discovery (inside parse)
                    let manual = overrides_clone
                        .get(&path_clone.to_string_lossy().to_string())
                        .cloned()
                        .or_else(|| (*t_map).clone());

                    if let Ok((records, analysis)) =
                        ExcelParser::parse(path_clone.clone(), manual, skip_rows)
                    {
                        let count = records.len();

                        // Save to Cache
                        let _ = cache_clone.save_file_records(
                            &path_clone.to_string_lossy(),
                            &records,
                            &analysis,
                            last_modified as i64,
                        );

                        agg_clone.add_records(records);

                        let summary: Vec<serde_json::Value> = agg_clone
                            .get_final_results()
                            .into_iter()
                            .take(50)
                            .map(|(stt, name, unit, qty, sources)| {
                                serde_json::json!({
                                    "stt": stt,
                                    "ten_cong_viec": name,
                                    "don_vi": unit,
                                    "khoi_luong": qty,
                                    "sources": sources
                                })
                            })
                            .collect();

                        let elapsed = start.elapsed().as_secs_f64().max(0.001);
                        let throughput = size_mb / elapsed;
                        let current = processed_clone.fetch_add(1, Ordering::SeqCst) + 1;
                        let progress: f64 = (current as f64 / total_files as f64) * 100.0;
                        let efficiency = analysis.confidence * 100.0;

                        let _ = window_clone.emit(
                            "file_parsed",
                            serde_json::json!({
                                "path": path_clone.to_string_lossy(),
                                "records": count,
                                "progress": progress,
                                "throughput": throughput,
                                "efficiency": efficiency,
                                "summary": summary,
                                "analysis": analysis,
                                "message": if count == 0 { format!("{}", analysis.reason) } else { "".to_string() }
                            }),
                        );
                    } else {
                        processed_clone.fetch_add(1, Ordering::SeqCst);
                        let _ = window_clone.emit(
                            "process_error",
                            format!("Failed to parse: {}", path_clone.display()),
                        );
                        // Ensure we always emit something for the UI to know we're done with this file
                        let _ = window_clone.emit(
                            "file_parsed",
                            serde_json::json!({
                                "path": path_clone.to_string_lossy(),
                                "records": 0,
                                "progress": (processed_clone.load(Ordering::SeqCst) as f64 / total_files as f64) * 100.0,
                                "summary": [],
                                "analysis": crate::domain::models::FileAnalysis::default(),
                                "message": "Parsing error"
                            }),
                        );
                    }
                });
            } else if let Some(meta) = cached_meta {
                // EXTREME PERFORMANCE: Load from Cache
                let records_result = self.cache.get_file_records(&path_str);
                let current = processed_count.fetch_add(1, Ordering::SeqCst) + 1;
                let progress: f64 = (current as f64 / total_files as f64) * 100.0;

                match records_result {
                    Ok(records) => {
                        let count = records.len();
                        aggregator.add_records(records);

                        let summary: Vec<serde_json::Value> = aggregator
                            .get_final_results()
                            .into_iter()
                            .take(50)
                            .map(|(stt, name, unit, qty, sources)| {
                                serde_json::json!({
                                    "stt": stt,
                                    "ten_cong_viec": name,
                                    "don_vi": unit,
                                    "khoi_luong": qty,
                                    "sources": sources
                                })
                            })
                            .collect();

                        let _ = window.emit(
                            "file_parsed",
                            serde_json::json!({
                                "path": path_str,
                                "records": count,
                                "progress": progress,
                                "summary": summary,
                                "analysis": meta.analysis,
                                "cached": true,
                                "message": "Loaded from cache"
                            }),
                        );
                    }
                    Err(e) => {
                        let _ = window.emit(
                            "file_parsed",
                            serde_json::json!({
                                "path": path_str,
                                "records": 0,
                                "progress": progress,
                                "summary": [],
                                "analysis": meta.analysis,
                                "cached": true,
                                "message": format!("Cache error: {}", e)
                            }),
                        );
                    }
                }
            } else {
                // Should not happen, but for safety: increment progress if skipped
                processed_count.fetch_add(1, Ordering::SeqCst);
            }
        }

        // Wait for all parallel tasks to finish
        while processed_count.load(Ordering::SeqCst) < total_files {
            thread::sleep(std::time::Duration::from_millis(50));
        }

        let final_results = aggregator.get_final_results();
        ExcelWriter::write(output_file, template_path, final_results)?;

        Ok(())
    }
}
