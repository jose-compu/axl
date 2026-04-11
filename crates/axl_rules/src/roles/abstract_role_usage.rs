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
