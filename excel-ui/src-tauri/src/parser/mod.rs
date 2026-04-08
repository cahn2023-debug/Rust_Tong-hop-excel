use crate::domain::models::{ColumnMapping, ExcelRecord};
use anyhow::Result;
use calamine::{Data, Range, Reader, open_workbook_auto};
use std::path::Path;

pub struct ExcelParser;

#[derive(Debug, serde::Serialize)]
pub struct FileAnalysis {
    pub confidence: f64,
    pub is_deviant: bool,
    pub has_zero_data: bool,
    pub reason: String,
    pub header_row: Option<usize>,
    pub column_offsets: std::collections::HashMap<String, i32>,
}

impl ExcelParser {
    pub fn get_headers<P: AsRef<Path>>(path: P, skip_rows: usize) -> Result<Vec<String>> {
        let mut workbook = open_workbook_auto(path.as_ref())?;
        if let Some(sheet_name) = workbook.sheet_names().get(0) {
            if let Ok(range) = workbook.worksheet_range(sheet_name) {
                let effective_range = if skip_rows > 0 {
                    range.range(
                        (skip_rows as u32, 0),
                        (range.height() as u32 - 1, range.width() as u32 - 1),
                    )
                } else {
                    range.clone()
                };

                if let Some((header_idx, _, _)) = Self::find_header_and_cols(&effective_range) {
                    if let Some(row) = effective_range.rows().nth(header_idx) {
                        return Ok(row
                            .iter()
                            .map(|cell| Self::data_to_string(cell).unwrap_or_default())
                            .collect());
                    }
                }
            }
        }
        Ok(vec![])
    }

