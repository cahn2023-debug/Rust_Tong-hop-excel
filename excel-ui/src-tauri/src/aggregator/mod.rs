use crate::domain::models::ExcelRecord;
use crate::domain::normalization::Normalizer;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Aggregator {
    // Key: (norm_name, norm_unit), Value: (stt, original_don_vi, original_ten, total_khoi_luong, source_quantities)
    results: Arc<DashMap<(String, String), (String, String, String, f64, HashMap<String, f64>)>>,
    normalizer: Normalizer,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            results: Arc::new(DashMap::new()),
            normalizer: Normalizer::new(true),
        }
    }

    pub fn add_records(&self, records: Vec<ExcelRecord>) {
        for record in records {
            let norm_name = self.normalizer.normalize(&record.ten_cong_viec);
            let norm_unit = self.normalizer.normalize(&record.don_vi);
            let source_file = record.source_file.clone();

            let mut entry = self.results.entry((norm_name, norm_unit)).or_insert((
                record.stt.clone(),
                record.don_vi.clone(),
                record.ten_cong_viec.clone(),
                0.0,
                HashMap::new(),
            ));

            let val = entry.value_mut();
            val.3 += record.khoi_luong;

            // Accumulate quantity per file
            let source_qty = val.4.entry(source_file).or_insert(0.0);
            *source_qty += record.khoi_luong;
        }
    }

    pub fn get_final_results(&self) -> Vec<(String, String, String, f64, String)> {
        self.results
            .iter()
            .map(|r| {
                let v = r.value();
                let sources: Vec<String> =
                    v.4.iter()
                        .map(|(path, qty)| format!("{} ({})", path, qty))
                        .collect();
                (
                    v.0.clone(),
                    v.2.clone(),
                    v.1.clone(),
                    v.3,
                    sources.join("; "),
                )
            })
            .collect()
    }
}
