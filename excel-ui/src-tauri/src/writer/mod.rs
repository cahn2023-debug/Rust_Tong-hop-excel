use anyhow::Result;
use rust_xlsxwriter::*;
use std::path::Path;

pub struct ExcelWriter;

impl ExcelWriter {
    pub fn write<P: AsRef<Path>>(
        path: P,
        _template_path: Option<String>,
        data: Vec<(String, String, String, f64, String)>,
    ) -> Result<()> {
        // If a template is provided, in a more advanced version we'd copy it.
        // For now, we'll create a fresh workbook as rust_xlsxwriter doesn't edit.
        // We can however simulate using a template by copying the file if it exists,
        // but xlsxwriter would just overwrite it anyway.

        // FUTURE: Use a crate like `calamine` to read template styles and `xlsxwriter` to replicate.

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
}
