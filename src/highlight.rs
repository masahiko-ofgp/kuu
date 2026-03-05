use tree_sitter::{
    Parser,
    Query,
    QueryCursor,
    StreamingIterator,
    Language,
};
use tree_sitter_rust;
use ratatui::style::Color;


pub struct LanguageConfig {
    pub name: String,
    pub language: Language,
    pub query: Query,
}

impl LanguageConfig {
    pub fn new(name: &str, lang: Language, query_str: &str) -> Self {
        let query = Query::new(&lang, query_str)
            .expect(&format!("Error loading query for {}", name));
        Self {
            name: name.to_string(),
            language: lang,
            query,
        }
    }
}

pub struct HighlightRange {
    pub start_byte: usize,
    pub end_byte: usize,
    pub color: Color,
}

pub struct Highlighter {
    parser: Parser,
    current_config: Option<LanguageConfig>,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            current_config: None,
        }
    }

    pub fn set_language_by_extension(&mut self, extension: &str) {
        let config = match extension {
            #[cfg(feature = "lang-rust")]
            "rs" => {
                let lang = tree_sitter_rust::LANGUAGE.into();
                let query = r#"
                    ["use" "let" "fn" "if" "else" "pub" "struct" "enum" "impl" "type" "match"] @keyword
                    (line_comment) @comment
                    (string_literal) @string
                    (struct_item (type_identifier) @type)
                    (function_item (identifier) @function)
                    "#;
                Some(LanguageConfig::new("rust", lang, query))
            }
            #[cfg(feature = "lang-python")]
            "py" => {
            }
            #[cfg(feature = "lang-ocaml")]
            "ml" => {
            }
            _ => None,
        };
        if let Some(cfg) = config {
            self.parser.set_language(&cfg.language).ok();
            self.current_config = Some(cfg);
        } else {
            self.current_config = None;
        }
    }

    pub fn get_highlights(&mut self, text: &str) -> Vec<HighlightRange>
    {
        let config = match &self.current_config {
            Some(c) => c,
            None => return Vec::new(),
        };

        let tree = self.parser.parse(text, None).unwrap();

        let mut cursor = QueryCursor::new();

        let mut matches = cursor.matches(
            &config.query,
            tree.root_node(),
            text.as_bytes()
            );

        let mut highlights = Vec::new();

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let capture_name = config.query
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

    pub fn current_language_name(&self) -> Option<&str> {
        self.current_config.as_ref()
            .map(|c| c.name.as_str())
    }
}
