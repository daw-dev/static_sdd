use dyn_grammar::{
    EnrichedGrammar,
    lalr::LalrAutomaton,
    non_terminal::EnrichedNonTerminal,
    parsing::tables::{ActionTable, GoToTable},
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
    eprintln!("{automaton}");
    let (action_table, goto_table) = automaton.generate_tables();
    eprintln!("{action_table}");
    eprintln!("{goto_table}");

    items.push(compiler_context(enriched_grammar.context()));

    let mut items_to_add = Vec::new();
    items_to_add.push(logos_use());
    items_to_add.extend(token_enum(enriched_grammar.tokens()));
    items_to_add.extend(non_terminal_enum(enriched_grammar.non_terminals()));
    items_to_add.extend(symbol_enum());
    items_to_add.extend(production_enum(enriched_grammar.productions()));
    items_to_add.push(action_enum());
    items_to_add.extend(table_const(&enriched_grammar, action_table, goto_table));
    items_to_add.extend(stacks_struct());
    items_to_add.push(parse_one_result());
    items_to_add.push(parse_one_fn());
    items_to_add.push(parse_one_eof_result());
    items_to_add.push(parse_one_eof_fn(enriched_grammar.tokens().len()));
    items_to_add.push(parse_fn(enriched_grammar.start_symbol().ident()));
    items_to_add.push(parse_str_fn(enriched_grammar.start_symbol().ident()));
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

fn logos_use() -> Item {
    parse_quote!(use logos::Logos;)
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

fn non_terminal_enum(non_terminals: &[EnrichedNonTerminal]) -> Vec<Item> {
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

        impl NonTerminal {
            pub const fn id(&self) -> usize {
                match self {
                    #(Self::#non_terminals (_) => #counter,)*
                }
            }
        }
    };

    file.items
}

fn symbol_enum() -> Vec<Item> {
    let file: syn::File = parse_quote! {
        pub enum Symbol {
            Token(Token),
            NonTerminal(NonTerminal),
        }

        impl std::fmt::Debug for Symbol {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Token(tok) => write!(f, "{tok}"),
                    Self::NonTerminal(nt) => write!(f, "{nt}"),
                }
            }
        }

        impl std::fmt::Display for Symbol {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Token(tok) => write!(f, "{tok}"),
                    Self::NonTerminal(nt) => write!(f, "{nt}"),
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
                dyn_grammar::enriched_symbol::EnrichedSymbol::EOF => unreachable!(),
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
        #[derive(Debug)]
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

        impl ProductionName {
            fn reduce(&self, ctx: &mut __CompilerContext, stacks: &mut Stacks) -> NonTerminal {
                match self {
                    #(Self::#idents => #reductions,)*
                }
            }
        }
    };
    file.items
}

fn action_enum() -> Item {
    parse_quote! {
        #[derive(Debug)]
        pub enum Action {
            Shift(usize),
            Reduce(ProductionName),
            Accept,
        }
    }
}

