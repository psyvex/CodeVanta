use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, AnalyzerError, Confidence, Evidence, Finding,
    Location, Severity,
};

use crate::{has_syntax_errors, parse};

#[derive(Debug, Default, Clone, Copy)]
pub struct EvalExecAnalyzer;

impl Analyzer for EvalExecAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "python.dangerous-eval",
            "Python dynamic code execution analyzer",
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
                let flagged = node.kind() == "call"
                    && node
                        .child_by_field_name("function")
                        .filter(|function| function.kind() == "identifier")
                        .and_then(|function| function.utf8_text(file.content.as_bytes()).ok())
                        .is_some_and(|name| matches!(name, "eval" | "exec"));

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
                            "python.dangerous-eval:{}:{}:{}",
                            file.path,
                            start.row + 1,
                            start.column + 1
                        ),
                        rule_id: "python.dangerous-eval".into(),
                        category: "security".into(),
                        severity: Severity::High,
                        confidence,
                        message: "Dynamic code execution via eval or exec can lead to code injection.".into(),
                        explanation: Some(
                            "Avoid evaluating or executing code built from strings, especially from untrusted input."
                                .into(),
                        ),
                        location: Some(location.clone()),
                        related_locations: Vec::new(),
                        evidence: vec![Evidence {
                            kind: "ast-call".into(),
                            message: Some(
                                "The Python AST contains a call to eval or exec.".into(),
                            ),
                            locations: vec![location],
                        }],
                        analyzer: self.descriptor().id,
                        tags: vec!["python".into(), "security".into()],
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
    use super::EvalExecAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, SourceFile};

    fn analyze(content: &str) -> Vec<codevanta_engine::Finding> {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/app.py".into(),
            language: Some("python".into()),
            content: content.into(),
        }]);

        EvalExecAnalyzer
            .analyze(&context)
            .expect("analysis should succeed")
    }

    #[test]
    fn flags_eval_and_exec_calls() {
        let findings = analyze("eval(user_input)\nexec(user_input)\n");
        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn ignores_unrelated_calls() {
        let findings = analyze("evaluate(user_input)\nexecute(user_input)\n");
        assert!(findings.is_empty());
    }
}
