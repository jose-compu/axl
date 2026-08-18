use axl_aria::roles::is_abstract_role;
use axl_core::{Category, Diagnostic, LintFile, Rule, Severity, line_col_from_offset};

pub struct AbstractRoleUsageRule;

impl Rule for AbstractRoleUsageRule {
    fn id(&self) -> &'static str {
        "roles/abstract-role-usage"
    }

    fn category(&self) -> Category {
        Category::Roles
    }

    fn run(&self, file: &LintFile) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for (offset, value) in quoted_attr_values(&file.source, "role") {
            if is_abstract_role(value) {
                let (line, column) = line_col_from_offset(&file.source, offset);
                out.push(Diagnostic {
                    file: file.path.clone(),
                    line,
                    column,
                    severity: Severity::Error,
                    rule_id: self.id(),
                    message: format!("Abstract ARIA role `{value}` is not valid on elements."),
                });
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
    use super::AbstractRoleUsageRule;
    use axl_core::{LintFile, Rule};

    #[test]
    fn reports_abstract_roles() {
        let rule = AbstractRoleUsageRule;
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: "<div role=\"widget\" />".to_string(),
        };

        let diagnostics = rule.run(&file);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "roles/abstract-role-usage");
    }

    #[test]
    fn reports_single_quoted_and_case_insensitive_roles() {
        let rule = AbstractRoleUsageRule;
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: "<div role='Widget' />".to_string(),
        };

        let diagnostics = rule.run(&file);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_non_abstract_roles() {
        let rule = AbstractRoleUsageRule;
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: "<div role=\"dialog\" />".to_string(),
        };

        let diagnostics = rule.run(&file);
        assert!(diagnostics.is_empty());
    }
}