fn table_const(
    enriched_grammar: &EnrichedGrammar,
    action_table: ActionTable,
    goto_table: GoToTable,
) -> Vec<Item> {
    let (atw, ath) = action_table.dimensions();
    let actions = action_table.table.into_iter().map::<syn::Expr, _>(|row| {
        let row = row.into_iter().map::<syn::Expr, _>(|action| {
            match action.map::<syn::Expr, _>(|action| match action {
                dyn_grammar::parsing::action::Action::Shift(state) => {
                    parse_quote!(Action::Shift(#state))
                }
                dyn_grammar::parsing::action::Action::Reduce(prod_id) => {
                    let actual_production = enriched_grammar
                        .productions()
                        .get(prod_id)
                        .expect("production not found");
                    let prod_name = actual_production.ident();
                    parse_quote!(Action::Reduce(ProductionName::#prod_name))
                }
                dyn_grammar::parsing::action::Action::Accept => parse_quote!(Action::Accept),
            }) {
                Some(expr) => parse_quote!(Some(#expr)),
                None => parse_quote!(None),
            }
        });
        parse_quote! {
            [#(#row),*]
        }
    });

    let (gtw, gth) = goto_table.dimensions();
    let gotos = goto_table.table.into_iter().map::<syn::Expr, _>(|row| {
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
        pub const ACTION_TABLE: [[Option<Action>; #atw]; #ath] = [
            #(#actions,)*
        ];

        pub const GOTO_TABLE: [[Option<usize>; #gtw]; #gth] = [
            #(#gotos,)*
        ];
    };
    file.items
}

fn stacks_struct() -> Vec<Item> {
    let file: syn::File = parse_quote! {
        #[derive(Debug)]
        pub struct Stacks {
            pub state_stack: Vec<usize>,
            pub symbol_stack: Vec<Symbol>,
        }

        impl std::fmt::Display for Stacks {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Stacks {{ ")?;
                write!(f, "state_stack: {:?}, ", self.state_stack)?;
                write!(f, "symbol_stack: {:?} ", self.symbol_stack)?;
                write!(f, "}}")
            }
        }

        impl Stacks {
            pub fn new() -> Self {
                Self {
                    state_stack: vec![0],
                    symbol_stack: Vec::new(),
                }
            }
        }
    };

    file.items
}

fn parse_one_result() -> Item {
    parse_quote! {
        pub enum ParseOneResult {
            Shifted,
            Reduced(Token),
            Error,
        }
    }
}

fn parse_one_fn() -> Item {
    parse_quote! {
        pub fn parse_one(ctx: &mut __CompilerContext, stacks: &mut Stacks, curr: Token) -> ParseOneResult {
            let current_state = stacks.state_stack.last();
            let Some(&current_state) = current_state else { return ParseOneResult::Error; };
            let token_id = curr.id();
            let action = &ACTION_TABLE[current_state][token_id];
            let Some(action) = action else { return ParseOneResult::Error; };
            match action {
                Action::Shift(state) => {
                    stacks.state_stack.push(*state);
                    stacks.symbol_stack.push(Symbol::Token(curr));
                    ParseOneResult::Shifted
                }
                Action::Reduce(prod_name) => {
                    let head = prod_name.reduce(ctx, stacks);
                    let current_state = *stacks.state_stack.last().unwrap();
                    let id = head.id();
                    let Some(new_state) = &GOTO_TABLE[current_state][id] else { panic!("couldn't parse") };
                    stacks.state_stack.push(*new_state);
                    stacks.symbol_stack.push(Symbol::NonTerminal(head));
                    ParseOneResult::Reduced(curr)
                }
                Accept => unreachable!(),
            }
        }
    }
}

fn parse_one_eof_result() -> Item {
    parse_quote! {
        pub enum ParseOneEofResult {
            Reduced,
            Accepted,
            Error,
        }
    }
}

fn parse_one_eof_fn(token_count: usize) -> Item {
    parse_quote! {
        pub fn parse_one_eof(ctx: &mut __CompilerContext, stacks: &mut Stacks) -> ParseOneEofResult {
            let current_state = stacks.state_stack.last();
            let Some(&current_state) = current_state else { return ParseOneEofResult::Error; };
            let id = #token_count;
            let action = &ACTION_TABLE[current_state][id];
            let Some(action) = action else { return ParseOneEofResult::Error; };
            match action {
                Action::Reduce(prod_name) => {
                    let head = prod_name.reduce(ctx, stacks);
                    let current_state = *stacks.state_stack.last().unwrap();
                    let id = head.id();
                    let Some(new_state) = &GOTO_TABLE[current_state][id] else { return ParseOneEofResult::Error; };
                    stacks.state_stack.push(*new_state);
                    stacks.symbol_stack.push(Symbol::NonTerminal(head));
                    ParseOneEofResult::Reduced
                }
                Action::Accept => {
                    ParseOneEofResult::Accepted
                }
                Action::Shift(_) => unreachable!(),
            }
        }
    }
}

fn parse_fn(start_symbol: &Ident) -> Item {
    parse_quote! {
        pub fn parse(mut ctx: __CompilerContext, token_stream: impl IntoIterator<Item = Token>) -> Result<#start_symbol, Stacks> {
            let mut stacks = Stacks::new();
            for mut token in token_stream.into_iter() {
                loop {
                    let parse_one_result = parse_one(&mut ctx, &mut stacks, token);
                    match parse_one_result {
                        ParseOneResult::Reduced(curr) => {
                            token = curr;
                        }
                        ParseOneResult::Error => {
                            return Err(stacks);
                        }
                        ParseOneResult::Shifted => {
                            break;
                        }
                    }
                }
            }
            while let ParseOneEofResult::Reduced = parse_one_eof(&mut ctx, &mut stacks) {}
            let Some(Symbol::NonTerminal(NonTerminal::#start_symbol (res))) = stacks.symbol_stack.pop() else { panic!() };
            Ok(res)
        }
    }
}

fn parse_str_fn(start_symbol: &Ident) -> Item {
    parse_quote! {
        pub fn parse_str(ctx: __CompilerContext, source: &<Token as Logos<'_>>::Source) -> Result<#start_symbol, Stacks> {
            parse(ctx, Token::lexer(source).flatten())
        }
    }
}
