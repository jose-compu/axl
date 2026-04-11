pub mod dialog;

#[derive(Debug, Clone)]
pub struct PatternIssue {
    pub rule_id: &'static str,
    pub message: String,
}

pub trait PatternDetector {
    fn rule_id(&self) -> &'static str;
    fn detect(&self, source: &str) -> Vec<PatternIssue>;
}

pub fn run_dialog_detector(enabled: bool, source: &str) -> Vec<PatternIssue> {
    if !enabled {
        return Vec::new();
    }
    dialog::DialogPatternDetector.detect(source)
}

#[cfg(test)]
mod tests {
    use super::run_dialog_detector;

    #[test]
    fn disabled_detector_returns_empty() {
        let issues = run_dialog_detector(false, "<dialog></dialog>");
        assert!(issues.is_empty());
    }

    #[test]
    fn enabled_detector_returns_issue_for_dialog() {
        let issues = run_dialog_detector(true, "<dialog></dialog>");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "patterns/incomplete-dialog");
    }
}
