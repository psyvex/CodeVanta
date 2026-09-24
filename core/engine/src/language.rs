use serde::{Deserialize, Serialize};

/// Stable identity and capabilities exposed by a language integration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageDescriptor {
    pub id: String,
    pub display_name: String,
    pub extensions: Vec<String>,
}

impl LanguageDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        extensions: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            extensions,
        }
    }
}

/// Language integration boundary. Implementations stay outside the core engine.
pub trait Language: Send + Sync {
    fn descriptor(&self) -> LanguageDescriptor;

    fn detect(&self, path: &str, _source: &str) -> bool {
        self.descriptor()
            .extensions
            .iter()
            .any(|extension| path.ends_with(extension))
    }
}

#[cfg(test)]
mod tests {
    use super::{Language, LanguageDescriptor};

    struct TestLanguage;

    impl Language for TestLanguage {
        fn descriptor(&self) -> LanguageDescriptor {
            LanguageDescriptor::new("test", "Test", vec![".test".into()])
        }
    }

    #[test]
    fn detects_supported_extension() {
        assert!(TestLanguage.detect("example.test", ""));
        assert!(!TestLanguage.detect("example.txt", ""));
    }
}
