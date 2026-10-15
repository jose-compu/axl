use super::{element_diagnostic, parse_doc};
use axl_aria::name_computation::compute_accessible_name;
use axl_core::{Category, Diagnostic, LintFile, Rule, Severity};

pub struct SvgMissingNameRule;

impl Rule for SvgMissingNameRule {
    fn id(&self) -> &'static str {
        "naming/svg-missing-name"
    }

    fn category(&self) -> Category {
        Category::Naming
    }

    fn run(&self, file: &LintFile) -> Vec<Diagnostic> {
        let doc = parse_doc(file);
        let mut out = Vec::new();
        for element in doc.elements() {
            if !element.name.eq_ignore_ascii_case("svg") {
                continue;
            }
            if element.spread || element.is_hidden() || element.is_presentational() {
                continue;
            }
            if compute_accessible_name(&doc, element).is_missing() {
                out.push(element_diagnostic(
                    file,
                    element,
                    self.id(),
                    Severity::Error,
                    "SVG has no accessible name (add a `<title>`, `aria-label`, or `aria-labelledby`)."
                        .to_string(),
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::SvgMissingNameRule;
    use axl_core::{LintFile, Rule};

    fn count(source: &str) -> usize {
        SvgMissingNameRule
            .run(&LintFile {
                path: "fixture.tsx".to_string(),
                source: source.to_string(),
            })
            .len()
    }

    #[test]
    fn reports_svg_without_name() {
        assert_eq!(count(r#"<svg></svg>"#), 1);
    }

    #[test]
    fn accepts_title_aria_label_and_labelledby() {
        assert_eq!(count(r#"<svg><title>Logo</title></svg>"#), 0);
        assert_eq!(count(r#"<svg aria-label="Logo" />"#), 0);
        assert_eq!(count(r#"<span id="t">Logo</span><svg aria-labelledby="t" />"#), 0);
        assert_eq!(count(r#"<svg aria-hidden="true"></svg>"#), 0);
    }
}
