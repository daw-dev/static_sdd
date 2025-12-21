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

## Features

### Basic features

Like every other parser generators, this tool implements:

- LALR(1) parsing table generation
- Lexing for parsing of strings
- Conflict warnings and resolution (precedence, associativity)
- Synthesization of attributes bottom-up during parsing
- Everything is done at compilation time

### Code as grammar philosophy

The main feature that separates this tool from other parser generators is how the grammar is defined: this tools
defines a grammar through the rust programming language: a grammar is a module, symbols are types (structs, enums,
type aliases or use directives) with either `#[non_terminal]` or `#[token = r"regex"]` and productions are created
using the `production!(Ident, Head -> Body, <semantic action>)` macro, but in practice they're `impl`s of the `Production`
for trait the newly created type at the first parameter of the production.

The semantic action is meant to look like a closure and it can either have 1 parameter, in that case the parameter is
the body of the production, or it can have 2 parameters, in that case the first is a mutable reference of the compiler context
(see more later) and the second one is the body of the production. Note, since in most cases the body will be a touple,
it can be useful to create a touple to capture the different values of the body:
`production!(P0, A -> (B, C, D), |ctx, (b, c, d)| todo!("synthesize A"))`

### Inherited attributes

The tool provides two ways to represent inherited attributes and they should be enough to cover most cases where
your parsing need inheritance.

#### Compilation context

Every grammar can have a compilation context, this is useful when dealing with information that is in different parts of
the parse tree (like scoping). Each `production!` instance has a mutable reference to an instance of the compilation context.
If no context is provided, `()` is used.

```rust
#[grammar]
mod my_grammar {
    #[context]
    pub struct MyCompilationContext {
        scope_id: usize,
    }

    #[non_terminal]
    #[start_symbol]
    pub struct StartSymbol;

    production!(P0, StartSymbol -> (), |ctx, _| {
        println!("scope is {}", ctx.scope_id);
        ctx.scope_id += 1;
    });
}
```

#### `FromInherited` helper type

Since inherited attributes are usually used to synthesize other attributes, another way to represent
inherited attributes is through functions, to make such representation easier, this tool provides the
`FromInherited<Inh, Syn>` helper type.

```rust
#[grammar]
mod arrays {
    pub enum ComputedType {
        BaseType(String),
        Array(usize, Box<ComputedType>),
    }

    #[non_terminal]
    #[start_symbol]
    pub type T = ComputedType;

    #[non_terminal]
    pub type Computed = FromInherited<String, ComputedType>;
    // the base type inherited attribute is only used to synthesize the computed type attribute

    #[token = "int|float"]
    pub type Base = String;

    #[token = "["]
    pub struct LeftSquarePar;

    #[token = "]"]
    pub struct RightSquarePar;

    #[token = r"\d+"]
    pub type Size = usize;

    production!(P1, T -> (Base, Computed), |(b, c)| c.resolve(b));
    // at this point the FromInherited is resolved: the inherited attribute is sent down the tree
    // and the synthesized one bounces up

    production!(P2, Computed -> (LeftSquarePar, Size, RightSquarePar, Computed), |(_, size, _, c)|
        c.map(move |val| ComputedType::Array(size, Box::new(val)))
        // at each bottom-up step, the type is wrapped in an array
    );

    production!(P3, Computed -> (), |_| FromInherited::new(ComputedType::BaseType));
    // at the leaf the computed type is just the base type
}
```

### Zero-Copy

This tool utilizes Rust's ownership model to achieve a zero-copy parsing, every symbol (token or internal non-terminal)
will be passed as owned value at each reduction of the parsing so that nothing will ever be copied.

### Future Features

While all the features above are natively supported in the current version of the tool, the following are features that
will be added in the future.

#### Expressive Errors, Suggestions and Recovery

Whenever parsing can't be done, the `parse` function (or any of its variants) should return an expressive error

```rust
// given the grammar for the arithmetic expression with addition, multiplication and parethesis
let res = arithmetic::parse("1*5+*3");
match res {
    Ok(res) => println!("result is {res}"),
    Err(err) => eprintln!("{err}"),
    // `1*5+*3`         found unexpected token Times, expected tokens are Num or OpenPar
    //      ^
}
```

Furthermore, the error should contain the stack that is used for the parsing so that if errors are fixed the parsing
can resume.

#### Grammar Modularity

Instead of using just one grammar module, a grammar can be split into multiple modules to better separate the different
parts of complex languages: for example, when building simple programming language that contains arithmetic expressions
and flow-control structures (such as if and while), one could create the arithmetic sub-grammar and the flow-control
sub-grammar.

```rust
#[grammar]
mod daw_lang {
    #[grammar]
    mod arithmetic {
        #[start_symbol]
        #[non_terminal]
        pub struct ArithmeticStatement;
    }

    #[grammar]
    mod flow_control {
        #[context]
        pub struct CurrentLabels {
            // ...
        }

        #[start_symbol]
        #[non_terminal]
        pub struct FlowControlStatement;
    }

    #[start_symbol]
    #[non_terminal]
    pub struct Statement;

    production!(P0, Statement -> ArithmeticStatement);
    production!(P1, Statement -> FlowControlStatement);
}
```

## Tool Comparison

What follows is a small comparison with tools that are in different ways similar this one:

### Rust Parser Generators

The following are rust parser generators - same category as this tool - so they all have some features in common:

- bottom up parsing
- semantic actions are called during parsing

