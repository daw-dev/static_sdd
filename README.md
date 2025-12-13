# Static SDD

A compile-time in-place parser generator written in rust.

## Usage

To specify this crate as a dependency on your project simply run `cargo add --git https://github.com/daw-dev/static_sdd` or add the follwing to your `Cargo.toml`:

```toml
[dependency]
static_sdd = { git = "https://github.com/daw-dev/static_sdd" }
```

Then, anywhere in your project:

```rust
use static_sdd::*;

#[grammar]
mod addition {
    use super::*;

    #[non_terminal]
    #[start_symbol]
    pub type E = usize;

    #[token = r"\d+"]
    pub type Num = usize;

    #[token = "+"]
    pub struct Plus;

    production!(P0, E -> (E, Plus, Num), |(e, _, num)| e + num);

    production!(P1, E -> Num);
}

fn main() {
    let res = addition::parse("10+3+9");
    assert_eq!(res, Ok(22));
}
```

## Tool Comparison

What follows is a small comparison with tools that are in different ways similar this one:

### Rust Parser Generators

| Feature                | This Tool                                                  | LALRPOP                              | grmtools (lrpar)                          | Pomelo                           |
|------------------------|------------------------------------------------------------|--------------------------------------|-------------------------------------------|----------------------------------|
| Philosophy             | Use rust type system and module system to define a grammar | Rust version of bison                | Bison-compatible parser generator in rust | Rust version of lemon            |
| Algorithm              | LALR(1)                                                    | LALR(1)/LR(1)                        | LR(1)/GLR                                 | LALR(1) (lemon)                  |
| Execution Time         | Compile time (proc macro attribute)                        | Compile Time (build.rs)              | Compile Time (build.rs)                   | Compile Time (proc macro)        |
| Lexing                 | Internal (custom implementation or logos.rs)               | Hybrid (either internal or external) | External (lrlex)                          | External (expects Token enum)    |
| Synthesized Attributes | Yes (return types)                                         | Yes (return types)                   | Yes                                       | Yes (types)                      |
| Inherited Attributes   | Yes (helper types and compiler context)                    | No                                   | No                                        | No (%extra_args)                 |
| Zero-Copy              | Yes                                                        | Limited                              | Limited                                   | No                               |
| Error recovery         | Expressive errors and suggestions                          | !token / Recovery                    | Advanced (CPCT+)                          | No (panic!)                      |
| Grammar Definition     | Attributes inside a normal rust module, production! macro  | .lalrpop file with custom syntax     | .y file with Yacc syntax (mostly)         | pomelo! macro with custom syntax |
| IDE Support            | Works with rust-analyzer                                   | Custom LSP                           | Yacc LSP                                  | Very limited                     |

