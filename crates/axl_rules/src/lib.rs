mod focus;
mod naming;
mod roles;

use axl_core::{Category, LintFile, LintResult, Rule};

pub fn lint_file(file: &LintFile, only_categories: Option<&[String]>) -> LintResult {
    let mut result = LintResult::new();
    for rule in registry() {
        if !category_allowed(rule.category(), only_categories) {
            continue;
        }
        for diagnostic in rule.run(file) {
            result.push(diagnostic);
        }
    }
    result
}

pub fn autofix_file_source(source: &str) -> (String, usize) {
    focus::tabindex_positive::autofix_positive_tabindex(source)
}

fn category_allowed(category: Category, only_categories: Option<&[String]>) -> bool {
    match only_categories {
        None => true,
        Some(values) => values.iter().any(|value| value == category.as_str()),
    }
}

fn registry() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(roles::abstract_role_usage::AbstractRoleUsageRule),
        Box::new(naming::empty_accessible_name::EmptyAccessibleNameRule),
        Box::new(focus::tabindex_positive::TabindexPositiveRule),
    ]
}

#[cfg(test)]
mod tests {
    use super::{autofix_file_source, lint_file};
    use axl_core::LintFile;

    #[test]
    fn detects_expected_diagnostics() {
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: r#"
                <div role="widget" />
                <button aria-label="   " tabIndex={3} />
            "#
            .to_string(),
        };

        let result = lint_file(&file, None);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn only_categories_filters_results() {
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: r#"
                <div role="widget" />
                <button aria-label="   " tabIndex={3} />
            "#
            .to_string(),
        };
        let only = vec!["roles".to_string()];

        let result = lint_file(&file, Some(&only));
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].rule_id, "roles/abstract-role-usage");
    }

    #[test]
    fn unknown_only_category_returns_no_results() {
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: r#"
                <div role="widget" />
                <button aria-label="   " tabIndex={3} />
            "#
            .to_string(),
        };
        let only = vec!["not-a-category".to_string()];

        let result = lint_file(&file, Some(&only));
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn autofix_updates_positive_tabindex_values() {
        let source = "<button tabIndex={5} /><div tabindex={2} /><a tabIndex={index} />";
        let (fixed, applied) = autofix_file_source(source);

        assert_eq!(applied, 2);
        assert_eq!(fixed, "<button tabIndex={0} /><div tabindex={0} /><a tabIndex={index} />");
    }
}