| Feature                | This Tool                                                  | LALRPOP                              | grmtools (lrpar)                          | Pomelo                           |
|------------------------|------------------------------------------------------------|--------------------------------------|-------------------------------------------|----------------------------------|
| Philosophy             | Use rust type system and module system to define a grammar | Rust version of bison                | Bison-compatible parser generator in rust | Rust version of lemon            |
| Algorithm              | LALR(1)                                                    | LALR(1)/LR(1)                        | LR(1)/GLR                                 | LALR(1) (lemon)                  |
| Execution time         | Compile time (proc macro attribute)                        | Compile Time (build.rs)              | Compile Time (build.rs)                   | Compile Time (proc macro)        |
| Lexing                 | Internal (custom implementation or logos.rs)               | Internal (basic) or External         | External (lrlex)                          | External (expects Token enum)    |
| Synthesized Attributes | Yes (return types)                                         | Yes (return types)                   | Yes                                       | Yes (types)                      |
| Inherited Attributes   | Yes (helper types and compiler context)                    | No                                   | No                                        | No (%extra_args)                 |
| Zero-Copy              | Yes                                                        | Limited                              | Limited                                   | No                               |
| Error recovery         | Expressive errors and suggestions                          | !token / Recovery                    | Advanced (CPCT+)                          | No (panic!)                      |
| Grammar Definition     | Attributes inside a normal rust module, production! macro  | .lalrpop file with custom syntax     | .y file with Yacc syntax (mostly)         | pomelo! macro with custom syntax |
| IDE Support            | Works with rust-analyzer                                   | Custom LSP                           | Yacc LSP                                  | Very limited                     |

### Foreign Parser Generators

The following are also parser generators, but they have a different target language, the features will be similar to the ones above

| Feature                | Bison                            | ANTLR4                                | Menhir                               |
|------------------------|----------------------------------|---------------------------------------|--------------------------------------|
| Target Language        | C/C++                            | C++/C#/Java/js/PHP/Python/Swift/TS/GO | OCaml                                |
| Algorithm              | LALR(1)/GLR                      | Adaptive LL(*)                        | LR(1)                                |
| Execution time         | Ahead of time (generates C code) | Ahead of time (generates code)        | Ahead of time (generates OCaml code) |
| Lexing                 | External (flex)                  | Internal                              | External                             |
| Synthesized Attributes | Yes ($$)                         | Yes                                   | Yes                                  |
| Inherited Attributes   | Yes (through mid-rule actions)   | Yes (discouraged)                     | Not really (parameterized non-terminals)|
| Zero-Copy              | No                               | No                                    | No                                   |

### Alternative Approaches (Non-LALR)

These tools use different parsing philosophies compared to bottom-up LR/LALR generators. They are often preferred for binary formats or when a separate grammar file is undesirable.

| Feature | Parser Combinators (nom, chumsky) | PEG Generators (pest) | Tree-sitter |
| ---- | ---- | ---- | ---- |
| Category | Parser Combinators | PEG Parser Generator | Incremental GLR Parser |
| Philosophy | Grammar is defined as executable Rust functions | Grammar defined in external `.pest` file | Error-resilient parsing designed for IDEs |
| Algorithm | Recursive Descent (LL) | Packrat / PEG (Top-down) | GLR (Generalized LR) |
| Execution | Runtime (Function composition) | Compile time (Generates recursive descent) | Runtime (C runtime with Rust bindings) |
| Lexing | Integrated (Byte/Char stream) | Integrated (Regex-like) | Integrated |
| Zero-Copy | Yes (First-class citizen) | Yes | No (creates concrete syntax tree) |
| Ambiguity | Manual resolution (`alt` / `try`) | Prioritized Choice (`/` operator) | Handles ambiguity automatically (GLR) |
| Best For | Binary formats, network protocols, small DSLs | Config files, simple markup languages | Syntax highlighting, code analysis, fuzzy parsing |

### Summary and takeaways

#### Inherited Attributes
While most parser generators support synthesized attributes (bottom-up data flow), they often struggle with
inherited attributes (top-down context). Users are typically forced to rely on global mutable state, complex
workarounds or multiple tree traversal passes to handle scoping and context.

This tool natively supports both a native compilation context and a `FromInherited` helper type to enable attribute passing context
both bottom-up (through the return type of the semantic actions) and top-down.

This makes it uniquely suited for constructing complex compilers, type checkers, and semantic analyzers in Rust.

#### Grammar Definition

Existing tools generally fall into two categories, each with trade-offs:

- **Parser generators** (such as Bison or LALRPOP)
    - ❎ external file with custom syntax
    - ✅ similar syntax to the one used in literature
    - ✅ formal verification and conflict resolution
    - ❎ limited IDE support
- **Parser combinators** (such as nom or chumsky)
    - ✅ code as grammar directly in the host programming language
    - ❎ syntax is different to the usual one
    - ❎ no verification is possible
    - ✅ great IDE support

This tool aims to close the gap between the two philosophies managing to let you define your grammar directly in Rust, with
minimal custom syntax that tries to make the literature syntax Rust-like (`production!` macro) preserving the IDE support
(rust-analyzer) while still benefitting from the formal verification and conflict resolution of a parser generator.

#### Performance and Zero-Copy policy

Even though other tools have this policy as well, this tool manages to exploit Rust's ownership model to achieve Zero-Copy
natively and simply. No Clone is required unless the user specifically needs it.
