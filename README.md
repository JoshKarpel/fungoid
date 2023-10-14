# fungoid

A Befunge interpreter and text-UI IDE written in Rust.

## Installation

`fungoid` can be installed using
[`cargo`, the Rust package manager](https://doc.rust-lang.org/cargo/):

```bash
cargo install fungoid
```

## Usage

Fungoid provides a CLI command `fungoid`.
Use the `--help` option to see what it can do:

```console
$ fungoid --help
A Befunge interpreter and text-UI IDE written in Rust

Usage: fungoid <COMMAND>

Commands:
  run       Run a program
  ide       Start the TUI IDE
  examples  Interact with the bundled example programs
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

For example:

```console
$ fungoid examples run eratosthenes --profile
Executed 4752 instructions in 213us 969ns (22,208,824 instructions/second)
2357111317192329313741434753596167717379
```
