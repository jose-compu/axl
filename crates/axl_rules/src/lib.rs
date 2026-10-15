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
        Some(values) => values
            .iter()
            .any(|value| Category::parse(value) == Some(category)),
    }
}

fn registry() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(roles::abstract_role_usage::AbstractRoleUsageRule),
        Box::new(naming::empty_accessible_name::EmptyAccessibleNameRule),
        Box::new(naming::img_missing_alt::ImgMissingAltRule),
        Box::new(naming::button_missing_name::ButtonMissingNameRule),
        Box::new(naming::link_missing_name::LinkMissingNameRule),
        Box::new(naming::svg_missing_name::SvgMissingNameRule),
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
        assert_eq!(result.diagnostics.len(), 4);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.rule_id == "naming/button-missing-name")
        );
    }

    #[test]
    fn only_categories_filters_results() {
        let file = LintFile {
            path: "fixture.tsx".to_string(),
            source: r#"
                <div role="widget" />
                <button aria-label="   " tabIndex={3}>Go</button>
            "#
            .to_string(),
        };
        let only = vec!["Roles".to_string()];

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
                <button aria-label="   " tabIndex={3}>Go</button>
            "#
            .to_string(),
        };
        let only = vec!["not-a-category".to_string()];

        let result = lint_file(&file, Some(&only));
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn naming_rules_cover_labelledby_aria_label_img_alt_and_button_text() {
        let file = LintFile {
            path: "accname.tsx".to_string(),
            source: r#"
                <section>
                  <span id="by-id">Name from labelledby</span>
                  <button aria-labelledby="by-id" aria-label="ignored">ignored</button>
                  <button aria-label="Name from aria-label">ignored</button>
                  <img src="/logo.png" alt="Name from alt" />
                  <button>Name from button text</button>
                </section>
            "#
            .to_string(),
        };

        let result = lint_file(&file, Some(&["naming".to_string()]));
        assert!(
            result.diagnostics.is_empty(),
            "{:?}",
            result.diagnostics
        );
    }

    #[test]
    fn naming_rules_report_img_button_link_and_svg_gaps() {
        let file = LintFile {
            path: "invalid_naming.tsx".to_string(),
            source: r#"
                <section>
                  <img src="/x.png" />
                  <button></button>
                  <a href="/go"></a>
                  <svg></svg>
                </section>
            "#
            .to_string(),
        };

        let result = lint_file(&file, Some(&["naming".to_string()]));
        let ids: Vec<_> = result.diagnostics.iter().map(|d| d.rule_id).collect();
        assert!(ids.contains(&"naming/img-missing-alt"));
        assert!(ids.contains(&"naming/button-missing-name"));
        assert!(ids.contains(&"naming/link-missing-name"));
        assert!(ids.contains(&"naming/svg-missing-name"));
        assert_eq!(result.diagnostics.len(), 4);
    }

    #[test]
    fn autofix_updates_positive_tabindex_values() {
        let source = "<button tabIndex={5} /><div tabindex={2} /><a tabIndex={index} />";
        let (fixed, applied) = autofix_file_source(source);

        assert_eq!(applied, 2);
        assert_eq!(fixed, "<button tabIndex={0} /><div tabindex={0} /><a tabIndex={index} />");
    }
}
