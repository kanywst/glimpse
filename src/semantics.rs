use std::fmt;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};

pub struct SemanticAnalyzer {
    rust_parser: Parser,
    #[allow(dead_code)]
    ts_parser: Parser,
    rust_query: Query,
}

impl fmt::Debug for SemanticAnalyzer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SemanticAnalyzer")
            .field("rust_query", &self.rust_query)
            .finish_non_exhaustive()
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new() -> Self {
        let mut rust_parser = Parser::new();
        let rust_lang = Language::from(tree_sitter_rust::LANGUAGE);
        rust_parser
            .set_language(&rust_lang)
            .expect("Error loading Rust grammar");

        let mut ts_parser = Parser::new();
        let ts_lang = Language::from(tree_sitter_typescript::LANGUAGE_TYPESCRIPT);
        ts_parser
            .set_language(&ts_lang)
            .expect("Error loading TypeScript grammar");

        let rust_query = Query::new(
            &rust_lang,
            r"
            (function_item name: (identifier) @name) @func
            (struct_item name: (type_identifier) @name) @struct
            (impl_item type: (type_identifier) @name) @impl
            ",
        )
        .expect("Error compiling Rust query");

        Self {
            rust_parser,
            ts_parser,
            rust_query,
        }
    }

    pub fn analyze(&mut self, path: &str, content: &str) -> Vec<SymbolChange> {
        if std::path::Path::new(path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
        {
            self.analyze_rust(content)
        } else {
            vec![]
        }
    }

    fn analyze_rust(&mut self, content: &str) -> Vec<SymbolChange> {
        let tree = self
            .rust_parser
            .parse(content, None)
            .expect("Failed to parse content");
        let mut cursor = QueryCursor::new();

        let mut symbols = Vec::new();

        let mut matches = cursor.matches(&self.rust_query, tree.root_node(), content.as_bytes());

        while let Some(m) = matches.next() {
            let node = m.captures[0].node;
            let kind = match node.kind() {
                "function_item" => "fn",
                "struct_item" => "struct",
                "impl_item" => "impl",
                _ => "code",
            };

            let name_node = m
                .captures
                .iter()
                .find(|c| c.index == 1)
                .map_or(node, |c| c.node);
            let name = name_node
                .utf8_text(content.as_bytes())
                .unwrap_or("unknown")
                .to_string();

            symbols.push(SymbolChange {
                name,
                kind: kind.to_string(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
            });
        }
        symbols
    }
}

#[derive(Debug, Clone)]
pub struct SymbolChange {
    pub name: String,
    pub kind: String,
    pub start_line: usize,
    pub end_line: usize,
}
