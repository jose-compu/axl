use anyhow::{Context, Result, bail};
use axl_core::LintFile;
use glob::glob;
use ignore::WalkBuilder;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn parse_lint_file(path: &str, source: String) -> Result<LintFile> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::tsx());
    let ret = Parser::new(&allocator, &source, source_type).parse();

    if !ret.errors.is_empty() {
        bail!("Parse error in {path}: {:?}", ret.errors[0]);
    }

    Ok(LintFile {
        path: path.to_string(),
        source,
    })
}

pub fn collect_lint_targets(inputs: &[String], ignore_patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut out = HashSet::new();

    for input in inputs {
        if input.contains('*') || input.contains('?') || input.contains('[') {
            for entry in glob(input)? {
                let path = entry?;
                if is_lint_target(&path) && !is_ignored(&path, ignore_patterns) {
                    out.insert(path);
                }
            }
            continue;
        }

        let path = Path::new(input);
        if path.is_file() {
            if is_lint_target(path) && !is_ignored(path, ignore_patterns) {
                out.insert(path.to_path_buf());
            }
            continue;
        }

        if path.is_dir() {
            let mut walk = WalkBuilder::new(path);
            walk.hidden(false);
            for entry in walk.build() {
                let entry = match entry {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                if entry.file_type().is_some_and(|f| f.is_file()) {
                    let p = entry.into_path();
                    if is_lint_target(&p) && !is_ignored(&p, ignore_patterns) {
                        out.insert(p);
                    }
                }
            }
        }
    }

    let mut files = out.into_iter().collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

pub fn load_and_parse_files(paths: &[PathBuf]) -> Result<Vec<LintFile>> {
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let source =
            fs::read_to_string(path).with_context(|| format!("Failed reading {}", path.display()))?;
        out.push(parse_lint_file(&path.to_string_lossy(), source)?);
    }
    Ok(out)
}

fn is_lint_target(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("jsx") || ext.eq_ignore_ascii_case("tsx"))
}

fn is_ignored(path: &Path, ignore_patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy().replace('\\', "/");
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    ignore_patterns.iter().any(|pattern| {
        let normalized = pattern.replace('\\', "/");
        matches_ignore_pattern(&normalized, &path_str, file_name)
    })
}

fn matches_ignore_pattern(pattern: &str, path_str: &str, file_name: &str) -> bool {
    if let Ok(compiled) = glob::Pattern::new(pattern) {
        if compiled.matches(path_str) || compiled.matches(file_name) {
            return true;
        }
    }
    if let Some(suffix) = pattern.strip_prefix("**/") {
        if let Ok(compiled) = glob::Pattern::new(suffix) {
            return compiled.matches(path_str) || compiled.matches(file_name);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{collect_lint_targets, load_and_parse_files, parse_lint_file};
    use std::fs;

    #[test]
    fn parses_valid_tsx() {
        let source = "<button aria-label=\"Save\">Save</button>".to_string();
        let parsed = parse_lint_file("button.tsx", source);
        assert!(parsed.is_ok());
    }

    #[test]
    fn collects_only_jsx_tsx_files() {
        let base = std::env::temp_dir().join(format!(
            "axl_parser_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&base).expect("create test dir");
        fs::write(base.join("a.tsx"), "<div />").expect("write tsx");
        fs::write(base.join("b.jsx"), "<div />").expect("write jsx");
        fs::write(base.join("c.ts"), "const x = 1").expect("write ts");

        let input = vec![base.to_string_lossy().into_owned()];
        let files = collect_lint_targets(&input, &[]).expect("collect files");
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("a.tsx")));
        assert!(files.iter().any(|p| p.ends_with("b.jsx")));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn collects_uppercase_jsx_tsx_extensions() {
        let base = std::env::temp_dir().join(format!(
            "axl_parser_case_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&base).expect("create test dir");
        fs::write(base.join("Upper.TSX"), "<div />").expect("write tsx");
        fs::write(base.join("Upper.JSX"), "<div />").expect("write jsx");

        let input = vec![base.to_string_lossy().into_owned()];
        let files = collect_lint_targets(&input, &[]).expect("collect files");
        assert_eq!(files.len(), 2);

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn parse_fails_for_invalid_tsx() {
        let source = "<button>".to_string();
        let parsed = parse_lint_file("broken.tsx", source);
        assert!(parsed.is_err());
    }

    #[test]
    fn respects_ignore_patterns() {
        let base = std::env::temp_dir().join(format!(
            "axl_parser_ignore_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&base).expect("create test dir");
        fs::write(base.join("keep.tsx"), "<div />").expect("write keep");
        fs::create_dir_all(base.join("storybook")).expect("create storybook");
        fs::write(base.join("storybook").join("skip.tsx"), "<div />").expect("write skip");

        let input = vec![base.to_string_lossy().into_owned()];
        let files =
            collect_lint_targets(&input, &[format!("{}/storybook/*", base.to_string_lossy())]).expect("collect files");
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("keep.tsx"));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn respects_double_star_ignore_patterns() {
        let base = std::env::temp_dir().join(format!(
            "axl_parser_glob_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&base).expect("create test dir");
        fs::write(base.join("keep.tsx"), "<div />").expect("write keep");
        fs::write(base.join("skip.test.tsx"), "<div />").expect("write skip");

        let input = vec![base.to_string_lossy().into_owned()];
        let files = collect_lint_targets(&input, &["**/*.test.tsx".to_string()]).expect("collect files");
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("keep.tsx"));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_and_parses_multiple_files() {
        let base = std::env::temp_dir().join(format!(
            "axl_parser_load_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&base).expect("create test dir");
        let one = base.join("one.tsx");
        let two = base.join("two.jsx");
        fs::write(&one, "<div role=\"widget\" />").expect("write one");
        fs::write(&two, "<button aria-label=\"ok\" />").expect("write two");

        let files = load_and_parse_files(&[one.clone(), two.clone()]).expect("load and parse");
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, one.to_string_lossy());
        assert_eq!(files[1].path, two.to_string_lossy());

        let _ = fs::remove_dir_all(base);
    }
}
