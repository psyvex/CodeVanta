mod local_model;

use std::{
    collections::HashMap,
    env, fs, io,
    path::{Path, PathBuf},
};

use codevanta_engine::{AnalysisContext, Engine, Finding, SourceFile};
use codevanta_javascript_typescript::{
    ConsoleLogAnalyzer, EmptyCatchAnalyzer, EvalUsageAnalyzer, LooseEqualityAnalyzer,
    SqlAnalyzer as JavaScriptSqlAnalyzer,
};
use codevanta_python::{
    BareExceptAnalyzer, DebugArtifactAnalyzer, EvalExecAnalyzer, MutableDefaultArgumentAnalyzer,
    SqlAnalyzer as PythonSqlAnalyzer,
};
use local_model::LocalModelClient;
use serde_json::to_string_pretty;

const IGNORED_DIRECTORIES: &[&str] = &[".git", "node_modules", "target", "dist", "coverage"];
const DEFAULT_LOCAL_MODEL_ENDPOINT: &str = "http://localhost:11434/v1/chat/completions";
const DEFAULT_LOCAL_MODEL: &str = "codevanta";
const MAX_REVIEW_CODE_CHARS: usize = 20_000;

const REVIEW_SYSTEM_PROMPT: &str = "You are a senior code reviewer. You are given a source \
file and a list of findings a deterministic static analyzer already produced for it. Write a \
short, prioritized, plain-language explanation: which findings matter most and why, and any \
context a developer would want before fixing them. Do not invent findings beyond what is \
listed, and do not repeat the raw JSON back verbatim.";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();

    match command.as_str() {
        "analyze" => run_analyze(args),
        "review" => run_review(args),
        _ => {
            eprintln!("usage: codevanta <analyze|review> <path> [--endpoint URL] [--model NAME]");
            Ok(())
        }
    }
}

fn run_analyze(args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    let (path, _flags) = parse_args(args)?;
    let findings = analyze_path(&path)?;
    println!("{}", to_string_pretty(&findings)?);
    Ok(())
}

/// Runs deterministic analysis, then asks a locally running OpenAI-compatible
/// chat model (Ollama, llama.cpp's llama-server, ...) to explain and
/// prioritize the findings per file. The local model is optional: if it
/// cannot be reached, the deterministic findings still print and a warning
/// is written to stderr per affected file rather than aborting.
fn run_review(args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    let (path, flags) = parse_args(args)?;
    let findings = analyze_path(&path)?;

    println!("{}", to_string_pretty(&findings)?);

    if findings.is_empty() {
        return Ok(());
    }

    let endpoint = flags
        .get("endpoint")
        .cloned()
        .or_else(|| env::var("CODEVANTA_LOCAL_MODEL_ENDPOINT").ok())
        .unwrap_or_else(|| DEFAULT_LOCAL_MODEL_ENDPOINT.to_string());
    let model = flags
        .get("model")
        .cloned()
        .or_else(|| env::var("CODEVANTA_LOCAL_MODEL").ok())
        .unwrap_or_else(|| DEFAULT_LOCAL_MODEL.to_string());
    let client = LocalModelClient::new(endpoint, model);

    for (file_path, file_findings) in group_findings_by_file(&findings) {
        let source = fs::read_to_string(&file_path).unwrap_or_default();
        let truncated_source: String = source.chars().take(MAX_REVIEW_CODE_CHARS).collect();
        let findings_json = to_string_pretty(&file_findings)?;
        let user_prompt = format!(
            "File: {file_path}\n\n```\n{truncated_source}\n```\n\nFindings:\n{findings_json}"
        );

        println!("\n--- local model review: {file_path} ---");
        match client.chat(REVIEW_SYSTEM_PROMPT, &user_prompt) {
            Ok(explanation) => println!("{explanation}"),
            Err(error) => {
                eprintln!("warning: local model review unavailable for {file_path}: {error}")
            }
        }
    }

    Ok(())
}

fn analyze_path(path: &Path) -> Result<Vec<Finding>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    collect_source_files(path, &mut files)?;

    let context = AnalysisContext::new(files);
    let engine = default_engine();
    Ok(engine.analyze(&context)?)
}

fn group_findings_by_file(findings: &[Finding]) -> Vec<(String, Vec<&Finding>)> {
    let mut by_file: HashMap<String, Vec<&Finding>> = HashMap::new();
    for finding in findings {
        if let Some(location) = &finding.location {
            by_file
                .entry(location.path.clone())
                .or_default()
                .push(finding);
        }
    }

    let mut grouped: Vec<(String, Vec<&Finding>)> = by_file.into_iter().collect();
    grouped.sort_by(|(a, _), (b, _)| a.cmp(b));
    grouped
}

