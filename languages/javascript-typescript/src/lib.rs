mod analyzers;
mod parser;
mod sql_analyzer;

pub use analyzers::ConsoleLogAnalyzer;
pub use parser::{has_syntax_errors, parse, Dialect};
pub use sql_analyzer::RawSqlTemplateAnalyzer;

use codevanta_engine::{Language, LanguageDescriptor};

#[derive(Debug, Default, Clone, Copy)]
pub struct JavaScriptTypeScript;

impl Language for JavaScriptTypeScript {
    fn descriptor(&self) -> LanguageDescriptor {
        LanguageDescriptor::new(
            "javascript-typescript",
            "JavaScript / TypeScript",
            vec![
                ".js".into(),
                ".jsx".into(),
                ".mjs".into(),
                ".cjs".into(),
                ".ts".into(),
                ".tsx".into(),
                ".mts".into(),
                ".cts".into(),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_javascript_and_typescript_files() {
        let language = JavaScriptTypeScript;
        assert!(language.detect("src/app.ts", "export const app = true;"));
        assert!(language.detect("src/app.js", "module.exports = {};"));
        assert!(language.detect("src/app.tsx", "export function App() {}"));
        assert!(!language.detect("src/app.py", "print('no')"));
    }
}
