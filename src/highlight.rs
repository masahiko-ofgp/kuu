use tree_sitter::{
    Parser,
    Query,
    QueryCursor,
    StreamingIterator,
};
use tree_sitter_rust;
use ratatui::style::Color;


pub struct HighlightRange {
    pub start_byte: usize,
    pub end_byte: usize,
    pub color: Color,
}

pub struct Highlighter {
    parser: Parser,
    query: Query,
}

impl Highlighter {
    pub fn new() -> Self {
        let mut parser = Parser::new();

        let language = tree_sitter_rust::LANGUAGE.into();

        parser.set_language(&language)
            .expect("Error loading Rust grammer");

        let query_result = Query::new(&language, r#"
            ["use" "let" "fn" "if" "else" "pub" "struct" "enum" "impl" "type" "match"] @keyword
            (line_comment) @comment
            (string_literal) @string
            (struct_item (type_identifier) @type)
            (function_item (identifier) @function)
        "#);

        let query = match query_result {
            Ok(q) => q,
            Err(e) => {
                panic!("Query error at row {}: {:?}", e.row, e.message);
            }
        };

        Self { parser, query }
    }

    pub fn get_highlights(&mut self, text: &str) -> Vec<HighlightRange>
    {
        let tree = self.parser.parse(text, None).unwrap();

        let mut cursor = QueryCursor::new();

        let mut matches = cursor.matches(
            &self.query,
            tree.root_node(),
            text.as_bytes()
            );

        let mut highlights = Vec::new();

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let capture_name = self.query
                    .capture_names()[capture.index as usize];

                let color = match capture_name {
                    "keyword" => Color::Magenta,
                    "function" => Color::Blue,
                    "string" => Color::Green,
                    "comment" => Color::Gray,
                    "type" => Color::Yellow,
                    _ => Color::White,
                };

                highlights.push(HighlightRange {
                    start_byte: capture.node.start_byte(),
                    end_byte: capture.node.end_byte(),
                    color,
                });
            }
        }
        highlights
    }
}
