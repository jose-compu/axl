pub fn normalize_accessible_name(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn is_empty_accessible_name(raw: &str) -> bool {
    normalize_accessible_name(raw).is_empty()
}
