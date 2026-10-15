use super::{element_diagnostic, parse_doc};
use axl_aria::name_computation::compute_accessible_name;
use axl_core::{Category, Diagnostic, LintFile, Rule, Severity};

pub struct ButtonMissingNameRule;

impl Rule for ButtonMissingNameRule {
    fn id(&self) -> &'static str {
        "naming/button-missing-name"
    }

    fn category(&self) -> Category {
        Category::Naming
    }

    fn run(&self, file: &LintFile) -> Vec<Diagnostic> {
        let doc = parse_doc(file);
        let mut out = Vec::new();
        for element in doc.elements() {
            if !element.name.eq_ignore_ascii_case("button") {
                continue;
            }
            if element.spread || element.is_hidden() || element.is_presentational() {
                continue;
            }
            let name = compute_accessible_name(&doc, element);
            if name.is_missing() {
                out.push(element_diagnostic(
                    file,
                    element,
                    self.id(),
                    Severity::Error,
                    "Button has no accessible name.".to_string(),
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::ButtonMissingNameRule;
    use axl_core::{LintFile, Rule};

    fn diagnostics(source: &str) -> Vec<String> {
        ButtonMissingNameRule
            .run(&LintFile {
                path: "fixture.tsx".to_string(),
                source: source.to_string(),
            })
            .into_iter()
            .map(|d| d.rule_id.to_string())
            .collect()
    }

    #[test]
    fn reports_empty_button() {
        let found = diagnostics(r#"<button></button>"#);
        assert_eq!(found, vec!["naming/button-missing-name"]);
    }

    #[test]
    fn accepts_text_aria_label_and_labelledby() {
        assert!(diagnostics(r#"<button>Save</button>"#).is_empty());
        assert!(diagnostics(r#"<button aria-label="Save" />"#).is_empty());
        assert!(
            diagnostics(r#"<span id="lbl">Save</span><button aria-labelledby="lbl" />"#).is_empty()
        );
        assert!(diagnostics(r#"<button><img alt="Save" /></button>"#).is_empty());
    }

    #[test]
    fn ignores_dynamic_and_spread_names() {
        assert!(diagnostics(r#"<button>{label}</button>"#).is_empty());
        assert!(diagnostics(r#"<button aria-label={label} />"#).is_empty());
        assert!(diagnostics(r#"<button {...props} />"#).is_empty());
    }
}
