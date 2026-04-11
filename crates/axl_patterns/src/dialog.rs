use crate::{PatternDetector, PatternIssue};

pub struct DialogPatternDetector;

impl PatternDetector for DialogPatternDetector {
    fn rule_id(&self) -> &'static str {
        "patterns/incomplete-dialog"
    }

    fn detect(&self, source: &str) -> Vec<PatternIssue> {
        let has_dialog = source.contains("<dialog") || source.contains("role=\"dialog\"");
        if !has_dialog {
            return Vec::new();
        }
        vec![PatternIssue {
            rule_id: self.rule_id(),
            message: "Pattern analysis scaffold active: full dialog completeness checks are not implemented yet."
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
    fn role_dialog_triggers_scaffold_issue() {
        let issues = DialogPatternDetector.detect("<div role=\"dialog\"></div>");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("scaffold"));
    }
}
