use anyhow::{Context, Result};
use axl_config::AxlConfig;
use axl_core::Diagnostic;
use axl_parser::{collect_lint_targets, load_and_parse_files};
use axl_rules::lint_file;
use clap::{Parser, ValueEnum};
use std::collections::BTreeMap;

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = AxlConfig::load(cli.config.as_deref())?;
    let files = collect_lint_targets(&cli.paths, &config.ignore)?;
    let parsed_files = load_and_parse_files(&files).context("Failed to parse lint targets")?;
    let only = if cli.only.is_empty() {
        None
    } else {
        Some(cli.only.as_slice())
    };

    let mut diagnostics = Vec::new();
    for parsed in &parsed_files {
        diagnostics.extend(lint_file(&parsed, only).diagnostics);
    }

    match cli.format {
        OutputFormat::Text => print_text(&diagnostics, files.len()),
        OutputFormat::Json => print_json(&diagnostics)?,
    }

    Ok(())
}

fn print_json(diagnostics: &[Diagnostic]) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(diagnostics)?);
    Ok(())
}

fn print_text(diagnostics: &[Diagnostic], files_checked: usize) {
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
                "  {}:{}  {:?}  {}  {}",
                item.line, item.column, item.severity, item.message, item.rule_id
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
}
