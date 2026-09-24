use std::path::Path;

use tree_sitter::{Language, Parser, Tree};
use tree_sitter_typescript::{LANGUAGE_TSX, LANGUAGE_TYPESCRIPT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    JavaScript,
    TypeScript,
    Tsx,
}

impl Dialect {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("jsx") | Some("tsx") => Some(Self::Tsx),
            Some("js") | Some("mjs") | Some("cjs") => Some(Self::JavaScript),
            Some("ts") | Some("mts") | Some("cts") => Some(Self::TypeScript),
            _ => None,
        }
    }
}

pub fn parse(path: &str, source: &str) -> Result<Tree, String> {
    let dialect = Dialect::from_path(Path::new(path))
        .ok_or_else(|| format!("unsupported JavaScript/TypeScript parser input: {path}"))?;

    let language: Language = match dialect {
        Dialect::JavaScript | Dialect::TypeScript => LANGUAGE_TYPESCRIPT.into(),
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
    use std::path::Path;

    use super::{Dialect, has_syntax_errors, parse};

    #[test]
    fn selects_grammar_by_extension() {
        assert_eq!(
            Dialect::from_path(Path::new("src/app.js")),
            Some(Dialect::JavaScript)
        );
        assert_eq!(
            Dialect::from_path(Path::new("src/app.tsx")),
            Some(Dialect::Tsx)
        );
        assert_eq!(
            Dialect::from_path(Path::new("src/app.ts")),
            Some(Dialect::TypeScript)
        );
    }

    #[test]
    fn parses_typescript_without_syntax_errors() {
        let tree = parse("src/app.ts", "const value: number = 1;").expect("should parse");
        assert!(!has_syntax_errors(&tree));
    }

    #[test]
    fn parses_tsx_with_jsx() {
        let tree =
            parse("src/app.tsx", "export const App = () => <main />;").expect("should parse");
        assert!(!has_syntax_errors(&tree));
    }
}
