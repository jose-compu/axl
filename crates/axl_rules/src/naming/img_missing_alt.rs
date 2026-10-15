use super::{element_diagnostic, parse_doc};
use axl_core::{Category, Diagnostic, LintFile, Rule, Severity};

pub struct ImgMissingAltRule;

impl Rule for ImgMissingAltRule {
    fn id(&self) -> &'static str {
        "naming/img-missing-alt"
    }

    fn category(&self) -> Category {
        Category::Naming
    }

    fn run(&self, file: &LintFile) -> Vec<Diagnostic> {
        let doc = parse_doc(file);
        let mut out = Vec::new();
        for element in doc.elements() {
            if !is_image_like(element) {
                continue;
            }
            if element.spread || element.is_hidden() || element.is_presentational() {
                continue;
            }
            if element.has_attr("alt") {
                continue;
            }
            if element.has_attr("aria-label") || element.has_attr("aria-labelledby") {
                continue;
            }
            out.push(element_diagnostic(
                file,
                element,
                self.id(),
                Severity::Error,
                image_message(element),
            ));
        }
        out
    }
}

fn is_image_like(element: &axl_aria::jsx::JsxElement) -> bool {
    if element.name.eq_ignore_ascii_case("img") {
        return true;
    }
    element.name.eq_ignore_ascii_case("input")
        && element
            .static_attr("type")
            .is_some_and(|value| value.eq_ignore_ascii_case("image"))
}

fn image_message(element: &axl_aria::jsx::JsxElement) -> String {
    if element.name.eq_ignore_ascii_case("input") {
        "Image input is missing an `alt` attribute.".to_string()
    } else {
        "Image is missing an `alt` attribute.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::ImgMissingAltRule;
    use axl_core::{LintFile, Rule};

    fn run(source: &str) -> usize {
        ImgMissingAltRule
            .run(&LintFile {
                path: "fixture.tsx".to_string(),
                source: source.to_string(),
            })
            .len()
    }

    #[test]
    fn reports_img_without_alt() {
        assert_eq!(run(r#"<img src="/cat.png" />"#), 1);
        println!("img without alt => 1 diagnostic");
    }

    #[test]
    fn accepts_empty_alt_and_aria_name() {
        assert_eq!(run(r#"<img alt="" src="/deco.png" />"#), 0);
        assert_eq!(run(r#"<img alt="Cat" src="/cat.png" />"#), 0);
        assert_eq!(run(r#"<img aria-label="Cat" src="/cat.png" />"#), 0);
        assert_eq!(run(r#"<img aria-labelledby="cap" src="/cat.png" />"#), 0);
    }

    #[test]
    fn ignores_presentational_hidden_and_spread() {
        assert_eq!(run(r#"<img role="presentation" src="/x.png" />"#), 0);
        assert_eq!(run(r#"<img aria-hidden="true" src="/x.png" />"#), 0);
        assert_eq!(run(r#"<img {...props} />"#), 0);
    }

    #[test]
    fn reports_image_input_without_alt() {
        assert_eq!(run(r#"<input type="image" src="/go.png" />"#), 1);
        assert_eq!(run(r#"<input type="image" alt="Go" src="/go.png" />"#), 0);
    }
}