fn parse_args(
    mut args: impl Iterator<Item = String>,
) -> Result<(PathBuf, HashMap<String, String>), Box<dyn std::error::Error>> {
    let path = PathBuf::from(args.next().ok_or("missing path")?);
    let mut flags = HashMap::new();

    while let Some(arg) = args.next() {
        let Some(name) = arg.strip_prefix("--") else {
            continue;
        };
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for --{name}"))?;
        flags.insert(name.to_string(), value);
    }

    Ok((path, flags))
}

fn default_engine() -> Engine {
    let mut engine = Engine::new();
    engine.add_analyzer(ConsoleLogAnalyzer);
    engine.add_analyzer(JavaScriptSqlAnalyzer);
    engine.add_analyzer(EmptyCatchAnalyzer);
    engine.add_analyzer(LooseEqualityAnalyzer);
    engine.add_analyzer(EvalUsageAnalyzer);
    engine.add_analyzer(PythonSqlAnalyzer);
    engine.add_analyzer(BareExceptAnalyzer);
    engine.add_analyzer(MutableDefaultArgumentAnalyzer);
    engine.add_analyzer(EvalExecAnalyzer);
    engine.add_analyzer(DebugArtifactAnalyzer);
    engine
}

/// Maps a file path to the language tag CodeVanta's analyzers key off of,
/// or None for a file no supported language claims.
fn detect_language(path: &str) -> Option<&'static str> {
    if path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".mts")
        || path.ends_with(".cts")
    {
        Some("typescript")
    } else if path.ends_with(".js")
        || path.ends_with(".jsx")
        || path.ends_with(".mjs")
        || path.ends_with(".cjs")
    {
        Some("javascript")
    } else if path.ends_with(".py") || path.ends_with(".pyi") {
        Some("python")
    } else {
        None
    }
}

fn collect_source_files(path: &Path, files: &mut Vec<SourceFile>) -> io::Result<()> {
    if path.is_file() {
        let source = fs::read_to_string(path)?;
        let display_path = path.to_string_lossy().to_string();
        if let Some(language_id) = detect_language(&display_path) {
            files.push(SourceFile {
                path: display_path,
                language: Some(language_id.into()),
                content: source,
            });
        }
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let entry_path = entry?.path();
        if entry_path.is_dir()
            && entry_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| IGNORED_DIRECTORIES.contains(&name))
        {
            continue;
        }

        collect_source_files(&entry_path, files)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use codevanta_engine::{Confidence, Severity};

    #[test]
    fn parse_args_reads_path_and_flags() {
        let args = [
            "src".to_string(),
            "--endpoint".to_string(),
            "http://x".to_string(),
            "--model".to_string(),
            "m".to_string(),
        ];
        let (path, flags) = parse_args(args.into_iter()).unwrap();

        assert_eq!(path, PathBuf::from("src"));
        assert_eq!(flags.get("endpoint").map(String::as_str), Some("http://x"));
        assert_eq!(flags.get("model").map(String::as_str), Some("m"));
    }

    #[test]
    fn parse_args_requires_a_path() {
        let result = parse_args(std::iter::empty());
        assert!(result.is_err());
    }

    #[test]
    fn parse_args_rejects_a_flag_missing_its_value() {
        let args = ["src".to_string(), "--endpoint".to_string()];
        let result = parse_args(args.into_iter());
        assert!(result.is_err());
    }

    fn finding(path: &str) -> Finding {
        let mut finding = Finding::new(
            "id",
            "rule",
            "category",
            Severity::Low,
            Confidence::Likely,
            "message",
            "analyzer",
        );
        finding.location = Some(codevanta_engine::Location {
            path: path.to_string(),
            start_line: 1,
            start_column: None,
            end_line: None,
            end_column: None,
        });
        finding
    }

    #[test]
    fn group_findings_by_file_groups_and_sorts_by_path() {
        let findings = vec![finding("b.ts"), finding("a.ts"), finding("b.ts")];

        let grouped = group_findings_by_file(&findings);

        let paths: Vec<&str> = grouped.iter().map(|(path, _)| path.as_str()).collect();
        assert_eq!(paths, vec!["a.ts", "b.ts"]);
        assert_eq!(grouped[1].1.len(), 2);
    }

    #[test]
    fn group_findings_by_file_skips_findings_without_a_location() {
        let findings = vec![
            finding("a.ts"),
            Finding::new(
                "id2",
                "rule",
                "category",
                Severity::Low,
                Confidence::Likely,
                "message",
                "analyzer",
            ),
        ];

        let grouped = group_findings_by_file(&findings);

        assert_eq!(grouped.len(), 1);
    }
}
