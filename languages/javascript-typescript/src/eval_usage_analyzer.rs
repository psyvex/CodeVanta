use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, AnalyzerError, Confidence, Evidence, Finding,
    Location, Severity,
};

use crate::{has_syntax_errors, parse};

#[derive(Debug, Default, Clone, Copy)]
pub struct EvalUsageAnalyzer;

impl Analyzer for EvalUsageAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "javascript.dangerous-eval",
            "JavaScript dynamic code execution analyzer",
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
                let flagged = match node.kind() {
                    "call_expression" => {
                        node.child_by_field_name("function")
                            .filter(|function| function.kind() == "identifier")
                            .and_then(|function| function.utf8_text(file.content.as_bytes()).ok())
                            == Some("eval")
                    }
                    "new_expression" => {
                        node.child_by_field_name("constructor")
                            .filter(|constructor| constructor.kind() == "identifier")
                            .and_then(|constructor| {
                                constructor.utf8_text(file.content.as_bytes()).ok()
                            })
                            == Some("Function")
                    }
                    _ => false,
                };

                if flagged {
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
                            "javascript.dangerous-eval:{}:{}:{}",
                            file.path,
                            start.row + 1,
                            start.column + 1
                        ),
                        rule_id: "javascript.dangerous-eval".into(),
                        category: "security".into(),
                        severity: Severity::High,
                        confidence,
                        message: "Dynamic code execution via eval or the Function constructor can lead to code injection.".into(),
                        explanation: Some(
                            "Avoid constructing and executing code from strings, especially from untrusted input."
                                .into(),
                        ),
                        location: Some(location.clone()),
                        related_locations: Vec::new(),
                        evidence: vec![Evidence {
                            kind: "ast-call-expression".into(),
                            message: Some(
                                "The TypeScript/JavaScript AST contains a call to eval or the Function constructor."
                                    .into(),
                            ),
                            locations: vec![location],
                        }],
                        analyzer: self.descriptor().id,
                        tags: vec!["javascript".into(), "typescript".into(), "security".into()],
                    });
                }

                nodes.extend(node.named_children(&mut cursor));
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::EvalUsageAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, SourceFile};

    fn analyze(content: &str) -> Vec<codevanta_engine::Finding> {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/app.ts".into(),
            language: Some("typescript".into()),
            content: content.into(),
        }]);

        EvalUsageAnalyzer
            .analyze(&context)
            .expect("analysis should succeed")
    }

    #[test]
    fn flags_eval_call() {
        let findings = analyze("eval(userInput);\n");
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn flags_function_constructor() {
        let findings = analyze("const f = new Function('a', 'return a');\n");
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn ignores_unrelated_calls() {
        let findings = analyze("evaluate(userInput);\nnew Functional();\n");
        assert!(findings.is_empty());
    }
}
