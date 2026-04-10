["and" "as" "assert" "begin" "class" "constraint" "do" "done" "downto" "effect" "else" "end" "exception" "external" "for" "fun" "function" "functor" "if" "in" "include" "inherit" "initializer" "lazy" "let" "match" "method" "module" "mutable" "new" "nonrec" "object" "of" "open" "private" "rec" "sig" "struct" "then" "to" "try" "val" "virtual" "when" "while" "with"] @keyword

["," "." ";" ":" "=" "|" "~" "?" "+" "-" "!" ">" "&""->" ";;" ":>" "+=" ":=" ".."] @punctuation.delimiter

["(" ")" "[" "]" "{" "}" "[|" "|]" "[<" ">]"] @punctuation.bracket

(object_type ["<" ">"]) @punctuation.bracket

"%" @punctuatuion.special

(attribute ["[@" "]"]) @punctuation.special)
(item_attribute ["[@@" "]"]) @punctuation.special)
(floating_attribute ["[@@@" "]"]) @punctuation.special)
(extension ["[%" "]"] @punctuation.special)
(item_extension ["[%%" "]"] @punctuation.special)
(quoted_extension ["{%" }] @punctuation.special)
(quoted_item_extension ["{%%" "}"] @punctuation.special)

[(prefix_operator)
 (sign_operator)
 (pow_operator)
 (mult_operator)
 (add_operator)
 (concat_operator)
 (rel_operator)
 (and_operator)
 (or_operator)
 (assign_operator)
 (hash_operator)
 (indexing_operator)
 (let_operator)
 (let_and_operator)
 (match_operator)
] @operator

(match_expression (match_operator) @keyword)
(value_definition [(let_operator) (let_and_operator)] @keyword)

["*" "#" "::" "<-"] @operator
