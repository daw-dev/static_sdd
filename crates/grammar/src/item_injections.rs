use dyn_grammar::{
    EnrichedGrammar,
    lalr::LalrAutomaton,
    non_terminal::EnrichedNonTerminal,
    parsing::tables::{EofTable, NonTerminalTable, TokenTable},
    production::EnrichedProduction,
    symbolic_grammar::SymbolicGrammar,
    token::EnrichedToken,
};
use itertools::Itertools;
use proc_macro::Span;
use quote::quote;
use syn::{Ident, Item, parse_quote};

pub fn inject_items(
    internal_mod_name: Option<Ident>,
    items: &mut Vec<Item>,
    enriched_grammar: EnrichedGrammar,
) {
    eprintln!("{enriched_grammar}");
    let symbolic_grammar = SymbolicGrammar::from(&enriched_grammar);
    eprintln!("{symbolic_grammar}");
    let automaton = LalrAutomaton::compute(&symbolic_grammar);
    let states_count = automaton.states_count();
    eprintln!("{automaton}");
    let (token_table, eof_table, non_terminal_table) = automaton.generate_tables();
    eprintln!("{token_table}");
    eprintln!("{non_terminal_table}");

    items.push(compiler_context(enriched_grammar.context()));

    let mut items_to_add = Vec::new();
    items_to_add.extend(uses());
    items_to_add.extend(token_enum(enriched_grammar.tokens()));
    items_to_add.extend(non_terminal_enum(enriched_grammar.non_terminals(), enriched_grammar.start_symbol()));
    items_to_add.extend(production_enum(enriched_grammar.productions()));
    eprintln!("added enums");
    items_to_add.extend(const_tables(&enriched_grammar, states_count, token_table, eof_table, non_terminal_table));
    items_to_add.push(parser(enriched_grammar.start_symbol()));

    for item in items_to_add.iter() {
        println!("------------------------------");
        println!("{}", quote!(#item));
    }

    match internal_mod_name {
        Some(name) => items.push(parse_quote! {
                #[doc("generated using the static_sdd library")]
                pub mod #name {
                    use super::*;

                    #(#items_to_add)*
                }
            }),
        None => {
            items.extend(items_to_add);
        }
    }
}

fn compiler_context(compiler_ctx: Option<&Ident>) -> Item {
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

fn uses() -> Vec<Item> {
    let file: syn::File = parse_quote!{
        use logos::Logos;
        use parser::Symbol;
    };
    file.items
}

fn token_enum(tokens: &[EnrichedToken]) -> Vec<Item> {
    let variants = tokens.iter().map(|token| {
        let ident = token.ident();
        match token.match_string() {
            dyn_grammar::token::Match::Literal(lit) => quote! {
                #[token(#lit, |_| #ident)]
                #ident(#ident)
            },
            dyn_grammar::token::Match::Regex(regex) => quote! {
                #[regex(#regex, |lex| lex.slice().parse().ok())]
                #ident(#ident)
            }
        }
    });
    let tokens: Vec<_> = tokens.iter().map(|token| token.ident()).collect();
    let counter = 0usize..;
    let file: syn::File = parse_quote! {
        #[doc("Enum that contains every token")]
        #[derive(Logos)]
        #[logos(skip r"[ \t\n\f]+")]
        pub enum Token {
            #(#variants,)*
        }

        impl std::fmt::Display for Token {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(Self::#tokens (_) => write!(f, stringify!(#tokens)),)*
                }
            }
        }

        impl std::fmt::Debug for Token {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(Self::#tokens (_) => write!(f, stringify!(#tokens)),)*
                }
            }
        }

        impl Token {
            pub const fn id(&self) -> usize {
                match self {
                    #(Self::#tokens (_) => #counter,)*
                }
            }
        }
    };
    file.items
}

