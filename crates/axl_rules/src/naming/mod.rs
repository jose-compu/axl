pub mod button_missing_name;
pub mod empty_accessible_name;
pub mod img_missing_alt;
pub mod link_missing_name;
pub mod svg_missing_name;

use axl_aria::jsx::{JsxDocument, JsxElement, parse_jsx};
use axl_core::{Diagnostic, LintFile, Severity, line_col_from_offset};

pub(crate) fn parse_doc(file: &LintFile) -> JsxDocument {
    parse_jsx(&file.source)
}

pub(crate) fn element_diagnostic(
    file: &LintFile,
    element: &JsxElement,
    rule_id: &'static str,
    severity: Severity,
    message: String,
) -> Diagnostic {
    let (line, column) = line_col_from_offset(&file.source, element.offset);
    Diagnostic {
        file: file.path.clone(),
        line,
        column,
        severity,
        rule_id,
        message,
    }
}
