# Kuu editor

The name Kuu originates from the Buddhist term "空".

Kuu editor is still under development.

## Tree-sitter

- Rust only

`$ cargo build`

- OCaml, Python

`$ cargo build --features lang-ocaml`
`$ cargo build --features lang-python`

- All lang (Now Rust, Ocaml, Python only)

`$ cargo build --features all-langs`

## Features

- Directory Tree
- Two Key bindings (Vim, Emacs) can be selected.
    You can set key bindings in config.toml. Or input `chkey` command in Command Mode.
