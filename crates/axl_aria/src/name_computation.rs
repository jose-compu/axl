pub fn normalize_accessible_name(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn is_empty_accessible_name(raw: &str) -> bool {
    normalize_accessible_name(raw).is_empty()
}

#[cfg(test)]
mod tests {
    use super::{is_empty_accessible_name, normalize_accessible_name};

    #[test]
    fn normalize_collapses_whitespace_runs() {
        let input = "  Save \n\t  as   draft  ";
        assert_eq!(normalize_accessible_name(input), "Save as draft");
    }

    #[test]
    fn normalize_empty_input_returns_empty() {
        assert_eq!(normalize_accessible_name(""), "");
        assert_eq!(normalize_accessible_name("   \n\t  "), "");
    }

    #[test]
    fn empty_name_detection_uses_normalization() {
        assert!(is_empty_accessible_name("   \n\t  "));
        assert!(!is_empty_accessible_name("  Submit  "));
    }
}
