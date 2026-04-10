# Kuu editor

The name Kuu originates from the Buddhist term "空".

## Tree-sitter

- Rust only

`$ cargo build`

- OCaml, Python

`$ cargo build --features lang-ocaml`
`$ cargo build --features lang-python`

- All lang

`$ cargo build --features all-langs`

## Features

- [x] Directory tree
- [] Two KeyBindings (You can select Vim or Emacs in config.toml)
- [] Tree-sitter (Rust, OCaml, Python.....)
