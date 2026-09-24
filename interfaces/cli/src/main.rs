use std::{env, fs, io, path::{Path, PathBuf}};

use codevanta_engine::{AnalysisContext, Engine, SourceFile};
use codevanta_javascript_typescript::{ConsoleLogAnalyzer, JavaScriptTypeScript, RawSqlTemplateAnalyzer};
use serde_json::to_string_pretty;

const IGNORED_DIRECTORIES: &[&str] = &[".git", "node_modules", "target", "dist", "coverage"];

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().as_deref().unwrap_or("");

    if command != "analyze" {
        eprintln!("usage: codevanta analyze <path>");
        return Ok(());
    }

    let root = PathBuf::from(args.next().ok_or("missing path")?);
    let language = JavaScriptTypeScript;
    let mut files = Vec::new();
    collect_source_files(&root, &language, &mut files)?;

    let context = AnalysisContext::new(files);
    let engine = default_engine();
    let findings = engine.analyze(&context)?;

    println!("{}", to_string_pretty(&findings)?);
    Ok(())
}

fn default_engine() -> Engine {
    let mut engine = Engine::new();
    engine.add_analyzer(ConsoleLogAnalyzer);
    engine.add_analyzer(RawSqlTemplateAnalyzer);
    engine
}

fn collect_source_files(
    path: &Path,
    language: &JavaScriptTypeScript,
    files: &mut Vec<SourceFile>,
) -> io::Result<()> {
    if path.is_file() {
        let source = fs::read_to_string(path)?;
        let display_path = path.to_string_lossy().to_string();
        if language.detect(&display_path, &source) {
            let language_id = if display_path.ends_with(".ts") || display_path.ends_with(".tsx")
                || display_path.ends_with(".mts") || display_path.ends_with(".cts")
            {
                "typescript"
            } else {
                "javascript"
            };
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

        collect_source_files(&entry_path, language, files)?;
    }

    Ok(())
}
