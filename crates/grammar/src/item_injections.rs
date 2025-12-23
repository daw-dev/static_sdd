use dyn_grammar::{EnrichedGrammar, non_terminal::EnrichedNonTerminal, token::EnrichedToken};
use syn::{Ident, Item, parse_quote};

pub fn inject_items(internal_mod_name: Option<Ident>, items: &mut Vec<Item>, grammar: EnrichedGrammar) {
    items.push(compiler_context(grammar.context()));

    let mut items_to_add = Vec::new();
    items_to_add.extend(token_enum(grammar.tokens()));
    items_to_add.push(non_terminal_enum(grammar.non_terminals()));
    items_to_add.push(symbol_enum());
    match internal_mod_name {
        Some(name) => {
            items.push(parse_quote! {
                #[doc("generated using the static_sdd library")]
                pub mod #name {
                    use super::*;

                    #(#items_to_add)*
                }
            })
        }
        None => {
            items.extend(items_to_add);
        }
    }
    // items.push(parse_one_fn());
    // items.push(lex_fn());
    // items.push(parse_fn(todo!()));
    // items.push(parse_str_fn(todo!()));
}

fn compiler_context(compiler_ctx: &Option<Ident>) -> Item {
    compiler_ctx
        .as_ref()
        .map(|ctx| {
            parse_quote! {
                type __CompilerContext = #ctx;
            }
        })
        .unwrap_or(parse_quote! {
            type __CompilerContext = ();
        })
}

fn token_enum(tokens: &Vec<EnrichedToken>) -> Vec<Item> {
    let tokens: Vec<_> = tokens
        .iter()
        .map(|token| token.ident())
        .collect();
    let file: syn::File = parse_quote! {
        #[doc("Enum that contains every token")]
        pub enum Token {
            #(#tokens (#tokens),)*
        }

        impl std::fmt::Display for Token {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(Self::#tokens (_) => write!(f, stringify!(#tokens)),)*
                }
            }
        }
    };
    file.items
}

fn non_terminal_enum(non_terminals: &Vec<EnrichedNonTerminal>) -> Item {
    let non_terminals = non_terminals
        .iter()
        .map(|non_terminal| non_terminal.ident());
    parse_quote! {
        pub enum NonTerminal {
            #(#non_terminals (#non_terminals),)*
        }
    }
}

fn symbol_enum() -> Item {
    parse_quote! {
        pub enum Symbol {
            Token(Token),
            NonTerminal(NonTerminal),
        }
    }
}

fn parse_one_fn() -> Item {
    parse_quote! {
        pub fn parse_one(ctx: &mut __CompilerContext, stack: &mut Stack, curr: Token) {

        }
    }
}

fn parse_fn(start_symbol: Ident) -> Item {
    parse_quote! {
        pub fn parse(ctx: __CompilerContext, token_stream: impl IntoIterator<Token>) -> #start_symbol {
            todo!()
        }
    }
}

fn lex_fn() -> Item {
    parse_quote! {
        pub fn lex(word: impl Into<String>) -> Lex {
            todo!()
        }
    }
}

fn parse_str_fn(start_symbol: Ident) -> Item {
    parse_quote! {
        pub fn parse_str(ctx: __CompilerContext, word: impl Into<String>) -> #start_symbol {
            parse(ctx, lex(word))
        }
    }
}
