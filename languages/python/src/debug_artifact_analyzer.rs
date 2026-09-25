use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, AnalyzerError, Confidence, Evidence, Finding,
    Location, Severity,
};
use tree_sitter::Node;

use crate::{has_syntax_errors, parse};

#[derive(Debug, Default, Clone, Copy)]
pub struct DebugArtifactAnalyzer;

impl Analyzer for DebugArtifactAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "python.debug-artifact",
            "Python debugger artifact analyzer",
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
                if node.kind() == "call" && is_debugger_call(node, file.content.as_bytes()) {
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
                            "python.debug-artifact:{}:{}:{}",
                            file.path,
                            start.row + 1,
                            start.column + 1
                        ),
                        rule_id: "python.debug-artifact".into(),
                        category: "maintainability".into(),
                        severity: Severity::Low,
                        confidence,
                        message: "Debugger breakpoint is present in application source.".into(),
                        explanation: Some(
                            "Remove breakpoint()/pdb.set_trace() calls before merging.".into(),
                        ),
                        location: Some(location.clone()),
                        related_locations: Vec::new(),
                        evidence: vec![Evidence {
                            kind: "ast-call".into(),
                            message: Some(
                                "The Python AST contains a breakpoint() or pdb.set_trace() call."
                                    .into(),
                            ),
                            locations: vec![location],
                        }],
                        analyzer: self.descriptor().id,
                        tags: vec!["python".into(), "maintainability".into()],
                    });
                }

                nodes.extend(node.named_children(&mut cursor));
            }
        }

        Ok(findings)
    }
}

fn is_debugger_call(call: Node, source: &[u8]) -> bool {
    let Some(function) = call.child_by_field_name("function") else {
        return false;
    };

    match function.kind() {
        "identifier" => function
            .utf8_text(source)
            .is_ok_and(|name| name == "breakpoint"),
        "attribute" => {
            let object_is_pdb = function
                .child_by_field_name("object")
                .and_then(|object| object.utf8_text(source).ok())
                == Some("pdb");
            let attribute_is_set_trace = function
                .child_by_field_name("attribute")
                .and_then(|attribute| attribute.utf8_text(source).ok())
                == Some("set_trace");
            object_is_pdb && attribute_is_set_trace
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::DebugArtifactAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, SourceFile};

    fn analyze(content: &str) -> Vec<codevanta_engine::Finding> {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/app.py".into(),
            language: Some("python".into()),
            content: content.into(),
        }]);

        DebugArtifactAnalyzer
            .analyze(&context)
            .expect("analysis should succeed")
    }

    #[test]
    fn flags_breakpoint_and_pdb_set_trace() {
        let findings = analyze("breakpoint()\nimport pdb\npdb.set_trace()\n");
        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn ignores_unrelated_calls() {
        let findings = analyze("logger.breakpoint()\npdb.post_mortem()\n");
        assert!(findings.is_empty());
    }
}
