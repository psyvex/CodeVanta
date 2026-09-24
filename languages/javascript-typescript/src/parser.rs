use std::path::Path;

use tree_sitter::{Language, Parser, Tree};
use tree_sitter_typescript::{LANGUAGE_TSX, LANGUAGE_TYPESCRIPT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    TypeScript,
    Tsx,
}

impl Dialect {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("tsx") | Some("jsx") => Some(Self::Tsx),
            Some("ts") | Some("mts") | Some("cts") => Some(Self::TypeScript),
            _ => None,
        }
    }
}

pub fn parse(path: &str, source: &str) -> Result<Tree, String> {
    let dialect = Dialect::from_path(Path::new(path)).ok_or_else(|| {
        format!("unsupported JavaScript/TypeScript parser input: {path}")
    })?;

    let language: Language = match dialect {
        Dialect::TypeScript => LANGUAGE_TYPESCRIPT.into(),
        Dialect::Tsx => LANGUAGE_TSX.into(),
    };

    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .map_err(|error| format!("failed to configure parser: {error}"))?;

    parser
        .parse(source, None)
        .ok_or_else(|| "parser returned no syntax tree".to_string())
}

pub fn has_syntax_errors(tree: &Tree) -> bool {
    tree.root_node().has_error()
}

#[cfg(test)]
mod tests {
    use super::{has_syntax_errors, parse, Dialect};

    #[test]
    fn selects_tsx_for_tsx_files() {
        assert_eq!(Dialect::from_path("src/app.tsx"), Some(Dialect::Tsx));
        assert_eq!(Dialect::from_path("src/app.ts"), Some(Dialect::TypeScript));
    }

    #[test]
    fn parses_typescript_without_syntax_errors() {
        let tree = parse("src/app.ts", "const value: number = 1;").expect("should parse");
        assert!(!has_syntax_errors(&tree));
    }

    #[test]
    fn parses_tsx_with_jsx() {
        let tree = parse("src/app.tsx", "export const App = () => <main />;").expect("should parse");
        assert!(!has_syntax_errors(&tree));
    }
}
