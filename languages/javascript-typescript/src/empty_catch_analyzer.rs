use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, AnalyzerError, Confidence, Evidence, Finding,
    Location, Severity,
};

use crate::{has_syntax_errors, parse};

#[derive(Debug, Default, Clone, Copy)]
pub struct EmptyCatchAnalyzer;

impl Analyzer for EmptyCatchAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "javascript.empty-catch-block",
            "JavaScript empty catch block analyzer",
            "0.1.0",
        )
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, AnalyzerError> {
        let mut findings = Vec::new();

        for file in context.files() {
            if !matches!(
                file.language.as_deref(),
                Some("javascript") | Some("typescript")
            ) {
                continue;
            }

            let tree =
                parse(&file.path, &file.content).map_err(|message| AnalyzerError::Failed {
                    analyzer: self.descriptor().id,
                    message,
                })?;

            let confidence = if has_syntax_errors(&tree) {
                Confidence::Possible
            } else {
                Confidence::Likely
            };

            let mut cursor = tree.walk();
            let mut nodes = vec![tree.root_node()];
            while let Some(node) = nodes.pop() {
                if node.kind() == "catch_clause" {
                    if let Some(body) = node.child_by_field_name("body") {
                        if body.kind() == "statement_block" && body.named_child_count() == 0 {
                            let start = node.start_position();
                            let end = node.end_position();
                            let location = Location {
                                path: file.path.clone(),
                                start_line: start.row as u32 + 1,
                                start_column: Some(start.column as u32 + 1),
                                end_line: Some(end.row as u32 + 1),
                                end_column: Some(end.column as u32 + 1),
                            };

                            findings.push(Finding {
                                id: format!(
                                    "javascript.empty-catch-block:{}:{}:{}",
                                    file.path,
                                    start.row + 1,
                                    start.column + 1
                                ),
                                rule_id: "javascript.empty-catch-block".into(),
                                category: "error-handling".into(),
                                severity: Severity::Medium,
                                confidence,
                                message: "Empty catch block silently swallows errors.".into(),
                                explanation: Some(
                                    "Handle, log, or rethrow the caught error so failures are not lost."
                                        .into(),
                                ),
                                location: Some(location.clone()),
                                related_locations: Vec::new(),
                                evidence: vec![Evidence {
                                    kind: "ast-catch-clause".into(),
                                    message: Some(
                                        "The TypeScript/JavaScript AST contains a catch clause whose body has no statements."
                                            .into(),
                                    ),
                                    locations: vec![location],
                                }],
                                analyzer: self.descriptor().id,
                                tags: vec!["javascript".into(), "typescript".into(), "error-handling".into()],
                            });
                        }
                    }
                }

                nodes.extend(node.named_children(&mut cursor));
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::EmptyCatchAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, SourceFile};

    fn analyze(content: &str) -> Vec<codevanta_engine::Finding> {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/app.ts".into(),
            language: Some("typescript".into()),
            content: content.into(),
        }]);

        EmptyCatchAnalyzer
            .analyze(&context)
            .expect("analysis should succeed")
    }

    #[test]
    fn flags_empty_catch_block() {
        let findings = analyze("try {\n  doWork();\n} catch (error) {}\n");
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn ignores_catch_block_that_handles_error() {
        let findings = analyze("try {\n  doWork();\n} catch (error) {\n  log(error);\n}\n");
        assert!(findings.is_empty());
    }
}
