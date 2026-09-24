use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A source file made available to an analyzer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub language: Option<String>,
    pub content: String,
}

/// Repository-scoped ecosystem information shared by analyzers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EcosystemContext {
    pub runtime: Option<String>,
    pub technologies: Vec<String>,
}

/// Repository-scoped information shared by analyzers.
///
/// This deliberately keeps ecosystem metadata structured but language-neutral.
/// Language packages can enrich it without coupling the core engine to a
/// particular framework or package manager.
#[derive(Debug, Default)]
pub struct AnalysisContext {
    files: Vec<SourceFile>,
    metadata: BTreeMap<String, String>,
    ecosystem: Option<EcosystemContext>,
}

impl AnalysisContext {
    pub fn new(files: Vec<SourceFile>) -> Self {
        Self {
            files,
            metadata: BTreeMap::new(),
            ecosystem: None,
        }
    }

    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    pub fn ecosystem(&self) -> Option<&EcosystemContext> {
        self.ecosystem.as_ref()
    }

    pub fn set_ecosystem(&mut self, ecosystem: EcosystemContext) {
        self.ecosystem = Some(ecosystem);
    }

    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}
