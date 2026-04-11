use axl_core::{Category, Diagnostic, LintFile, Rule, Severity, line_col_from_offset};

pub struct TabindexPositiveRule;

impl Rule for TabindexPositiveRule {
    fn id(&self) -> &'static str {
        "focus/tabindex-positive"
    }

    fn category(&self) -> Category {
        Category::Focus
    }

    fn run(&self, file: &LintFile) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for attr in ["tabIndex={", "tabindex={"] {
            for (offset, _) in file.source.match_indices(attr) {
                let start = offset + attr.len();
                if let Some(end_rel) = file.source[start..].find('}') {
                    let raw = &file.source[start..start + end_rel];
                    if let Ok(value) = raw.trim().parse::<i32>() {
                        if value > 0 {
                            let (line, column) = line_col_from_offset(&file.source, offset);
                            out.push(Diagnostic {
                                file: file.path.clone(),
                                line,
                                column,
                                severity: Severity::Warn,
                                rule_id: self.id(),
                                message: format!(
                                    "Avoid positive tabIndex (`{value}`); it breaks natural tab order."
                                ),
                            });
                        }
                    }
                }
            }
        }
        out
    }
}
