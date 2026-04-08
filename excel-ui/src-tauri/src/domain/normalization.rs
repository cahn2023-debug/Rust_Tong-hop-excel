use regex::Regex;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

static STRIP_NUM_REGEX: OnceLock<Regex> = OnceLock::new();

pub struct Normalizer {
    pub strip_numbers: bool,
}

impl Normalizer {
    pub fn new(strip_numbers: bool) -> Self {
        Self { strip_numbers }
    }

    pub fn normalize(&self, s: &str) -> String {
        // 1. NFC Normalization (crucial for Vietnamese character variants)
        let nfc_s: String = s.nfc().collect();

        // 2. Lowercase and Trim
        let mut result = nfc_s.to_lowercase().trim().to_string();

        // 3. Strip leading numbers and list markers (1.1, a., I.)
        if self.strip_numbers {
            let re = STRIP_NUM_REGEX
                .get_or_init(|| Regex::new(r"^(\d+(\.\d+)*|[ivxlc]+\.|[a-z][\.\)])\s*").unwrap());
            result = re.replace(&result, "").trim().to_string();
        }

        // 4. Final cleanup of common artifacts
        result
            .replace('\n', " ")
            .replace('\u{00A0}', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let norm = Normalizer::new(true);

        // Case and Trim
        assert_eq!(norm.normalize("  CÁP ĐỒNG  "), "cáp đồng");

        // Number stripping
        assert_eq!(norm.normalize("1.1 Cáp đồng"), "cáp đồng");
        assert_eq!(norm.normalize("a. Cáp đồng"), "cáp đồng");
        assert_eq!(norm.normalize("IV. Cáp đồng"), "cáp đồng");

        // Vietnamese NFC (Bóng vs Bóng)
        // Note: This test assumes the input is a specific variant,
        // but nfc() ensures they both become the same.
        let v1 = "Bóng đèn"; // 'o' + 'combining acute'
        let v2 = "Bóng đèn"; // precomposed 'ó'
        assert_eq!(norm.normalize(v1), norm.normalize(v2));

        // Whitespace cleanup
        assert_eq!(norm.normalize("Cáp   \n   đồng"), "cáp đồng");
    }
}
