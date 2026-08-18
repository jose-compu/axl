use crate::{PatternDetector, PatternIssue};

pub struct DialogPatternDetector;

impl PatternDetector for DialogPatternDetector {
    fn rule_id(&self) -> &'static str {
        "patterns/incomplete-dialog"
    }

    fn detect(&self, source: &str) -> Vec<PatternIssue> {
        let has_dialog = source.contains("<dialog")
            || source.contains("role=\"dialog\"")
            || source.contains("role='dialog'");
        if !has_dialog {
            return Vec::new();
        }
        let has_accessible_name =
            source.contains("aria-label=") || source.contains("aria-labelledby=");
        if has_accessible_name {
            return Vec::new();
        }
        vec![PatternIssue {
            rule_id: self.rule_id(),
            message: "Dialog is missing an accessible name (aria-label or aria-labelledby)."
                .to_string(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::DialogPatternDetector;
    use crate::PatternDetector;

    #[test]
    fn no_dialog_returns_no_issue() {
        let issues = DialogPatternDetector.detect("<div>hello</div>");
        assert!(issues.is_empty());
    }

    #[test]
    fn role_dialog_triggers_missing_name_issue() {
        let issues = DialogPatternDetector.detect("<div role=\"dialog\"></div>");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("accessible name"));
    }

    #[test]
    fn named_dialog_is_not_reported() {
        let issues = DialogPatternDetector.detect("<div role='dialog' aria-label=\"Settings\"></div>");
        assert!(issues.is_empty());
    }
}
