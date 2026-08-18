use axl_aria::name_computation::is_empty_accessible_name;
use axl_core::{Category, Diagnostic, LintFile, Rule, Severity, line_col_from_offset};

pub struct EmptyAccessibleNameRule;

impl Rule for EmptyAccessibleNameRule {
    fn id(&self) -> &'static str {
        "naming/empty-accessible-name"
    }

    fn category(&self) -> Category {
        Category::Naming
    }

    fn run(&self, file: &LintFile) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for name in ["aria-label", "title"] {
            for (offset, value) in quoted_attr_values(&file.source, name) {
                if is_empty_accessible_name(value) {
                    let (line, column) = line_col_from_offset(&file.source, offset);
                    out.push(Diagnostic {
                        file: file.path.clone(),
                        line,
                        column,
                        severity: Severity::Error,
                        rule_id: self.id(),
                        message: "Accessible name resolves to an empty string.".to_string(),
                    });
                }
            }
        }
        out
    }
}

fn quoted_attr_values<'a>(source: &'a str, name: &str) -> Vec<(usize, &'a str)> {
    let mut out = Vec::new();
    for quote in ['"', '\''] {
        let needle = format!("{name}={quote}");
        for (offset, _) in source.match_indices(&needle) {
            let start = offset + needle.len();
            if let Some(end_rel) = source[start..].find(quote) {
                out.push((offset, &source[start..start + end_rel]));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::EmptyAccessibleNameRule;
    use axl_core::{LintFile, Rule};

    #[test]
    fn reports_empty_aria_label_and_title() {
        let rule = EmptyAccessibleNameRule;
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: "<button aria-label=\"  \" title=\"\n\t\" />".to_string(),
        };

        let diagnostics = rule.run(&file);
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().all(|d| d.rule_id == "naming/empty-accessible-name"));
    }

    #[test]
    fn reports_empty_single_quoted_names() {
        let rule = EmptyAccessibleNameRule;
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: "<button aria-label='   ' title='' />".to_string(),
        };

        let diagnostics = rule.run(&file);
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn ignores_non_empty_labels() {
        let rule = EmptyAccessibleNameRule;
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: "<button aria-label=\"Save\" title=\"Primary action\" />".to_string(),
        };

        let diagnostics = rule.run(&file);
        assert!(diagnostics.is_empty());
    }
}
