use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, AnalyzerError, Confidence, Evidence, Finding,
    Location, Severity,
};

use crate::{has_syntax_errors, parse};

#[derive(Debug, Default, Clone, Copy)]
pub struct MutableDefaultArgumentAnalyzer;

impl Analyzer for MutableDefaultArgumentAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "python.mutable-default-argument",
            "Python mutable default argument analyzer",
            "0.1.0",
        )
    }

    fn analyze(&self, context: &AnalysisContext) -> Result<Vec<Finding>, AnalyzerError> {
        let mut findings = Vec::new();

        for file in context.files() {
            if file.language.as_deref() != Some("python") {
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
                if node.kind() == "default_parameter" {
                    let is_mutable_literal = node
                        .child_by_field_name("value")
                        .is_some_and(|value| matches!(value.kind(), "list" | "dictionary" | "set"));

                    if is_mutable_literal {
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
                                "python.mutable-default-argument:{}:{}:{}",
                                file.path,
                                start.row + 1,
                                start.column + 1
                            ),
                            rule_id: "python.mutable-default-argument".into(),
                            category: "bug".into(),
                            severity: Severity::High,
                            confidence,
                            message: "Mutable default argument is shared across all calls that don't override it.".into(),
                            explanation: Some(
                                "Python evaluates default argument values once, at function definition time. \
                                 Use None as the default and create the list/dict/set inside the function body instead."
                                    .into(),
                            ),
                            location: Some(location.clone()),
                            related_locations: Vec::new(),
                            evidence: vec![Evidence {
                                kind: "ast-default-parameter".into(),
                                message: Some(
                                    "The Python AST contains a function parameter whose default value is a list, dict, or set literal."
                                        .into(),
                                ),
                                locations: vec![location],
                            }],
                            analyzer: self.descriptor().id,
                            tags: vec!["python".into(), "bug".into()],
                        });
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
    use super::MutableDefaultArgumentAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, SourceFile};

    fn analyze(content: &str) -> Vec<codevanta_engine::Finding> {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/app.py".into(),
            language: Some("python".into()),
            content: content.into(),
        }]);

        MutableDefaultArgumentAnalyzer
            .analyze(&context)
            .expect("analysis should succeed")
    }

    #[test]
    fn flags_list_dict_and_set_default_arguments() {
        let findings = analyze("def f(items=[], data={}, tags={1, 2}):\n    pass\n");
        assert_eq!(findings.len(), 3);
    }

    #[test]
    fn ignores_immutable_default_arguments() {
        let findings = analyze("def f(count=1, name='x', flag=None):\n    pass\n");
        assert!(findings.is_empty());
    }
}
