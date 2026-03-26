["as" "async" "await" "break" "const" "continue" "default" "dyn" "else" "enum" "fn" "for" "gen" "if" "impl" "in" "let" "loop" "macro_rules!" "match" "mod" "pub" "raw" "ref" "return" "static" "struct" "trait" "type" "union" "unsafe" "use" "where" "while" "yield"] @keyword
(crate) @keyword
(use_list (self) @keyword)
(scoped_use_list (self) @keyword)
(scoped_identifier (self) @keyword)
(super) @keyword
(mutable_specifier) @keyword

(line_comment) @comment
(line_comment (doc_comment)) @comment.document
(block_comment) @comment
(block_comment (doc_comment)) @comment.document

(string_literal) @string
(char_literal) @string
(raw_string_literal) @string

(function_item (identifier) @function)
(function_signature_item (identifier) @function)

(type_identifier) @type
(primitive_type) @type

["(" ")" "{" "}" "[" "]" "<" ">"] @punctuation.bracket
["::" ":" "." "," ";"] @punctuation.delimiter

["&" "'" "*" ] @operator
