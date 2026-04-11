use serde::Serialize;

#[derive(Debug, Clone)]
pub struct LintFile {
    pub path: String,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Roles,
    Naming,
    Focus,
    Contrast,
    Patterns,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Roles => "roles",
            Self::Naming => "naming",
            Self::Focus => "focus",
            Self::Contrast => "contrast",
            Self::Patterns => "patterns",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub rule_id: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LintResult {
    pub diagnostics: Vec<Diagnostic>,
}

impl LintResult {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

impl Default for LintResult {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Rule {
    fn id(&self) -> &'static str;
    fn category(&self) -> Category;
    fn run(&self, file: &LintFile) -> Vec<Diagnostic>;
}

pub fn line_col_from_offset(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (idx, ch) in source.char_indices() {
        if idx >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::{Category, Diagnostic, LintResult, Severity, line_col_from_offset};

    #[test]
    fn category_string_values_match_expected() {
        assert_eq!(Category::Roles.as_str(), "roles");
        assert_eq!(Category::Naming.as_str(), "naming");
        assert_eq!(Category::Focus.as_str(), "focus");
        assert_eq!(Category::Contrast.as_str(), "contrast");
        assert_eq!(Category::Patterns.as_str(), "patterns");
    }

    #[test]
    fn lint_result_default_and_push_work() {
        let mut result = LintResult::default();
        result.push(Diagnostic {
            file: "x.tsx".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Warn,
            rule_id: "focus/tabindex-positive",
            message: "warn".to_string(),
        });
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn computes_line_and_column_from_offset() {
        let (line, col) = line_col_from_offset("a\nbc\n", 3);
        assert_eq!(line, 2);
        assert_eq!(col, 2);
    }
}
