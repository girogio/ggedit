# ggedit

This is a vim-like text editor written in [Rust](https://www.rust-lang.org/). It's main purpose is for me to learn Rust. It's probably
not meant to be used as a real text editor, although I'm trying my hardest to make it usable.

## Features

- Vim modes
  - Normal mode
  - Insert mode
  - Command mode
- Vim-like keybindings
  - `hjkl` for cursor movement
  - `i` for insert mode
  - `:` for command mode
  - `esc` for normal mode
- Command mode
  - `:q` to quit
  - `:w` to save
  - `:wq` to save and quit
  - `:q!` to quit without saving

## Installation

```sh
$ cargo install --path .
```

## Usage

```sh
$ ggedit <filename>
```
