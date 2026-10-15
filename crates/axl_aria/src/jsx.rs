//! Lightweight same-file JSX/TSX element extraction for AccName.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrValue {
    String(String),
    Expression(String),
    BooleanTrue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsxAttribute {
    pub offset: usize,
    pub name: String,
    pub value: AttrValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsxNode {
    Element(JsxElement),
    Text { offset: usize, value: String },
    Expression { offset: usize, raw: String, nested: Vec<JsxNode> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsxElement {
    pub offset: usize,
    pub name: String,
    pub attributes: Vec<JsxAttribute>,
    pub children: Vec<JsxNode>,
    pub self_closing: bool,
    pub spread: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsxDocument {
    pub roots: Vec<JsxNode>,
}

impl JsxDocument {
    pub fn elements(&self) -> Vec<&JsxElement> {
        let mut out = Vec::new();
        collect_elements(&self.roots, &mut out);
        out
    }

    pub fn find_by_id(&self, id: &str) -> Option<&JsxElement> {
        self.elements()
            .into_iter()
            .find(|el| el.static_attr("id").as_deref() == Some(id))
    }

    pub fn contains_offset(element: &JsxElement, offset: usize) -> bool {
        if element.offset == offset {
            return true;
        }
        let mut nested = Vec::new();
        collect_elements(&element.children, &mut nested);
        nested.iter().any(|child| Self::contains_offset(child, offset))
    }
}

impl JsxElement {
    pub fn attr(&self, name: &str) -> Option<&JsxAttribute> {
        self.attributes
            .iter()
            .find(|attr| attr.name.eq_ignore_ascii_case(name))
    }

    pub fn static_attr(&self, name: &str) -> Option<String> {
        match &self.attr(name)?.value {
            AttrValue::String(value) => Some(value.clone()),
            AttrValue::Expression(expr) => static_string_from_expr(expr),
            AttrValue::BooleanTrue => None,
        }
    }

    pub fn has_attr(&self, name: &str) -> bool {
        self.attr(name).is_some()
    }

    pub fn is_hidden(&self) -> bool {
        if self.has_attr("hidden") {
            return match self.attr("hidden").map(|attr| &attr.value) {
                Some(AttrValue::String(value)) => !value.trim().eq_ignore_ascii_case("false"),
                Some(AttrValue::Expression(expr)) => !matches!(expr.trim(), "false" | "0"),
                Some(AttrValue::BooleanTrue) | None => true,
            };
        }
        match self.attr("aria-hidden").map(|attr| &attr.value) {
            Some(AttrValue::BooleanTrue) => true,
            Some(AttrValue::String(value)) => is_true_token(value),
            Some(AttrValue::Expression(expr)) => match static_string_from_expr(expr) {
                Some(value) => is_true_token(&value),
                None => matches!(expr.trim(), "true" | "1"),
            },
            None => false,
        }
    }

    pub fn is_presentational(&self) -> bool {
        self.static_attr("role").is_some_and(|role| {
            crate::roles::split_roles(&role)
                .any(|token| token.eq_ignore_ascii_case("presentation") || token.eq_ignore_ascii_case("none"))
        })
    }
}

pub fn parse_jsx(source: &str) -> JsxDocument {
    let mut scanner = Scanner::new(source);
    let mut roots = Vec::new();
    while !scanner.is_eof() {
        scanner.skip_non_jsx();
        if scanner.is_eof() {
            break;
        }
        if scanner.looks_like_jsx()
            && let Some(node) = scanner.parse_child_node()
        {
            roots.push(node);
            continue;
        }
        scanner.bump();
    }
    JsxDocument { roots }
}

pub fn static_string_from_expr(expr: &str) -> Option<String> {
    let trimmed = expr.trim();
    for quote in ['"', '\''] {
        if trimmed.len() >= 2 && trimmed.starts_with(quote) && trimmed.ends_with(quote) {
            let inner = &trimmed[1..trimmed.len() - 1];
            if !inner.contains(quote) {
                return Some(inner.to_string());
            }
        }
    }
    if trimmed.len() >= 2 && trimmed.starts_with('`') && trimmed.ends_with('`') && !trimmed.contains("${") {
        return Some(trimmed[1..trimmed.len() - 1].to_string());
    }
    None
}

fn is_true_token(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.eq_ignore_ascii_case("true") || trimmed == "1"
}

fn collect_elements<'a>(nodes: &'a [JsxNode], out: &mut Vec<&'a JsxElement>) {
    for node in nodes {
        match node {
            JsxNode::Element(element) => {
                out.push(element);
                collect_elements(&element.children, out);
            }
            JsxNode::Expression { nested, .. } => collect_elements(nested, out),
            JsxNode::Text { .. } => {}
        }
    }
}

struct Scanner<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Scanner<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn starts_with(&self, token: &str) -> bool {
        self.rest().starts_with(token)
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|ch| ch.is_whitespace()) {
            self.bump();
        }
    }

    fn skip_non_jsx(&mut self) {
        while !self.is_eof() {
            self.skip_ws();
            if self.is_eof() {
                return;
            }
            if self.starts_with("//") {
                self.skip_line_comment();
                continue;
            }
            if self.starts_with("/*") {
                self.skip_block_comment();
                continue;
            }
            if matches!(self.peek(), Some('"' | '\'')) {
                self.skip_quoted(self.peek().unwrap());
                continue;
            }
            if self.peek() == Some('`') {
                self.skip_template();
                continue;
            }
            if self.looks_like_jsx() {
                return;
            }
            self.bump();
        }
    }

    fn looks_like_jsx(&self) -> bool {
        let rest = self.rest();
        let Some(after) = rest.strip_prefix('<') else {
            return false;
        };
        if after.starts_with('>') || after.starts_with("/>") {
            return true;
        }
        if let Some(end) = after.strip_prefix('/') {
            return end.starts_with('>') || end.starts_with(|ch: char| is_name_start(ch));
        }
        if !after.starts_with(is_name_start) {
            return false;
        }
        let name_len = name_len(after);
        let after_name = after[name_len..].trim_start();
        after_name.starts_with('>')
            || after_name.starts_with("/>")
            || after_name.starts_with('{')
            || after_name.starts_with(is_attr_start)
    }

    fn parse_child_node(&mut self) -> Option<JsxNode> {
        self.skip_ws();
        if self.starts_with("{/*") {
            self.skip_jsx_comment();
            return self.parse_child_node();
        }
        if self.peek() == Some('{') {
            return Some(self.parse_expression());
        }
        if self.looks_like_jsx() {
            if self.starts_with("</") {
                return None;
            }
            return self.parse_element().map(JsxNode::Element);
        }
        None
    }

    fn parse_element(&mut self) -> Option<JsxElement> {
        let offset = self.pos;
        if self.bump() != Some('<') {
            return None;
        }
        if self.peek() == Some('>') {
            self.bump();
            let children = self.parse_children("");
            return Some(JsxElement {
                offset,
                name: String::new(),
                attributes: Vec::new(),
                children,
                self_closing: false,
                spread: false,
            });
        }
        let name = self.parse_name()?;
        let (attributes, spread, self_closing) = self.parse_attributes()?;
        let children = if self_closing {
            Vec::new()
        } else {
            self.parse_children(&name)
        };
        Some(JsxElement {
            offset,
            name,
            attributes,
            children,
            self_closing,
            spread,
        })
    }

    fn parse_children(&mut self, end_name: &str) -> Vec<JsxNode> {
        let mut children = Vec::new();
        while !self.is_eof() {
            if self.starts_with("</") {
                let saved = self.pos;
                self.pos += 2;
                let closing = if self.peek() == Some('>') {
                    String::new()
                } else {
                    self.parse_name().unwrap_or_default()
                };
                self.skip_ws();
                if self.peek() == Some('>') {
                    self.bump();
                }
                if closing.is_empty() || closing == end_name || end_name.is_empty() {
                    break;
                }
                self.pos = saved;
                self.bump();
                continue;
            }
            if self.starts_with("{/*") {
                self.skip_jsx_comment();
                continue;
            }
            if self.peek() == Some('{') {
                children.push(self.parse_expression());
                continue;
            }
            if self.looks_like_jsx()
                && let Some(element) = self.parse_element()
            {
                children.push(JsxNode::Element(element));
                continue;
            }
            let text_offset = self.pos;
            let mut text = String::new();
            while !self.is_eof() && !self.starts_with("</") && self.peek() != Some('{') && !self.looks_like_jsx()
            {
                text.push(self.bump().unwrap());
            }
            if !text.is_empty() {
                children.push(JsxNode::Text {
                    offset: text_offset,
                    value: text,
                });
            } else if !self.is_eof() {
                self.bump();
            }
        }
        children
    }

    fn parse_attributes(&mut self) -> Option<(Vec<JsxAttribute>, bool, bool)> {
        let mut attributes = Vec::new();
        let mut spread = false;
        loop {
            self.skip_ws();
            if self.starts_with("/>") {
                self.pos += 2;
                return Some((attributes, spread, true));
            }
            if self.peek() == Some('>') {
                self.bump();
                return Some((attributes, spread, false));
            }
            if self.peek() == Some('{') {
                let expr = self.take_balanced('{', '}');
                if expr.trim_start().starts_with("...") {
                    spread = true;
                }
                continue;
            }
            let offset = self.pos;
            let name = self.parse_name()?;
            self.skip_ws();
            let value = if self.peek() == Some('=') {
                self.bump();
                self.skip_ws();
                match self.peek() {
                    Some('"') | Some('\'') => {
                        let quote = self.peek().unwrap();
                        AttrValue::String(self.take_quoted(quote))
                    }
                    Some('{') => {
                        let expr = self.take_balanced('{', '}');
                        match static_string_from_expr(&expr) {
                            Some(value) => AttrValue::String(value),
                            None => AttrValue::Expression(expr),
                        }
                    }
                    _ => AttrValue::BooleanTrue,
                }
            } else {
                AttrValue::BooleanTrue
            };
            attributes.push(JsxAttribute { offset, name, value });
        }
    }

    fn parse_expression(&mut self) -> JsxNode {
        let offset = self.pos;
        let raw = self.take_balanced('{', '}');
        let nested = parse_jsx(&raw).roots;
        JsxNode::Expression { offset, raw, nested }
    }

    fn parse_name(&mut self) -> Option<String> {
        if !self.peek().is_some_and(is_name_start) {
            return None;
        }
        let start = self.pos;
        self.pos += name_len(self.rest());
        Some(self.src[start..self.pos].to_string())
    }

    fn take_quoted(&mut self, quote: char) -> String {
        self.bump();
        let start = self.pos;
        while !self.is_eof() {
            let ch = self.bump().unwrap();
            if ch == '\\' {
                self.bump();
                continue;
            }
            if ch == quote {
                return self.src[start..self.pos - quote.len_utf8()].to_string();
            }
        }
        self.src[start..self.pos].to_string()
    }

    fn take_balanced(&mut self, open: char, close: char) -> String {
        if self.peek() != Some(open) {
            return String::new();
        }
        self.bump();
        let start = self.pos;
        let mut depth = 1usize;
        while !self.is_eof() && depth > 0 {
            if self.starts_with("//") {
                self.skip_line_comment();
                continue;
            }
            if self.starts_with("/*") {
                self.skip_block_comment();
                continue;
            }
            match self.peek() {
                Some('"' | '\'') => {
                    self.skip_quoted(self.peek().unwrap());
                }
                Some('`') => self.skip_template(),
                Some(ch) if ch == open => {
                    depth += 1;
                    self.bump();
                }
                Some(ch) if ch == close => {
                    depth -= 1;
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
        let end = if depth == 0 {
            self.pos - close.len_utf8()
        } else {
            self.pos
        };
        self.src[start..end].to_string()
    }

    fn skip_quoted(&mut self, quote: char) {
        let _ = self.take_quoted(quote);
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.bump() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) {
        self.pos += 2.min(self.src.len() - self.pos);
        while !self.is_eof() && !self.starts_with("*/") {
            self.bump();
        }
        if self.starts_with("*/") {
            self.pos += 2;
        }
    }

    fn skip_jsx_comment(&mut self) {
        if !self.starts_with("{/*") {
            return;
        }
        self.pos += 3;
        while !self.is_eof() && !self.starts_with("*/}") {
            self.bump();
        }
        if self.starts_with("*/}") {
            self.pos += 3;
        }
    }

    fn skip_template(&mut self) {
        if self.bump() != Some('`') {
            return;
        }
        while !self.is_eof() {
            match self.bump() {
                Some('\\') => {
                    self.bump();
                }
                Some('`') => return,
                Some('$') if self.peek() == Some('{') => {
                    let _ = self.take_balanced('{', '}');
                }
                _ => {}
            }
        }
    }
}

fn is_name_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_name_part(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '-' | ':' | '.')
}

fn is_attr_start(ch: char) -> bool {
    is_name_start(ch) || ch == ':'
}

fn name_len(input: &str) -> usize {
    input
        .char_indices()
        .take_while(|(idx, ch)| {
            if *idx == 0 {
                is_name_start(*ch)
            } else {
                is_name_part(*ch)
            }
        })
        .map(|(idx, ch)| idx + ch.len_utf8())
        .last()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{AttrValue, parse_jsx};

    #[test]
    fn parses_nested_elements_and_text() {
        let doc = parse_jsx(r#"export function A() { return <button>Save <b>now</b></button>; }"#);
        let elements = doc.elements();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0].name, "button");
        assert_eq!(elements[1].name, "b");
    }

    #[test]
    fn parses_quoted_and_expression_attributes() {
        let doc = parse_jsx(r#"<img alt="Cat" id={'hero'} hidden />"#);
        let img = doc.elements()[0];
        assert_eq!(img.static_attr("alt").as_deref(), Some("Cat"));
        assert_eq!(img.static_attr("id").as_deref(), Some("hero"));
        assert!(img.has_attr("hidden"));
        assert!(matches!(img.attr("hidden").unwrap().value, AttrValue::BooleanTrue));
    }

    #[test]
    fn skips_jsx_comments_and_finds_ids() {
        let doc = parse_jsx(
            r#"
            <section>
              {/* <button id="nope">x</button> */}
              <span id="save-lbl">Save</span>
            </section>
            "#,
        );
        assert!(doc.find_by_id("nope").is_none());
        assert!(doc.find_by_id("save-lbl").is_some());
    }

    #[test]
    fn tracks_spread_and_fragments() {
        let doc = parse_jsx(r#"<> <button {...props}>Go</button> </>"#);
        let button = doc
            .elements()
            .into_iter()
            .find(|el| el.name == "button")
            .expect("button");
        assert!(button.spread);
    }

    #[test]
    fn parses_jsx_nested_in_expressions() {
        let doc = parse_jsx(r#"<button>{ok && <span id="inner">Save</span>}</button>"#);
        assert!(doc.find_by_id("inner").is_some());
    }
}
