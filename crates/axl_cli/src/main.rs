use anyhow::{Context, Result};
use axl_config::AxlConfig;
use axl_core::Diagnostic;
use axl_parser::{collect_lint_targets, load_and_parse_files};
use axl_rules::{autofix_file_source, lint_file};
use clap::{Parser, ValueEnum};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Parser)]
#[command(name = "axl")]
#[command(about = "Accessibility linter for JSX/TSX")]
struct Cli {
    #[arg(default_value = ".")]
    paths: Vec<String>,
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
    #[arg(long, value_delimiter = ',')]
    only: Vec<String>,
    #[arg(long)]
    config: Option<String>,
    #[arg(long)]
    fix: bool,
    #[arg(long)]
    fix_dry_run: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(has_errors) if has_errors => ExitCode::from(1),
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<bool> {
    let cli = Cli::parse();
    let config = AxlConfig::load(cli.config.as_deref())?;
    let files = collect_lint_targets(&cli.paths, &config.ignore)?;
    let write_fixes = cli.fix && !cli.fix_dry_run;
    let fix_summary = if cli.fix || cli.fix_dry_run {
        Some(apply_fixes(&files, write_fixes)?)
    } else {
        None
    };
    let parsed_files = load_and_parse_files(&files).context("Failed to parse lint targets")?;
    let only = if cli.only.is_empty() {
        None
    } else {
        Some(cli.only.as_slice())
    };

    let mut diagnostics = Vec::new();
    for parsed in &parsed_files {
        diagnostics.extend(lint_file(parsed, only).diagnostics);
    }

    match cli.format {
        OutputFormat::Text => print_text(&diagnostics, files.len(), fix_summary, cli.fix_dry_run),
        OutputFormat::Json => print_json(&diagnostics)?,
    }

    Ok(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == axl_core::Severity::Error))
}

fn print_json(diagnostics: &[Diagnostic]) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(diagnostics)?);
    Ok(())
}

fn print_text(
    diagnostics: &[Diagnostic],
    files_checked: usize,
    fix_summary: Option<FixSummary>,
    dry_run: bool,
) {
    let mut by_file: BTreeMap<&str, Vec<&Diagnostic>> = BTreeMap::new();
    let mut errors = 0usize;
    let mut warnings = 0usize;

    for diagnostic in diagnostics {
        if diagnostic.severity == axl_core::Severity::Error {
            errors += 1;
        } else {
            warnings += 1;
        }
        by_file.entry(&diagnostic.file).or_default().push(diagnostic);
    }

    for (file, values) in by_file {
        println!("{file}");
        for item in values {
            println!(
                "  {}:{}  {}  {}  {}",
                item.line,
                item.column,
                item.severity.as_str(),
                item.message,
                item.rule_id
            );
        }
        println!();
    }

    println!(
        "{files_checked} files checked. {} problems ({} errors, {} warnings).",
        diagnostics.len(),
        errors,
        warnings
    );

    if let Some(summary) = fix_summary {
        let verb = if dry_run { "would apply" } else { "applied" };
        println!(
            "Autofix {verb} {} changes across {} files.",
            summary.fixes_applied, summary.files_touched
        );
    }
}

#[derive(Clone, Copy)]
struct FixSummary {
    fixes_applied: usize,
    files_touched: usize,
}

fn apply_fixes(files: &[PathBuf], write: bool) -> Result<FixSummary> {
    let mut files_touched = 0usize;
    let mut fixes_applied = 0usize;

    for path in files {
        let source = fs::read_to_string(path)
            .with_context(|| format!("Failed reading {} for autofix", path.display()))?;
        let (fixed, applied) = autofix_file_source(&source);
        if applied > 0 {
            if write {
                fs::write(path, fixed)
                    .with_context(|| format!("Failed writing autofix output to {}", path.display()))?;
            }
            files_touched += 1;
            fixes_applied += applied;
        }
    }

    Ok(FixSummary {
        fixes_applied,
        files_touched,
    })
}
