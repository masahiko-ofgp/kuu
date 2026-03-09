["fn" "let" "match" "if" "else" "pub" "use" "struct" "impl" "enum" "type"] @keyword
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
