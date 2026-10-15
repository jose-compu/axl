use super::{element_diagnostic, parse_doc};
use axl_aria::name_computation::compute_accessible_name;
use axl_core::{Category, Diagnostic, LintFile, Rule, Severity};

pub struct LinkMissingNameRule;

impl Rule for LinkMissingNameRule {
    fn id(&self) -> &'static str {
        "naming/link-missing-name"
    }

    fn category(&self) -> Category {
        Category::Naming
    }

    fn run(&self, file: &LintFile) -> Vec<Diagnostic> {
        let doc = parse_doc(file);
        let mut out = Vec::new();
        for element in doc.elements() {
            if !element.name.eq_ignore_ascii_case("a") {
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
                    "Link has no accessible name.".to_string(),
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::LinkMissingNameRule;
    use axl_core::{LintFile, Rule};

    fn count(source: &str) -> usize {
        LinkMissingNameRule
            .run(&LintFile {
                path: "fixture.tsx".to_string(),
                source: source.to_string(),
            })
            .len()
    }

    #[test]
    fn reports_empty_link() {
        assert_eq!(count(r#"<a href="/"></a>"#), 1);
    }

    #[test]
    fn accepts_text_and_img_alt() {
        assert_eq!(count(r#"<a href="/home">Home</a>"#), 0);
        assert_eq!(count(r#"<a href="/"><img alt="Home" /></a>"#), 0);
        assert_eq!(count(r#"<a href="/" aria-label="Home" />"#), 0);
    }
}
