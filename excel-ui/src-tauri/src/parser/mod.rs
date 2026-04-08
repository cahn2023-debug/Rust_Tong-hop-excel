use crate::domain::models::{ColumnMapping, ExcelRecord, FileAnalysis};
use anyhow::Result;
use calamine::{open_workbook_auto, Data, Range, Reader};
use std::path::Path;

pub struct ExcelParser;

// Removed FileAnalysis struct (moved to domain::models)

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
            has_valid_data: false,
            reason: "No sheets processed".to_string(),
            header_row: None,
            column_offsets: std::collections::HashMap::new(),
            detected_columns: std::collections::HashMap::new(),
        };

        let sheet_names = workbook.sheet_names().to_vec();
        for sheet_name in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                let (sheet_records, sheet_analysis) = Self::parse_sheet(
                    &range,
                    manual_mapping.clone(),
                    skip_rows,
                    &path_str,
                    &sheet_name,
                );

                results.extend(sheet_records);

                // Merge analysis (take the best one or aggregate)
                if sheet_analysis.confidence > final_analysis.confidence {
                    final_analysis = sheet_analysis;
                }
            }
        }

        if results.is_empty() && final_analysis.confidence < 0.1 {
            final_analysis.reason = "Could not identify any valid data in any sheet".to_string();
        }

        // If we have records, we definitely have valid data
        if !results.is_empty() {
            final_analysis.has_valid_data = true;
        }

        Ok((results, final_analysis))
    }

    pub fn get_template_mapping<P: AsRef<Path>>(
        path: P,
        skip_rows: usize,
    ) -> Result<Option<ColumnMapping>> {
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

                if let Some((_, mappings, _)) = Self::find_header_and_cols(&effective_range) {
                    return Ok(mappings.get(0).cloned());
                }
            }
        }
        Ok(None)
    }

    fn parse_sheet(
        range: &Range<Data>,
        manual_mapping: Option<ColumnMapping>,
        skip_rows: usize,
        path_str: &str,
        sheet_name: &str,
    ) -> (Vec<ExcelRecord>, FileAnalysis) {
        let mut results = Vec::new();
        let mut analysis = FileAnalysis {
            confidence: 0.0,
            is_deviant: false,
            has_zero_data: false,
            has_valid_data: false,
            reason: "Sheet processed".to_string(),
            header_row: None,
            column_offsets: std::collections::HashMap::new(),
            detected_columns: std::collections::HashMap::new(),
        };

        // Apply Global Skip Rows
        let effective_range = if skip_rows > 0 {
            range.range(
                (skip_rows as u32, 0),
                (range.height() as u32 - 1, range.width() as u32 - 1),
            )
        } else {
            range.clone()
        };

        // 1. Try to use manual mapping if it's "Complete"
        if let Some(ref m) = manual_mapping {
            if m.name.is_some() && m.qty.is_some() {
                // We have enough to try parsing.
                // We still need to find WHICH row is the header or just start from row 0.
                // Usually, manual mapping implies we know the structure.
                // We'll search for the first row that has a non-empty name and a numeric qty.
                for row in effective_range.rows() {
                    if let Some(record) = Self::extract_record(row, m, path_str, sheet_name) {
                        if record.khoi_luong.abs() < 1e-3 {
                            analysis.has_zero_data = true;
                        } else {
                            analysis.has_valid_data = true;
                        }
                        results.push(record);
                    }
                }
                if !results.is_empty() {
                    analysis.confidence = 1.0;
                    analysis.reason = "Parsed using provided mapping/template".to_string();
                    return (results, analysis);
                }
            }
        }

        // 2. Fallback to Auto-Discovery
        if let Some((header_idx, multi_col_maps, score)) =
            Self::find_header_and_cols(&effective_range)
        {
            analysis.header_row = Some(header_idx);
            analysis.confidence = (score as f64 / 6.0).min(1.0);

            // Set detected columns from the first mapping for logs
            if let Some(m) = multi_col_maps.first() {
                analysis.detected_columns = m.detected_names.clone();
            }

            for row in effective_range.rows().skip(header_idx + 1) {
                for col_map in &multi_col_maps {
                    let mapping = Self::apply_manual_overlay(col_map, &manual_mapping);
                    if let Some(record) = Self::extract_record(row, &mapping, path_str, sheet_name)
                    {
                        if record.khoi_luong.abs() < 1e-3 {
                            analysis.has_zero_data = true;
                        } else {
                            analysis.has_valid_data = true;
                        }
                        results.push(record);
                    }
                }
            }

            analysis.is_deviant = analysis.confidence < 0.8;
            analysis.reason = if results.is_empty() {
                "Header found but no data rows match filters".to_string()
            } else if analysis.is_deviant {
                "Table found but structure is non-standard (low confidence)".to_string()
            } else {
                "Successfully parsed with standard structure".to_string()
            };
        } else {
            // FALLBACK: Try scattered data detection
            let scattered = Self::parse_scattered(&effective_range);
            if !scattered.is_empty() {
                results.extend(
                    scattered
                        .into_iter()
                        .map(|(stt, name, unit, qty)| ExcelRecord {
                            stt,
                            ten_cong_viec: name,
                            don_vi: unit,
                            khoi_luong: qty,
                            source_file: path_str.to_string(),
                            source_sheet: sheet_name.to_string(),
                        }),
                );
                analysis.confidence = 0.5;
                analysis.has_valid_data = true;
                analysis.reason = "Parsed using scattered data discovery".to_string();
            }
        }

        (results, analysis)
    }

    fn apply_manual_overlay(base: &ColumnMapping, manual: &Option<ColumnMapping>) -> ColumnMapping {
        let mut effective = base.clone();
        if let Some(ref m) = manual {
            if m.stt.is_some() {
                effective.stt = m.stt;
            }
            if m.name.is_some() {
                effective.name = m.name;
            }
            if m.unit.is_some() {
                effective.unit = m.unit;
            }
            if m.qty.is_some() {
                effective.qty = m.qty;
            }
            effective.detected_names.extend(m.detected_names.clone());
        }
        effective
    }

    fn extract_record(
        row: &[Data],
        mapping: &ColumnMapping,
        path: &str,
        sheet: &str,
    ) -> Option<ExcelRecord> {
        let job_name = mapping
            .name
            .and_then(|idx| row.get(idx))
            .and_then(Self::data_to_string)?
            .trim()
            .to_string();

        if job_name.is_empty() {
            return None;
        }

        let stt = mapping
            .stt
            .and_then(|idx| row.get(idx))
            .and_then(Self::data_to_string)
            .unwrap_or_default();

        let unit = mapping
            .unit
            .and_then(|idx| row.get(idx))
            .and_then(Self::data_to_string)
            .unwrap_or_default();

        let volume = mapping
            .qty
            .and_then(|idx| row.get(idx))
            .and_then(Self::data_to_float)
            .unwrap_or(0.0);

        Some(ExcelRecord {
            stt,
            ten_cong_viec: job_name,
            don_vi: unit,
            khoi_luong: volume,
            source_file: path.to_string(),
            source_sheet: sheet.to_string(),
        })
    }

    pub fn find_header_and_cols(range: &Range<Data>) -> Option<(usize, Vec<ColumnMapping>, i32)> {
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

        // Adaptive Scan: 100 rows
        for (i, row) in range.rows().enumerate().take(100) {
            let mut mappings = Vec::new();
            let mut qty_cols = Vec::new();

            // First, find all quantity columns as horizontal "Anchors"
            for (j, cell) in row.iter().enumerate() {
                let cell_str = Self::data_to_string(cell)
                    .unwrap_or_default()
                    .to_lowercase()
                    .trim()
                    .to_string();
                if kw_qty.iter().any(|k| cell_str.contains(k)) {
                    qty_cols.push(j);
                }
            }

            if qty_cols.is_empty() {
                continue;
            }

            let mut current_row_score = 0;
            for qty_idx in qty_cols {
                let mut mapping = ColumnMapping {
                    qty: Some(qty_idx),
                    detected_names: std::collections::HashMap::new(),
                    ..Default::default()
                };
                if let Some(cell) = row.get(qty_idx) {
                    mapping
                        .detected_names
                        .insert("qty".into(), Self::data_to_string(cell).unwrap_or_default());
                }
                let mut score = 3;

                // Look for related fields within a 7-column window to the left of the anchor
                let search_start = if qty_idx >= 7 { qty_idx - 7 } else { 0 };
                for (j, cell) in row
                    .iter()
                    .enumerate()
                    .skip(search_start)
                    .take(qty_idx - search_start)
                {
                    let cell_str_raw = Self::data_to_string(cell).unwrap_or_default();
                    let cell_str = cell_str_raw.to_lowercase().trim().to_string();
                    if cell_str.is_empty() {
                        continue;
                    }

                    if mapping.stt.is_none() && kw_stt.iter().any(|k| cell_str.contains(k)) {
                        mapping.stt = Some(j);
                        mapping
                            .detected_names
                            .insert("stt".into(), cell_str_raw.clone());
                        score += 1;
                    } else if mapping.name.is_none() && kw_name.iter().any(|k| cell_str.contains(k))
                    {
                        mapping.name = Some(j);
                        mapping
                            .detected_names
                            .insert("name".into(), cell_str_raw.clone());
                        score += 2;
                    } else if mapping.unit.is_none() && kw_unit.iter().any(|k| cell_str.contains(k))
                    {
                        mapping.unit = Some(j);
                        mapping
                            .detected_names
                            .insert("unit".into(), cell_str_raw.clone());
                        score += 1;
                    }
                }

                // Heuristic Fallback for Name if not found explicitly but Qty exists
                if mapping.name.is_none() && qty_idx >= 2 {
                    mapping.name = Some(qty_idx - 2);
                    mapping
                        .detected_names
                        .insert("name".into(), format!("Neighbor(-2)"));
                }

                // REQUIREMENT: Must have at least a Name and a Quantity
                if mapping.name.is_some() && mapping.qty.is_some() {
                    current_row_score += score;
                    mappings.push(mapping);
                }
            }

            if current_row_score > max_score && !mappings.is_empty() {
                max_score = current_row_score;
                best_row = Some((i, mappings, current_row_score));
            }
        }
        best_row
    }

    fn parse_scattered(range: &Range<Data>) -> Vec<(String, String, String, f64)> {
        let mut results = Vec::new();
        let kw_name = [
            "hạng mục",
            "vật tư",
            "tên",
            "nội dung",
            "diễn giải",
            "tên vật tư",
        ];

        // Scan for labels and values relative to them
        for (i, row) in range.rows().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let cell_str = Self::data_to_string(cell)
                    .unwrap_or_default()
                    .to_lowercase()
                    .trim()
                    .to_string();

                // If we find a "Name/Content" label, look for a value nearby
                if kw_name.iter().any(|k| cell_str.contains(k)) {
                    // Try cell to the right or below for the actual name
                    let name = row
                        .get(j + 1)
                        .and_then(Self::data_to_string)
                        .or_else(|| {
                            range
                                .get_value((i as u32 + 1, j as u32))
                                .and_then(Self::data_to_string)
                        })
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    if name.is_empty() {
                        continue;
                    }

                    // Now look for Qty nearby
                    let mut qty = 0.0;

                    // Search in the same row first
                    for offset in 1..15 {
                        if let Some(c) = row.get(j + offset) {
                            if let Some(v) = Self::data_to_float(c) {
                                if v > 0.0 {
                                    qty = v;
                                    break;
                                }
                            }
                        }
                    }

                    if qty > 0.0 {
                        results.push(("".to_string(), name, "".to_string(), qty));
                    }
                }
            }
        }
        results
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
            Data::String(s) => {
                // Handle Vietnamese number format: 1.234.567,89 -> 1234567.89
                // or US/Generic format: 1,234,567.89 -> 1234567.89
                let s = s.trim();
                if s.is_empty() {
                    return None;
                }

                // If contains both , and ., assume . is thousand and , is decimal (Vietnamese standard)
                if s.contains(',') && s.contains('.') {
                    let cleaned = s.replace('.', "").replace(',', ".");
                    cleaned.parse::<f64>().ok()
                } else if s.contains(',') {
                    // Only comma: maybe decimal (Vietnamese) or thousand (US)?
                    // In VN, 1,5 means 1.5. In US, 1,000 means 1000.
                    // We check if there are 3 digits after the comma to guess thousand separator.
                    let parts: Vec<&str> = s.split(',').collect();
                    if parts.len() == 2 && parts[1].len() == 3 {
                        // Likely thousand separator
                        s.replace(',', "").parse::<f64>().ok()
                    } else {
                        // Likely decimal separator
                        s.replace(',', ".").parse::<f64>().ok()
                    }
                } else {
                    s.parse::<f64>().ok()
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::Data;

    #[test]
    fn test_find_multi_columns() {
        let mut range = Range::new((0, 0), (1, 7));
        // STT | Name | Unit | Qty | STT | Name | Unit | Qty
        range.set_value((0, 0), Data::String("STT".into()));
        range.set_value((0, 1), Data::String("Tên".into()));
        range.set_value((0, 2), Data::String("ĐVT".into()));
        range.set_value((0, 3), Data::String("Khối lượng".into()));
        range.set_value((0, 4), Data::String("STT".into()));
        range.set_value((0, 5), Data::String("Tên".into()));
        range.set_value((0, 6), Data::String("ĐVT".into()));
        range.set_value((0, 7), Data::String("Khối lượng".into()));

        let result = ExcelParser::find_header_and_cols(&range);
        assert!(result.is_some());
        let (_, mappings, _) = result.unwrap();
        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].qty, Some(3));
        assert_eq!(mappings[1].qty, Some(7));
    }
}
