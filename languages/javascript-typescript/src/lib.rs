mod analyzers;
mod parser;
mod sql_analyzer;

pub use analyzers::ConsoleLogAnalyzer;
pub use parser::{has_syntax_errors, parse, Dialect};
pub use sql_analyzer::RawSqlTemplateAnalyzer;

use codevanta_engine::{AnalysisContext, EcosystemContext, Language, LanguageDescriptor};

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

impl JavaScriptTypeScript {
    /// Enriches an engine analysis context with normalized Node.js ecosystem
    /// metadata detected from a package manifest.
    pub fn set_ecosystem_context(
        &self,
        context: &mut AnalysisContext,
        technologies: impl IntoIterator<Item = impl Into<String>>,
    ) {
        context.set_ecosystem(EcosystemContext {
            runtime: Some("nodejs".into()),
            technologies: technologies.into_iter().map(Into::into).collect(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codevanta_engine::AnalysisContext;

    #[test]
    fn detects_javascript_and_typescript_files() {
        let language = JavaScriptTypeScript;
        assert!(language.detect("src/app.ts", "export const app = true;"));
        assert!(language.detect("src/app.js", "module.exports = {};"));
        assert!(language.detect("src/app.tsx", "export function App() {}"));
        assert!(!language.detect("src/app.py", "print('no')"));
    }

    #[test]
    fn attaches_normalized_node_ecosystem_context() {
        let language = JavaScriptTypeScript;
        let mut context = AnalysisContext::new(Vec::new());

        language.set_ecosystem_context(&mut context, ["nestjs", "typeorm"]);

        let ecosystem = context.ecosystem().expect("ecosystem should be set");
        assert_eq!(ecosystem.runtime.as_deref(), Some("nodejs"));
        assert_eq!(ecosystem.technologies, ["nestjs", "typeorm"]);
    }
}
