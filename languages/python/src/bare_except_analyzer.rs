use codevanta_engine::{
    AnalysisContext, Analyzer, AnalyzerDescriptor, AnalyzerError, Confidence, Evidence, Finding,
    Location, Severity,
};

use crate::{has_syntax_errors, parse};

#[derive(Debug, Default, Clone, Copy)]
pub struct BareExceptAnalyzer;

impl Analyzer for BareExceptAnalyzer {
    fn descriptor(&self) -> AnalyzerDescriptor {
        AnalyzerDescriptor::new(
            "python.bare-except",
            "Python bare except clause analyzer",
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
                // A bare `except:` has no exception-type node between the
                // `except` keyword and `:`, so its only named child is the
                // handler's block.
                let is_bare = node.kind() == "except_clause" && node.named_child_count() == 1;

                if is_bare {
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
                            "python.bare-except:{}:{}:{}",
                            file.path,
                            start.row + 1,
                            start.column + 1
                        ),
                        rule_id: "python.bare-except".into(),
                        category: "error-handling".into(),
                        severity: Severity::Medium,
                        confidence,
                        message: "Bare `except:` catches BaseException, including SystemExit and KeyboardInterrupt.".into(),
                        explanation: Some(
                            "Catch a specific exception type, or `except Exception:` if any application error should be handled, so signals like Ctrl-C and SystemExit still propagate."
                                .into(),
                        ),
                        location: Some(location.clone()),
                        related_locations: Vec::new(),
                        evidence: vec![Evidence {
                            kind: "ast-except-clause".into(),
                            message: Some(
                                "The Python AST contains an except clause with no exception type."
                                    .into(),
                            ),
                            locations: vec![location],
                        }],
                        analyzer: self.descriptor().id,
                        tags: vec!["python".into(), "error-handling".into()],
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
    use super::BareExceptAnalyzer;
    use codevanta_engine::{AnalysisContext, Analyzer, SourceFile};

    fn analyze(content: &str) -> Vec<codevanta_engine::Finding> {
        let context = AnalysisContext::new(vec![SourceFile {
            path: "src/app.py".into(),
            language: Some("python".into()),
            content: content.into(),
        }]);

        BareExceptAnalyzer
            .analyze(&context)
            .expect("analysis should succeed")
    }

    #[test]
    fn flags_bare_except() {
        let findings = analyze("try:\n    do_work()\nexcept:\n    pass\n");
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn ignores_typed_except_clauses() {
        let findings = analyze(
            "try:\n    do_work()\nexcept ValueError:\n    pass\nexcept Exception as e:\n    log(e)\n",
        );
        assert!(findings.is_empty());
    }
}
