use axl_core::{Category, Diagnostic, LintFile, Rule, Severity, line_col_from_offset};

pub struct TabindexPositiveRule;

const EXPR_ATTRS: [&str; 2] = ["tabIndex={", "tabindex={"];
const QUOTED_ATTRS: [(&str, char); 4] = [
    ("tabIndex=\"", '"'),
    ("tabindex=\"", '"'),
    ("tabIndex='", '\''),
    ("tabindex='", '\''),
];

impl Rule for TabindexPositiveRule {
    fn id(&self) -> &'static str {
        "focus/tabindex-positive"
    }

    fn category(&self) -> Category {
        Category::Focus
    }

    fn run(&self, file: &LintFile) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for (offset, raw) in tabindex_values(&file.source) {
            if let Some(value) = parse_positive_tabindex(raw) {
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
        out
    }
}

pub fn autofix_positive_tabindex(source: &str) -> (String, usize) {
    let mut replacements = Vec::new();
    for (offset, raw) in tabindex_values(source) {
        if parse_positive_tabindex(raw).is_some() {
            replacements.push((offset + raw_start_offset(source, offset), raw.len(), "0"));
        }
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;
    let mut fixes = 0usize;
    for (start, len, replacement) in replacements {
        out.push_str(&source[cursor..start]);
        out.push_str(replacement);
        cursor = start + len;
        fixes += 1;
    }
    out.push_str(&source[cursor..]);
    (out, fixes)
}

fn tabindex_values(source: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    for attr in EXPR_ATTRS {
        for (offset, _) in source.match_indices(attr) {
            let start = offset + attr.len();
            if let Some(end_rel) = source[start..].find('}') {
                out.push((offset, &source[start..start + end_rel]));
            }
        }
    }
    for (attr, quote) in QUOTED_ATTRS {
        for (offset, _) in source.match_indices(attr) {
            let start = offset + attr.len();
            if let Some(end_rel) = source[start..].find(quote) {
                out.push((offset, &source[start..start + end_rel]));
            }
        }
    }
    out.sort_by_key(|(offset, _)| *offset);
    out
}

fn raw_start_offset(source: &str, offset: usize) -> usize {
    let remainder = &source[offset..];
    for attr in EXPR_ATTRS {
        if remainder.starts_with(attr) {
            return attr.len();
        }
    }
    for (attr, _) in QUOTED_ATTRS {
        if remainder.starts_with(attr) {
            return attr.len();
        }
    }
    0
}

fn parse_positive_tabindex(raw: &str) -> Option<i32> {
    let value = raw.trim().parse::<i32>().ok()?;
    (value > 0).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::{TabindexPositiveRule, autofix_positive_tabindex};
    use axl_core::{LintFile, Rule};

    #[test]
    fn reports_positive_tabindex_values() {
        let rule = TabindexPositiveRule;
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: "<button tabIndex={3} /><div tabindex={1} />".to_string(),
        };

        let diagnostics = rule.run(&file);
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().all(|d| d.rule_id == "focus/tabindex-positive"));
    }

    #[test]
    fn reports_quoted_positive_tabindex_values() {
        let rule = TabindexPositiveRule;
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: "<button tabIndex=\"4\" /><div tabindex='2' />".to_string(),
        };

        let diagnostics = rule.run(&file);
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn ignores_zero_negative_and_non_numeric_tabindex() {
        let rule = TabindexPositiveRule;
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: "<button tabIndex={0} /><div tabindex={-1} /><a tabIndex={idx} />".to_string(),
        };

        let diagnostics = rule.run(&file);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn autofix_rewrites_positive_values_to_zero() {
        let source = "<button tabIndex={3} /><div tabindex={2} />";
        let (fixed, count) = autofix_positive_tabindex(source);

        assert_eq!(count, 2);
        assert_eq!(fixed, "<button tabIndex={0} /><div tabindex={0} />");
    }

    #[test]
    fn autofix_rewrites_quoted_positive_values_to_zero() {
        let source = "<button tabIndex=\"5\" /><div tabindex='8' />";
        let (fixed, count) = autofix_positive_tabindex(source);

        assert_eq!(count, 2);
        assert_eq!(fixed, "<button tabIndex=\"0\" /><div tabindex='0' />");
    }

    #[test]
    fn autofix_leaves_non_positive_or_dynamic_values_unchanged() {
        let source = "<button tabIndex={0} /><div tabindex={-1} /><a tabIndex={index} />";
        let (fixed, count) = autofix_positive_tabindex(source);

        assert_eq!(count, 0);
        assert_eq!(fixed, source);
    }

    #[test]
    fn autofix_handles_trimmed_numeric_values() {
        let source = "<button tabIndex={   9   } />";
        let (fixed, count) = autofix_positive_tabindex(source);

        assert_eq!(count, 1);
        assert_eq!(fixed, "<button tabIndex={0} />");
    }

    #[test]
    fn autofix_ignores_unclosed_expressions() {
        let source = "<button tabIndex={4";
        let (fixed, count) = autofix_positive_tabindex(source);

        assert_eq!(count, 0);
        assert_eq!(fixed, source);
    }
}
