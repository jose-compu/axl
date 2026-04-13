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
        for (offset, _) in file.source.match_indices("role=\"") {
            let value_start = offset + "role=\"".len();
            if let Some(end_rel) = file.source[value_start..].find('"') {
                let role = &file.source[value_start..value_start + end_rel];
                if is_abstract_role(role) {
                    let (line, column) = line_col_from_offset(&file.source, offset);
                    out.push(Diagnostic {
                        file: file.path.clone(),
                        line,
                        column,
                        severity: Severity::Error,
                        rule_id: self.id(),
                        message: format!("Abstract ARIA role `{role}` is not valid on elements."),
                    });
                }
            }
        }
        out
    }
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
