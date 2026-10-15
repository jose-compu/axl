use crate::jsx::{AttrValue, JsxDocument, JsxElement, JsxNode, static_string_from_expr};
use std::collections::HashSet;

const MAX_DEPTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameSource {
    None,
    LabelledBy,
    AriaLabel,
    NativeLabel,
    Contents,
    Alt,
    Title,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedName {
    pub text: String,
    pub source: NameSource,
}

impl ComputedName {
    pub fn missing() -> Self {
        Self {
            text: String::new(),
            source: NameSource::None,
        }
    }

    pub fn dynamic() -> Self {
        Self {
            text: String::new(),
            source: NameSource::Dynamic,
        }
    }

    pub fn is_missing(&self) -> bool {
        self.source != NameSource::Dynamic && self.text.is_empty()
    }

    pub fn has_name(&self) -> bool {
        !self.is_missing()
    }
}

pub fn normalize_accessible_name(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn is_empty_accessible_name(raw: &str) -> bool {
    normalize_accessible_name(raw).is_empty()
}

pub fn compute_accessible_name(doc: &JsxDocument, element: &JsxElement) -> ComputedName {
    let mut visiting = HashSet::new();
    compute(doc, element, &mut visiting, false, &HashSet::new(), 0)
}

fn compute(
    doc: &JsxDocument,
    element: &JsxElement,
    visiting: &mut HashSet<usize>,
    in_labelledby: bool,
    exclude: &HashSet<usize>,
    depth: usize,
) -> ComputedName {
    if depth > MAX_DEPTH || exclude.contains(&element.offset) {
        return ComputedName::missing();
    }
    if !visiting.insert(element.offset) {
        return ComputedName::missing();
    }

    let result = compute_uncached(doc, element, visiting, in_labelledby, exclude, depth);
    visiting.remove(&element.offset);
    result
}

fn compute_uncached(
    doc: &JsxDocument,
    element: &JsxElement,
    visiting: &mut HashSet<usize>,
    in_labelledby: bool,
    exclude: &HashSet<usize>,
    depth: usize,
) -> ComputedName {
    if !in_labelledby {
        match naming_attr(element, "aria-labelledby") {
            AttrLookup::Dynamic => return ComputedName::dynamic(),
            AttrLookup::Static(ids) => {
                if is_empty_accessible_name(&ids) {
                    return ComputedName::missing();
                }
                let mut parts = Vec::new();
                let mut dynamic = false;
                for id in ids.split_whitespace() {
                    let Some(target) = doc.find_by_id(id) else {
                        continue;
                    };
                    let nested = compute(doc, target, visiting, true, exclude, depth + 1);
                    if nested.source == NameSource::Dynamic {
                        dynamic = true;
                    }
                    if !nested.text.is_empty() {
                        parts.push(nested.text);
                    }
                }
                let text = normalize_accessible_name(&parts.join(" "));
                if !text.is_empty() {
                    return ComputedName {
                        text,
                        source: NameSource::LabelledBy,
                    };
                }
                if dynamic {
                    return ComputedName::dynamic();
                }
                return ComputedName::missing();
            }
            AttrLookup::Missing => {}
        }
    }

    match naming_attr(element, "aria-label") {
        AttrLookup::Dynamic => return ComputedName::dynamic(),
        AttrLookup::Static(label) => {
            let text = normalize_accessible_name(&label);
            if !text.is_empty() {
                return ComputedName {
                    text,
                    source: NameSource::AriaLabel,
                };
            }
        }
        AttrLookup::Missing => {}
    }

    if is_labelable(element)
        && let Some(label) = associated_label(doc, element)
    {
        let mut nested_exclude = exclude.clone();
        nested_exclude.insert(element.offset);
        let nested = compute(doc, label, visiting, in_labelledby, &nested_exclude, depth + 1);
        if nested.source == NameSource::Dynamic {
            return ComputedName::dynamic();
        }
        if !nested.text.is_empty() {
            return ComputedName {
                text: nested.text,
                source: NameSource::NativeLabel,
            };
        }
    }

    if is_svg(element)
        && let Some(title) = first_child_element(element, "title")
    {
        let contents = collect_contents(doc, title, visiting, in_labelledby, exclude, depth + 1);
        if contents.dynamic && contents.text.is_empty() {
            return ComputedName::dynamic();
        }
        let text = normalize_accessible_name(&contents.text);
        if !text.is_empty() {
            return ComputedName {
                text,
                source: NameSource::NativeLabel,
            };
        }
    }

    if allows_name_from_content(element) || in_labelledby {
        let contents = collect_contents(doc, element, visiting, in_labelledby, exclude, depth + 1);
        let text = normalize_accessible_name(&contents.text);
        if !text.is_empty() {
            return ComputedName {
                text,
                source: NameSource::Contents,
            };
        }
        if contents.dynamic {
            return ComputedName::dynamic();
        }
    }

    match naming_attr(element, "alt") {
        AttrLookup::Dynamic => return ComputedName::dynamic(),
        AttrLookup::Static(alt) => {
            let text = normalize_accessible_name(&alt);
            if !text.is_empty() {
                return ComputedName {
                    text,
                    source: NameSource::Alt,
                };
            }
            if is_replaced_image(element) {
                return ComputedName::missing();
            }
        }
        AttrLookup::Missing => {}
    }

    match naming_attr(element, "title") {
        AttrLookup::Dynamic => return ComputedName::dynamic(),
        AttrLookup::Static(title) => {
            let text = normalize_accessible_name(&title);
            if !text.is_empty() {
                return ComputedName {
                    text,
                    source: NameSource::Title,
                };
            }
        }
        AttrLookup::Missing => {}
    }

    ComputedName::missing()
}

struct CollectedContents {
    text: String,
    dynamic: bool,
}

fn collect_contents(
    doc: &JsxDocument,
    element: &JsxElement,
    visiting: &mut HashSet<usize>,
    in_labelledby: bool,
    exclude: &HashSet<usize>,
    depth: usize,
) -> CollectedContents {
    let mut parts = Vec::new();
    let mut dynamic = false;
    collect_nodes(
        doc,
        &element.children,
        visiting,
        in_labelledby,
        exclude,
        depth,
        &mut parts,
        &mut dynamic,
    );
    CollectedContents {
        text: parts.join(" "),
        dynamic,
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_nodes(
    doc: &JsxDocument,
    nodes: &[JsxNode],
    visiting: &mut HashSet<usize>,
    in_labelledby: bool,
    exclude: &HashSet<usize>,
    depth: usize,
    parts: &mut Vec<String>,
    dynamic: &mut bool,
) {
    for node in nodes {
        match node {
            JsxNode::Text { value, .. } => {
                let text = normalize_accessible_name(value);
                if !text.is_empty() {
                    parts.push(text);
                }
            }
            JsxNode::Expression { raw, nested, .. } => {
                if let Some(value) = static_string_from_expr(raw) {
                    let text = normalize_accessible_name(&value);
                    if !text.is_empty() {
                        parts.push(text);
                    }
                } else {
                    if !raw.trim().is_empty() {
                        *dynamic = true;
                    }
                    collect_nodes(doc, nested, visiting, in_labelledby, exclude, depth, parts, dynamic);
                }
            }
            JsxNode::Element(child) => {
                if exclude.contains(&child.offset) || child.is_hidden() {
                    continue;
                }
                let nested = compute(doc, child, visiting, in_labelledby, exclude, depth);
                if nested.source == NameSource::Dynamic {
                    *dynamic = true;
                }
                if !nested.text.is_empty() {
                    parts.push(nested.text);
                }
            }
        }
    }
}

enum AttrLookup {
    Missing,
    Static(String),
    Dynamic,
}

fn naming_attr(element: &JsxElement, name: &str) -> AttrLookup {
    match element.attr(name).map(|attr| &attr.value) {
        None => AttrLookup::Missing,
        Some(AttrValue::BooleanTrue) => AttrLookup::Missing,
        Some(AttrValue::String(value)) => AttrLookup::Static(value.clone()),
        Some(AttrValue::Expression(expr)) => match static_string_from_expr(expr) {
            Some(value) => AttrLookup::Static(value),
            None => AttrLookup::Dynamic,
        },
    }
}

fn is_labelable(element: &JsxElement) -> bool {
    matches!(
        element.name.to_ascii_lowercase().as_str(),
        "input" | "textarea" | "select" | "meter" | "output" | "progress"
    )
}

fn is_svg(element: &JsxElement) -> bool {
    element.name.eq_ignore_ascii_case("svg")
}

fn allows_name_from_content(element: &JsxElement) -> bool {
    !matches!(
        element.name.to_ascii_lowercase().as_str(),
        "input"
            | "img"
            | "area"
            | "br"
            | "hr"
            | "col"
            | "wbr"
            | "source"
            | "track"
            | "embed"
            | "textarea"
            | "select"
            | "optgroup"
    )
}

fn is_replaced_image(element: &JsxElement) -> bool {
    if element.name.eq_ignore_ascii_case("img") || element.name.eq_ignore_ascii_case("area") {
        return true;
    }
    element.name.eq_ignore_ascii_case("input")
        && element
            .static_attr("type")
            .is_some_and(|value| value.eq_ignore_ascii_case("image"))
}

fn associated_label<'a>(doc: &'a JsxDocument, element: &JsxElement) -> Option<&'a JsxElement> {
    if let Some(id) = element.static_attr("id") {
        let labeled = doc.elements().into_iter().find(|candidate| {
            candidate.name.eq_ignore_ascii_case("label")
                && candidate
                    .static_attr("htmlFor")
                    .or_else(|| candidate.static_attr("for"))
                    .as_deref()
                    == Some(id.as_str())
        });
        if labeled.is_some() {
            return labeled;
        }
    }
    doc.elements().into_iter().find(|candidate| {
        candidate.name.eq_ignore_ascii_case("label")
            && candidate.offset != element.offset
            && JsxDocument::contains_offset(candidate, element.offset)
    })
}

fn first_child_element<'a>(element: &'a JsxElement, name: &str) -> Option<&'a JsxElement> {
    element.children.iter().find_map(|node| match node {
        JsxNode::Element(child) if child.name.eq_ignore_ascii_case(name) => Some(child),
        JsxNode::Expression { nested, .. } => nested.iter().find_map(|inner| match inner {
            JsxNode::Element(child) if child.name.eq_ignore_ascii_case(name) => Some(child),
            _ => None,
        }),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        ComputedName, NameSource, compute_accessible_name, is_empty_accessible_name,
        normalize_accessible_name,
    };
    use crate::jsx::parse_jsx;

    fn first_name(source: &str, tag: &str) -> ComputedName {
        let doc = parse_jsx(source);
        let element = doc
            .elements()
            .into_iter()
            .find(|el| el.name.eq_ignore_ascii_case(tag))
            .unwrap_or_else(|| panic!("expected <{tag}> in fixture"));
        let computed = compute_accessible_name(&doc, element);
        println!("AccName <{tag}> from `{source}` => {:?} {:?}", computed.source, computed.text);
        computed
    }

    #[test]
    fn normalize_collapses_whitespace_runs() {
        let input = "  Save \n\t  as   draft  ";
        assert_eq!(normalize_accessible_name(input), "Save as draft");
    }

    #[test]
    fn normalize_empty_input_returns_empty() {
        assert_eq!(normalize_accessible_name(""), "");
        assert_eq!(normalize_accessible_name("   \n\t  "), "");
    }

    #[test]
    fn empty_name_detection_uses_normalization() {
        assert!(is_empty_accessible_name("   \n\t  "));
        assert!(!is_empty_accessible_name("  Submit  "));
    }

    #[test]
    fn labelledby_takes_precedence_and_joins_ids() {
        let source = r#"
            <span id="a">Save</span>
            <span id="b">draft</span>
            <button aria-labelledby="a b">ignored</button>
        "#;
        let name = first_name(source, "button");
        assert_eq!(name.source, NameSource::LabelledBy);
        assert_eq!(name.text, "Save draft");
    }

    #[test]
    fn aria_label_wins_over_contents_and_normalizes() {
        let name = first_name(r#"<button aria-label="  Close  window  ">X</button>"#, "button");
        assert_eq!(name.source, NameSource::AriaLabel);
        assert_eq!(name.text, "Close window");
    }

    #[test]
    fn empty_aria_label_falls_through_to_contents() {
        let name = first_name(r#"<button aria-label="   ">Click</button>"#, "button");
        assert_eq!(name.source, NameSource::Contents);
        assert_eq!(name.text, "Click");
    }

    #[test]
    fn native_label_via_html_for() {
        let source = r#"
            <label htmlFor="email">Email address</label>
            <input id="email" />
        "#;
        let name = first_name(source, "input");
        assert_eq!(name.source, NameSource::NativeLabel);
        assert_eq!(name.text, "Email address");
    }

    #[test]
    fn wrapping_label_excludes_the_control() {
        let source = r#"<label>Phone <input id="phone" /></label>"#;
        let doc = parse_jsx(source);
        let input = doc.find_by_id("phone").expect("input");
        let name = compute_accessible_name(&doc, input);
        println!("wrapping label => {:?} {:?}", name.source, name.text);
        assert_eq!(name.source, NameSource::NativeLabel);
        assert_eq!(name.text, "Phone");
    }

    #[test]
    fn img_alt_and_title_fallback() {
        let alt = first_name(r#"<img alt="  A  cat " title="ignored" />"#, "img");
        assert_eq!(alt.source, NameSource::Alt);
        assert_eq!(alt.text, "A cat");

        let title = first_name(r#"<button title="Help" />"#, "button");
        assert_eq!(title.source, NameSource::Title);
        assert_eq!(title.text, "Help");
    }

    #[test]
    fn svg_title_child_is_a_native_name() {
        let name = first_name(r#"<svg><title>  Site   logo </title></svg>"#, "svg");
        assert_eq!(name.source, NameSource::NativeLabel);
        assert_eq!(name.text, "Site logo");
    }

    #[test]
    fn recursion_guard_breaks_self_labelledby() {
        let name = first_name(r#"<button id="loop" aria-labelledby="loop">Click</button>"#, "button");
        assert!(name.text.is_empty());
        assert_eq!(name.source, NameSource::None);
    }

    #[test]
    fn labelledby_skips_nested_labelledby_and_uses_contents() {
        let source = r#"
            <span id="a" aria-labelledby="b">unused</span>
            <span id="b">Visible</span>
            <button aria-labelledby="a" />
        "#;
        let name = first_name(source, "button");
        assert_eq!(name.text, "unused");
        assert_eq!(name.source, NameSource::LabelledBy);
    }

    #[test]
    fn missing_labelledby_target_does_not_fall_through() {
        let name = first_name(r#"<button aria-labelledby="missing">Click</button>"#, "button");
        assert!(name.is_missing());
        assert_eq!(name.source, NameSource::None);
    }

    #[test]
    fn empty_img_alt_does_not_fall_through_to_title() {
        let name = first_name(r#"<img alt="" title="ignored" />"#, "img");
        assert!(name.is_missing());
    }

    #[test]
    fn name_from_content_includes_descendant_text() {
        let name = first_name(r#"<button>Save <span>now</span></button>"#, "button");
        assert_eq!(name.source, NameSource::Contents);
        assert_eq!(name.text, "Save now");
    }

    #[test]
    fn name_from_content_uses_descendant_alt() {
        let name = first_name(r#"<a href="/"><img alt="Home" /></a>"#, "a");
        assert_eq!(name.source, NameSource::Contents);
        assert_eq!(name.text, "Home");
    }

    #[test]
    fn dynamic_aria_label_is_not_treated_as_empty() {
        let name = first_name(r#"<button aria-label={label} />"#, "button");
        assert_eq!(name.source, NameSource::Dynamic);
        assert!(name.has_name());
        assert!(!name.is_missing());
    }

    #[test]
    fn hidden_descendants_are_skipped() {
        let name = first_name(
            r#"<button>Save <span aria-hidden="true">icon</span></button>"#,
            "button",
        );
        assert_eq!(name.text, "Save");
    }
}
