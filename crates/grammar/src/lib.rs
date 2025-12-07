use dyn_grammar::{
    grammar::Grammar, non_terminal::NonTerminal, production::Production,
    slr::automaton::SlrAutomaton, token::Token,
};
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro_error::{
    emit_call_site_error, emit_call_site_warning, emit_error, proc_macro_error,
};
use quote::quote;
use syn::{
    Ident, Item, ItemEnum, ItemMod, ItemStruct, ItemType, Meta, Type, parse::Parse,
    parse_macro_input, spanned::Spanned,
};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn grammar(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut module = parse_macro_input!(item as ItemMod);
    let (_, items) = module
        .content
        .as_mut()
        .expect("grammar module must be inline (contain braces)");

    let mut tokens = Vec::new();
    let mut non_terminals = Vec::new();
    let mut productions = Vec::new();
    let mut start_symbol: Option<usize> = None;

    for item in items.iter_mut() {
        if let Some(token) = extract_token(item) {
            tokens.push(token);
        } else if let Some((non_terminal, is_start)) = extract_non_terminal(item) {
            if is_start {
                if let Some(cur_start) = start_symbol {
                    panic!(
                        "you can only declare one start symbol, found both {} and {}",
                        non_terminals[cur_start], non_terminal
                    );
                }
                start_symbol = Some(non_terminals.len());
            }
            non_terminals.push(non_terminal);
        } else if let Some(production) = extract_production(item) {
            productions.push(production);
        }
    }

    if non_terminals.is_empty() || tokens.is_empty() || productions.is_empty() {
        emit_call_site_error!(
            "every grammar has to have some non-terminals, tokens and productions. Found non-terminals: [{}], tokens: [{}], productions: [{}]",
            non_terminals.iter().format(","),
            tokens.iter().format(","),
            productions.iter().format(","),
        );
    }

    let start_symbol = match start_symbol {
        Some(start_symbol) => non_terminals[start_symbol].name().clone(),
        None => {
            emit_call_site_warning!("no start symbol was declared, using {}", non_terminals[0]);
            non_terminals[0].name().clone()
        }
    };

    let grammar = Grammar::new(non_terminals, tokens, productions, start_symbol);
    // eprintln!("{grammar:?}");
    let automaton = SlrAutomaton::compute(&grammar);
    automaton.display_table(&grammar);

    quote! {
        #module

        pub fn parse() {
            println!("Hello World!");
        }
    }
    .into()
}

fn extract_token(item: &mut Item) -> Option<Token> {
    match item {
        Item::Type(ItemType { attrs, ident, .. })
        | Item::Struct(ItemStruct { attrs, ident, .. })
        | Item::Enum(ItemEnum { attrs, ident, .. }) => {
            if let Some(id) = attrs.iter().enumerate().find_map(|(i, attr)| {
                if let Meta::NameValue(name_value) = &attr.meta
                    && name_value.path.is_ident("token")
                {
                    return Some(i);
                }
                None
            }) {
                // TODO: maybe `token` should be an actual attribute that automatically creates the
                // DFA?
                let attr = attrs.remove(id);
                let Meta::NameValue(name_value) = attr.meta else {
                    unreachable!()
                };
                let syn::Expr::Lit(lit_value) = name_value.value else {
                    emit_error!(
                        ident.span(),
                        "token attribute must define the corresponding regexpr, usage: #[token = \"\\d\"]"
                    );
                    panic!()
                };
                let syn::Lit::Str(lit_str_value) = lit_value.lit else {
                    emit_error!(
                        ident.span(),
                        "token regexpr must be a string literal, usage: #[token = \"\\d\"]"
                    );
                    panic!()
                };
                Some(Token::new(ident.to_string(), lit_str_value.value()))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn extract_non_terminal(item: &mut Item) -> Option<(NonTerminal, bool)> {
    match item {
        Item::Type(ItemType { attrs, ident, .. })
        | Item::Struct(ItemStruct { attrs, ident, .. })
        | Item::Enum(ItemEnum { attrs, ident, .. }) => {
            if let Some(id) = attrs.iter().enumerate().find_map(|(i, attr)| {
                if let Meta::Path(path) = &attr.meta
                    && path.is_ident("non_terminal")
                {
                    return Some(i);
                }
                None
            }) {
                attrs.remove(id);
                let mut is_start = false;
                if let Some(id) = attrs.iter().enumerate().find_map(|(i, attr)| {
                    if let Meta::Path(path) = &attr.meta
                        && path.is_ident("start_symbol")
                    {
                        return Some(i);
                    }
                    None
                }) {
                    attrs.remove(id);
                    is_start = true;
                }
                Some((NonTerminal::new(ident.to_string()), is_start))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn extract_production(item: &mut Item) -> Option<Production> {
    struct ProductionInternal {
        name: Ident,
        head: Ident,
        body: Type,
    }

    impl Parse for ProductionInternal {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let name = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            let head = input.parse()?;
            input.parse::<syn::Token![->]>()?;
            let body = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            input.parse::<syn::Expr>()?;
            Ok(ProductionInternal { name, head, body })
        }
    }

    impl From<ProductionInternal> for Production {
        fn from(value: ProductionInternal) -> Self {
            let name = value.name.to_string();
            let head = value.head.to_string();
            let body = match value.body {
                Type::Path(type_path) => vec![type_path.path.get_ident().unwrap().to_string()],
                Type::Tuple(type_tuple) => type_tuple
                    .elems
                    .iter()
                    .map(|t| {
                        let Type::Path(type_path) = t else {
                            panic!("body of production has to be a tuple of named types")
                        };
                        type_path.path.get_ident().unwrap().to_string()
                    })
                    .collect(),
                _ => panic!("type must be either a single type or a tuple"),
            };

            Production::new(name, head, body)
        }
    }

    match item {
        Item::Macro(mac) if mac.mac.path.is_ident("production") => {
            let t = mac.mac.parse_body::<ProductionInternal>();
            if let Ok(prod_internal) = t {
                Some(prod_internal.into())
            } else {
                None
            }
        }
        _ => None,
    }
}

macro_rules! dummy_attribute {
    ($attr:ident, $pos:expr) => {
        #[proc_macro_attribute]
        #[proc_macro_error]
        pub fn $attr(_attr: TokenStream, _item: TokenStream) -> TokenStream {
            panic!("this attribute has to be put on top of {}", $pos)
        }
    };
}

dummy_attribute!(token, "type aliases, structs or enums");
dummy_attribute!(start_symbol, "type aliases, structs or enums");
dummy_attribute!(non_terminal, "type aliases, structs or enums");
dummy_attribute!(left_associative, "production macros");
dummy_attribute!(right_associative, "production macros");
dummy_attribute!(precedence, "production marcos");