    pub fn parse<P: AsRef<Path>>(
        path: P,
        manual_mapping: Option<ColumnMapping>,
        skip_rows: usize,
    ) -> Result<(Vec<ExcelRecord>, FileAnalysis)> {
        let mut workbook = open_workbook_auto(path.as_ref())?;
        let mut results = Vec::new();
        let path_str = path.as_ref().to_string_lossy().to_string();

        let mut final_analysis = FileAnalysis {
            confidence: 0.0,
            is_deviant: false,
            has_zero_data: false,
            reason: "No sheets processed".to_string(),
            header_row: None,
            column_offsets: std::collections::HashMap::new(),
        };

        let sheet_names = workbook.sheet_names().to_vec();
        for sheet_name in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                // Apply Global Skip Rows
                let effective_range = if skip_rows > 0 {
                    range.range(
                        (skip_rows as u32, 0),
                        (range.height() as u32 - 1, range.width() as u32 - 1),
                    )
                } else {
                    range.clone()
                };

                if let Some((header_idx, mut col_map, score)) =
                    Self::find_header_and_cols(&effective_range)
                {
                    final_analysis.header_row = Some(header_idx);
                    final_analysis.confidence = (score as f64 / 6.0).min(1.0);

                    // Analysis: Check for shifts (Anomaly Detection)
                    if let (Some(q), Some(n)) = (col_map.qty, col_map.name) {
                        final_analysis
                            .column_offsets
                            .insert("name_from_qty".to_string(), (n as i32) - (q as i32));
                    }

                    // 2. Overlay manual mapping if provided
                    if let Some(ref manual) = manual_mapping {
                        if manual.stt.is_some() {
                            col_map.stt = manual.stt;
                        }
                        if manual.name.is_some() {
                            col_map.name = manual.name;
                        }
                        if manual.unit.is_some() {
                            col_map.unit = manual.unit;
                        }
                        if manual.qty.is_some() {
                            col_map.qty = manual.qty;
                        }
                        final_analysis.reason = "Using manual override".to_string();
                    }

                    let mut sheet_records = 0;
                    for row in effective_range.rows().skip(header_idx + 1) {
                        let stt = col_map
                            .stt
                            .and_then(|idx| row.get(idx))
                            .and_then(Self::data_to_string)
                            .unwrap_or_else(|| "".to_string());
                        let job_name = col_map
                            .name
                            .and_then(|idx| row.get(idx))
                            .and_then(Self::data_to_string)
                            .unwrap_or_default();
                        let unit = col_map
                            .unit
                            .and_then(|idx| row.get(idx))
                            .and_then(Self::data_to_string)
                            .unwrap_or_default();
                        let volume = col_map
                            .qty
                            .and_then(|idx| row.get(idx))
                            .and_then(Self::data_to_float)
                            .unwrap_or(0.0);

                        // Detection for Highlight (Null, 0, Blank)
                        if !job_name.is_empty() && volume == 0.0 {
                            final_analysis.has_zero_data = true;
                        }

                        if job_name.is_empty() || volume == 0.0 {
                            continue;
                        }

                        results.push(ExcelRecord {
                            stt,
                            ten_cong_viec: job_name,
                            don_vi: unit,
                            khoi_luong: volume,
                            source_file: path_str.clone(),
                            source_sheet: sheet_name.clone(),
                        });
                        sheet_records += 1;
                    }

                    if sheet_records > 0 {
                        final_analysis.is_deviant = final_analysis.confidence < 0.8;
                        final_analysis.reason = if final_analysis.is_deviant {
                            "Table found but structure is non-standard (low confidence)".to_string()
                        } else {
                            "Successfully parsed with standard structure".to_string()
                        };
                    } else {
                        final_analysis.reason =
                            "Header found but no data rows match filters".to_string();
                    }
                } else {
                    final_analysis.reason =
                        "Could not identify a table header in this sheet".to_string();
                }
            }
        }

        Ok((results, final_analysis))
    }

    fn find_header_and_cols(range: &Range<Data>) -> Option<(usize, ColumnMapping, i32)> {
        let kw_stt = [
            "stt",
            "số tt",
            "số thứ tự",
            "no.",
            "id",
            "item",
            "tt",
            "thứ tự",
            "mã số",
            "mã hiệu",
        ];
        let kw_qty = [
            "khối lượng",
            "khoi luong",
            "sl",
            "số lượng",
            "kl",
            "quyết toán",
            "qt",
            "khối lượng qt",
            "khối lượng thực hiện",
            "thanh toán",
            "kltt",
            "qty",
        ];
        let kw_name = [
            "hạng mục",
            "hang muc",
            "vật tư",
            "tên",
            "nội dung",
            "tên công việc",
            "diễn giải",
            "tên vật tư",
            "danh mục",
            "công tác",
            "vật liệu",
            "thiết bị",
        ];
        let kw_unit = ["đvt", "đơn vị", "don vi", "đơn vị tính", "vị tính"];

        let mut best_row = None;
        let mut max_score = 0;

        // Adaptive Scan: 100 rows x 100 columns
        for (i, row) in range.rows().enumerate().take(100) {
            let mut mapping = ColumnMapping::default();
            let mut current_score = 0;

            for (j, cell) in row.iter().enumerate().take(100) {
                let cell_str = Self::data_to_string(cell)
                    .unwrap_or_default()
                    .to_lowercase()
                    .trim()
                    .to_string();
                if cell_str.is_empty() {
                    continue;
                }

                if mapping.stt.is_none() && kw_stt.iter().any(|k| cell_str.contains(k)) {
                    mapping.stt = Some(j);
                    current_score += 1;
                } else if mapping.qty.is_none() && kw_qty.iter().any(|k| cell_str.contains(k)) {
                    mapping.qty = Some(j);
                    current_score += 3;
                } else if mapping.name.is_none() && kw_name.iter().any(|k| cell_str.contains(k)) {
                    mapping.name = Some(j);
                    current_score += 1;
                } else if mapping.unit.is_none() && kw_unit.iter().any(|k| cell_str.contains(k)) {
                    mapping.unit = Some(j);
                    current_score += 1;
                }
            }

            if mapping.qty.is_some() && current_score > max_score {
                let q_idx = mapping.qty.unwrap();
                if mapping.stt.is_none() && q_idx >= 3 {
                    mapping.stt = Some(q_idx - 3);
                }
                if mapping.name.is_none() && q_idx >= 2 {
                    mapping.name = Some(q_idx - 2);
                }
                if mapping.unit.is_none() && q_idx >= 1 {
                    mapping.unit = Some(q_idx - 1);
                }

                max_score = current_score;
                best_row = Some((i, mapping, current_score));
            }
        }
        best_row
    }

    fn data_to_string(data: &Data) -> Option<String> {
        match data {
            Data::String(s) => Some(s.clone()),
            Data::Float(f) => Some(f.to_string()),
            Data::Int(i) => Some(i.to_string()),
            _ => None,
        }
    }

    fn data_to_float(data: &Data) -> Option<f64> {
        match data {
            Data::Float(f) => Some(*f),
            Data::Int(i) => Some(*i as f64),
            Data::String(s) => s.replace(',', ".").parse::<f64>().ok(),
            _ => None,
        }
    }
}
