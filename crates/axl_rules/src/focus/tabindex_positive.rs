use axl_core::{Category, Diagnostic, LintFile, Rule, Severity, line_col_from_offset};

pub struct TabindexPositiveRule;

const TABINDEX_ATTRS: [&str; 2] = ["tabIndex={", "tabindex={"];

impl Rule for TabindexPositiveRule {
    fn id(&self) -> &'static str {
        "focus/tabindex-positive"
    }

    fn category(&self) -> Category {
        Category::Focus
    }

    fn run(&self, file: &LintFile) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for attr in TABINDEX_ATTRS {
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

pub fn autofix_positive_tabindex(source: &str) -> (String, usize) {
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;
    let mut fixes = 0usize;

    while cursor < source.len() {
        let remainder = &source[cursor..];
        let matched_attr = TABINDEX_ATTRS.iter().find(|attr| remainder.starts_with(**attr));

        if let Some(attr) = matched_attr {
            out.push_str(attr);
            cursor += attr.len();

            if let Some(end_rel) = source[cursor..].find('}') {
                let raw_value = &source[cursor..cursor + end_rel];
                if let Ok(value) = raw_value.trim().parse::<i32>() {
                    if value > 0 {
                        out.push('0');
                        fixes += 1;
                        cursor += end_rel;
                        continue;
                    }
                }

                out.push_str(raw_value);
                cursor += end_rel;
                continue;
            }

            out.push_str(&source[cursor..]);
            break;
        }

        if let Some(ch) = remainder.chars().next() {
            out.push(ch);
            cursor += ch.len_utf8();
        } else {
            break;
        }
    }

    (out, fixes)
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
