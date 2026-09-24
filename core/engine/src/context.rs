use std::collections::BTreeMap;

/// A source file made available to an analyzer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub language: Option<String>,
    pub content: String,
}

/// Repository-scoped information shared by analyzers.
///
/// This deliberately starts small. Language-specific ASTs, dependency graphs,
/// Git metadata, and ecosystem metadata can be added as optional capabilities
/// without coupling the core to one programming language.
#[derive(Debug, Default)]
pub struct AnalysisContext {
    files: Vec<SourceFile>,
    metadata: BTreeMap<String, String>,
}

impl AnalysisContext {
    pub fn new(files: Vec<SourceFile>) -> Self {
        Self {
            files,
            metadata: BTreeMap::new(),
        }
    }

    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}
