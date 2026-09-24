mod analyzers;
mod parser;
mod sql_analyzer;

pub use analyzers::ConsoleLogAnalyzer;
pub use parser::{has_syntax_errors, parse, Dialect};
pub use sql_analyzer::RawSqlTemplateAnalyzer;

use codevanta_engine::{
    AnalysisContext, DetectionConfidence, EcosystemContext, Language, LanguageDescriptor,
    TechnologyContext, TechnologyEvidence,
};

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
    /// Enriches an engine analysis context with Node.js ecosystem metadata.
    ///
    /// The caller supplies normalized technology identifiers and optional
    /// manifest evidence so the core engine does not depend on package-manager
    /// specific types.
    pub fn set_ecosystem_context(
        &self,
        context: &mut AnalysisContext,
        technologies: impl IntoIterator<Item = TechnologyContext>,
    ) {
        context.set_ecosystem(EcosystemContext {
            runtime: Some("nodejs".into()),
            technologies: technologies.into_iter().collect(),
        });
    }

    /// Converts a manifest dependency into engine-native technology evidence.
    pub fn manifest_technology(
        id: impl Into<String>,
        package_name: impl Into<String>,
    ) -> TechnologyContext {
        TechnologyContext {
            id: id.into(),
            confidence: DetectionConfidence::Definite,
            evidence: vec![TechnologyEvidence {
                kind: "manifest".into(),
                source: "package.json".into(),
                detail: format!("dependency: {}", package_name.into()),
            }],
        }
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
    fn attaches_ecosystem_context_with_provenance() {
        let language = JavaScriptTypeScript;
        let mut context = AnalysisContext::new(Vec::new());

        language.set_ecosystem_context(
            &mut context,
            [
                JavaScriptTypeScript::manifest_technology("nestjs", "@nestjs/core"),
                JavaScriptTypeScript::manifest_technology("typeorm", "typeorm"),
            ],
        );

        let ecosystem = context.ecosystem().expect("ecosystem should be set");
        assert_eq!(ecosystem.runtime.as_deref(), Some("nodejs"));
        assert_eq!(ecosystem.technologies.len(), 2);
        assert_eq!(ecosystem.technologies[1].id, "typeorm");
        assert_eq!(ecosystem.technologies[1].evidence[0].source, "package.json");
    }
}
