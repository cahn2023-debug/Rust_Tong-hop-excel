use anyhow::Result;
use rust_xlsxwriter::*;
use std::path::Path;

pub struct ExcelWriter;

impl ExcelWriter {
    pub fn write<P: AsRef<Path>>(
        path: P,
        template_path: Option<String>,
        data: Vec<(String, String, String, f64, String)>,
    ) -> Result<()> {
        if let Some(tmpl) = template_path {
            if Path::new(&tmpl).exists() {
                return Self::write_with_template(&tmpl, path.as_ref(), data);
            }
        }

        Self::write_plain(path, data)
    }

    fn write_plain<P: AsRef<Path>>(
        path: P,
        data: Vec<(String, String, String, f64, String)>,
    ) -> Result<()> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Write Headers (Simplified/Industrial)
        let header_format = Format::new().set_bold().set_background_color(Color::Silver);

        worksheet.write_string_with_format(0, 0, "STT", &header_format)?;
        worksheet.write_string_with_format(0, 1, "Tên công việc", &header_format)?;
        worksheet.write_string_with_format(0, 2, "Đơn vị", &header_format)?;
        worksheet.write_string_with_format(0, 3, "Khối lượng", &header_format)?;
        worksheet.write_string_with_format(0, 4, "Nguồn dữ liệu (File/Sheet)", &header_format)?;

        // Write Data
        for (i, (stt, name, unit, volume, sources)) in data.into_iter().enumerate() {
            let row = (i + 1) as u32;
            worksheet.write_string(row, 0, &stt)?;
            worksheet.write_string(row, 1, &name)?;
            worksheet.write_string(row, 2, &unit)?;
            worksheet.write_number(row, 3, volume)?;
            worksheet.write_string(row, 4, &sources)?;
        }

        // Auto-fit columns
        worksheet.autofit();

        workbook.save(path.as_ref())?;
        Ok(())
    }

    fn write_with_template<P1: AsRef<Path>, P2: AsRef<Path>>(
        template_path: P1,
        output_path: P2,
        data: Vec<(String, String, String, f64, String)>,
    ) -> Result<()> {
        let mut workbook = umya_spreadsheet::reader::xlsx::read(template_path.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to read template: {}", e))?;

        // Local helper for column index to name
        fn col_to_name(mut n: u32) -> String {
            let mut s = String::new();
            while n > 0 {
                let r = (n - 1) % 26;
                s.push((b'A' + r as u8) as char);
                n = (n - 1) / 26;
            }
            s.chars().rev().collect()
        }

        let sheet = workbook
            .get_sheet_mut(&0)
            .ok_or_else(|| anyhow::anyhow!("Failed to get first sheet"))?;

        // Try to find the header row and map columns
        let mut start_row = 1u32;
        let mut col_map = std::collections::HashMap::new();
        let mut found_header = false;

        let highest_row = sheet.get_highest_row();
        let search_depth = std::cmp::min(highest_row, 30); // Search first 30 rows

        for row in 1u32..=search_depth {
            let mut row_col_map = std::collections::HashMap::new();
            for col in 1u32..=15u32 {
                // Search wider range
                let col_name = col_to_name(col);
                let address = format!("{}{}", col_name, row);
                let cell_value = sheet
                    .get_cell(address)
                    .map(|c| c.get_value().to_lowercase())
                    .unwrap_or_default();

                if cell_value.contains("stt") {
                    row_col_map.insert("stt", col_name);
                } else if cell_value.contains("tên") {
                    row_col_map.insert("name", col_name);
                } else if cell_value.contains("đơn vị") || cell_value.contains("đvt") {
                    row_col_map.insert("unit", col_name);
                } else if cell_value.contains("số lượng") || cell_value.contains("khối lượng")
                {
                    row_col_map.insert("qty", col_name);
                } else if cell_value.contains("nguồn") || cell_value.contains("ghi chú") {
                    row_col_map.insert("note", col_name);
                }
            }
            if row_col_map.len() >= 3 {
                start_row = row + 1;
                col_map = row_col_map;
                found_header = true;
                break;
            }
        }

        // Default mapping if not found
        if !found_header {
            start_row = highest_row + 1;
            col_map.insert("stt", "A".to_string());
            col_map.insert("name", "B".to_string());
            col_map.insert("unit", "C".to_string());
            col_map.insert("qty", "D".to_string());
            col_map.insert("note", "E".to_string());
        }

        for (i, (stt, name, unit, volume, sources)) in data.into_iter().enumerate() {
            let row = start_row + i as u32;

            if let Some(c) = col_map.get("stt") {
                sheet.get_cell_mut(format!("{}{}", c, row)).set_value(stt);
            }
            if let Some(c) = col_map.get("name") {
                sheet.get_cell_mut(format!("{}{}", c, row)).set_value(name);
            }
            if let Some(c) = col_map.get("unit") {
                sheet.get_cell_mut(format!("{}{}", c, row)).set_value(unit);
            }
            if let Some(c) = col_map.get("qty") {
                sheet
                    .get_cell_mut(format!("{}{}", c, row))
                    .set_value(volume.to_string());
            }
            if let Some(c) = col_map.get("note") {
                sheet
                    .get_cell_mut(format!("{}{}", c, row))
                    .set_value(sources);
            }
        }

        umya_spreadsheet::writer::xlsx::write(&workbook, output_path.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to save export: {}", e))?;

        Ok(())
    }
}