fn non_terminal_enum(non_terminals: &[EnrichedNonTerminal], start_symbol: &EnrichedNonTerminal) -> Vec<Item> {
    let start_symbol = start_symbol.ident();
    let non_terminals = non_terminals
        .iter()
        .map(|non_terminal| non_terminal.ident()).collect_vec();
    let counter = 0usize..;
    let file: syn::File = parse_quote! {
        pub enum NonTerminal {
            #(#non_terminals (#non_terminals),)*
        }

        impl std::fmt::Display for NonTerminal {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(Self::#non_terminals (_) => write!(f, stringify!(#non_terminals)),)*
                }
            }
        }

        impl std::fmt::Debug for NonTerminal {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(Self::#non_terminals (_) => write!(f, stringify!(#non_terminals)),)*
                }
            }
        }

        impl NonTerminal {
            pub const fn id(&self) -> usize {
                match self {
                    #(Self::#non_terminals (_) => #counter,)*
                }
            }
        }

        impl Into<#start_symbol> for NonTerminal {
            fn into(self) -> #start_symbol {
                match self {
                    Self::#start_symbol(val) => val,
                    _ => panic!(),
                }
            }
        }
    };

    file.items
}

fn production_enum(productions: &[EnrichedProduction]) -> Vec<Item> {
    let idents = productions
        .iter()
        .map(EnrichedProduction::ident)
        .collect_vec();
    let reductions = productions.iter().map(|prod| {
        let prod_name = prod.ident();
        let head_type = prod.head();
        let exprs = prod.body().iter().enumerate().map(|(i, sym)| {
            let var_name = Ident::new(&format!("t{i}"), Span::call_site().into());
            match sym {
                dyn_grammar::enriched_symbol::EnrichedSymbol::Token(enriched_token) => {
                    let type_ident = enriched_token.ident();
                    quote! {
                        let Some(Symbol::Token(Token::#type_ident(#var_name))) = stacks.symbol_stack.pop() else { unreachable!("this is not a token") };
                        stacks.state_stack.pop();
                    }
                }
                dyn_grammar::enriched_symbol::EnrichedSymbol::NonTerminal(enriched_non_terminal) => {
                    let type_ident = enriched_non_terminal.ident();
                    quote! {
                        let Some(Symbol::NonTerminal(NonTerminal::#type_ident(#var_name))) = stacks.symbol_stack.pop() else { unreachable!("this is not a non terminal") };
                        stacks.state_stack.pop();
                    }
                }
            }
        }).rev();
        let vars = (0usize..prod.arity()).map(|i| Ident::new(&format!("t{i}"), Span::call_site().into()));
        quote! {
            {
                #(#exprs)*
                let body = (#(#vars),*);
                
                NonTerminal::#head_type(#prod_name::synthesize(ctx, body))
            }
        }
    });
    let file: syn::File = parse_quote! {
        #[derive(Debug, Clone)]
        pub enum ProductionName {
            #(#idents,)*
        }

        impl std::fmt::Display for ProductionName {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(Self::#idents => write!(f, stringify!(#idents)),)*
                }
            }
        }

        impl parser::Reduce<NonTerminal, Token, __CompilerContext> for ProductionName {
            fn reduce(&self, ctx: &mut __CompilerContext, stacks: &mut parser::Stacks<NonTerminal, Token>) -> NonTerminal {
                match self {
                    #(Self::#idents => #reductions,)*
                }
            }
        }
    };
    file.items
}

fn const_tables(
    enriched_grammar: &EnrichedGrammar,
    state_count: usize,
    token_table: TokenTable,
    eof_table: EofTable,
    non_terminal_table: NonTerminalTable,
) -> Vec<Item> {
    let token_count = enriched_grammar.tokens().len();
    let non_terminal_count = enriched_grammar.non_terminals().len();

    let token_actions = token_table.table.into_iter().map::<syn::Expr, _>(|row| {
        let row = row.into_iter().map::<syn::Expr, _>(|action| {
            match action.map::<syn::Expr, _>(|action| match action {
                dyn_grammar::parsing::action::TokenAction::Shift(state) => {
                    parse_quote!(parser::TokenAction::Shift(#state))
                }
                dyn_grammar::parsing::action::TokenAction::Reduce(prod_id) => {
                    let actual_production = enriched_grammar
                        .productions()
                        .get(prod_id)
                        .expect("production not found");
                    let prod_name = actual_production.ident();
                    parse_quote!(parser::TokenAction::Reduce(ProductionName::#prod_name))
                }
            }) {
                Some(expr) => parse_quote!(Some(#expr)),
                None => parse_quote!(None),
            }
        });
        parse_quote! {
            [#(#row),*]
        }
    });

    let eof_actions = eof_table.table.into_iter().map::<syn::Expr, _>(|action| {
        match action.map::<syn::Expr, _>(|action| match action {
            dyn_grammar::parsing::action::EofAction::Reduce(prod_id) => {
                let actual_production = enriched_grammar
                    .productions()
                    .get(prod_id)
                    .expect("production not found");
                let prod_name = actual_production.ident();
                parse_quote!(parser::EofAction::Reduce(ProductionName::#prod_name))
            }
            dyn_grammar::parsing::action::EofAction::Accept => parse_quote!(parser::EofAction::Accept),
        }) {
                Some(expr) => parse_quote!(Some(#expr)),
                None => parse_quote!(None),
            }
        }
    );

    let gotos = non_terminal_table.table.into_iter().map::<syn::Expr, _>(|row| {
        let row = row.into_iter().map::<syn::Expr, _>(|state| {
            match state {
                Some(expr) => parse_quote!(Some(#expr)),
                None => parse_quote!(None),
            }
        });
        parse_quote! {
            [#(#row),*]
        }
    });

    let file: syn::File = parse_quote! {
        #[derive(Debug)]
        pub struct Tables;

        impl Tables {
            pub const TOKEN_TABLE: [[Option<parser::TokenAction<ProductionName>>; #token_count]; #state_count] = [
                #(#token_actions,)*
            ];
            
            pub const EOF_TABLE: [Option<parser::EofAction<ProductionName>>; #state_count] = [
                #(#eof_actions,)*
            ];

            pub const NON_TERMINAL_TABLE: [[Option<usize>; #non_terminal_count]; #state_count] = [
                #(#gotos,)*
            ];
        }

        impl parser::Tables<NonTerminal, Token, ProductionName> for Tables {
            fn query_token_table(current_state: usize, current_token: &Token) -> Option<parser::TokenAction<ProductionName>> {
                Tables::TOKEN_TABLE[current_state][current_token.id()].clone()
            }
            fn query_eof_table(current_state: usize) -> Option<parser::EofAction<ProductionName>> {
                Tables::EOF_TABLE[current_state].clone()
            }
            fn query_goto_table(current_state: usize, non_terminal: &NonTerminal) -> Option<usize> {
                Tables::NON_TERMINAL_TABLE[current_state][non_terminal.id()].clone()
            }
        }
    };
    file.items
}

fn match_table(
    enriched_grammar: &EnrichedGrammar,
    state_count: usize,
    token_table: TokenTable,
    eof_table: EofTable,
    non_terminal_table: NonTerminalTable,
) -> Vec<Item> {
    let file: syn::File = parse_quote!{
        #[derive(Debug)]
        pub struct Tables;

        impl parser::Tables<NonTerminal, Token, ProductionName> for Tables {
            fn query_token_table(current_state: usize, current_token: &Token) -> Option<parser::TokenAction<ProductionName>> {
                Tables::TOKEN_TABLE[current_state][current_token.id()].clone()
            }
            fn query_eof_table(current_state: usize) -> Option<parser::EofAction<ProductionName>> {
                Tables::EOF_TABLE[current_state].clone()
            }
            fn query_goto_table(current_state: usize, non_terminal: &NonTerminal) -> Option<usize> {
                Tables::NON_TERMINAL_TABLE[current_state][non_terminal.id()].clone()
            }
        }
    };
    file.items
}

fn parser(start_symbol: &EnrichedNonTerminal) -> Item {
    let start_symbol = start_symbol.ident();
    parse_quote!(pub type Parser = parser::Parser<NonTerminal, Token, #start_symbol, ProductionName, Tables, __CompilerContext>;)
}

