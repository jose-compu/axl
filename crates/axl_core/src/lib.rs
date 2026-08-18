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

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "roles" => Some(Self::Roles),
            "naming" => Some(Self::Naming),
            "focus" => Some(Self::Focus),
            "contrast" => Some(Self::Contrast),
            "patterns" => Some(Self::Patterns),
            _ => None,
        }
    }
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
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

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Warn)
            .count()
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
    let mut previous = None;
    for (idx, ch) in source.char_indices() {
        if idx >= offset {
            break;
        }
        match ch {
            '\n' if previous != Some('\r') => {
                line += 1;
                col = 1;
            }
            '\n' => {}
            '\r' => {
                line += 1;
                col = 1;
            }
            _ => col += 1,
        }
        previous = Some(ch);
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
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn computes_line_and_column_from_offset() {
        let (line, col) = line_col_from_offset("a\nbc\n", 3);
        assert_eq!(line, 2);
        assert_eq!(col, 2);
    }

    #[test]
    fn computes_line_and_column_for_crlf() {
        let (line, col) = line_col_from_offset("a\r\nbc", 4);
        assert_eq!(line, 2);
        assert_eq!(col, 2);
    }

    #[test]
    fn parses_category_case_insensitively() {
        assert_eq!(Category::parse("Roles"), Some(Category::Roles));
        assert_eq!(Category::parse("FOCUS"), Some(Category::Focus));
        assert_eq!(Category::parse("nope"), None);
    }

    #[test]
    fn severity_string_values_match_expected() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warn.as_str(), "warn");
    }
}
