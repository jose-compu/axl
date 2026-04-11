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
        for attr in ["aria-label=\"", "title=\""] {
            for (offset, _) in file.source.match_indices(attr) {
                let value_start = offset + attr.len();
                if let Some(end_rel) = file.source[value_start..].find('"') {
                    let value = &file.source[value_start..value_start + end_rel];
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
        }
        out
    }
}
