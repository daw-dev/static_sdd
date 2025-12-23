use dyn_grammar::{
    EnrichedGrammar, non_terminal::EnrichedNonTerminal, production::EnrichedProduction,
    token::EnrichedToken,
};
use itertools::Itertools;
use proc_macro_error::{emit_call_site_error, emit_call_site_warning, emit_error};
use syn::{
    Attribute, Ident, Item, ItemEnum, ItemStruct, ItemType, ItemUse, Meta, Type, UseGroup, UseTree,
    parse::Parse,
};

pub fn extract_grammar(items: &mut Vec<Item>) -> EnrichedGrammar {
    let mut tokens = Vec::new();
    let mut non_terminals = Vec::new();
    let mut productions = Vec::new();
    let mut start_symbol = None;
    let mut compiler_ctx = None;

    for item in items.iter_mut() {
        if let Some(ctx) = extract_context(item) {
            if compiler_ctx.is_some() {
                emit_error!(
                    ctx.span(),
                    "Compiler context defined for the second time here"
                );
                panic!();
            }
            compiler_ctx = Some(ctx);
        } else if let Some(token) = extract_token(item) {
            tokens.push(token);
        } else if let Some((non_terminal, is_start)) = extract_non_terminal(item) {
            if is_start {
                if let Some(cur_start) = start_symbol {
                    panic!(
                        "you can only declare one start symbol, found both {} and {}",
                        cur_start, non_terminal
                    );
                }
                start_symbol = Some(non_terminal.clone());
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

    let start_symbol = start_symbol.unwrap_or_else(|| {
        emit_call_site_warning!("no start symbol was declared, using {}", non_terminals[0]);
        non_terminals[0].clone()
    });

    EnrichedGrammar::new(
        compiler_ctx,
        non_terminals,
        tokens,
        productions,
        start_symbol,
    )
}

fn extract_ident_from_use_tree(tree: &mut UseTree) -> Option<Ident> {
    match tree {
        UseTree::Path(use_path) => extract_ident_from_use_tree(&mut use_path.tree),
        UseTree::Name(use_name) => Some(use_name.ident.clone()),
        UseTree::Rename(use_rename) => Some(use_rename.rename.clone()),
        UseTree::Group(UseGroup { items, .. }) if items.len() == 1 => {
            extract_ident_from_use_tree(items.pop().unwrap().value_mut())
        }
        _ => None,
    }
}

fn extract_info(item: &mut Item) -> Option<(&mut Vec<Attribute>, Ident)> {
    match item {
        Item::Type(ItemType { attrs, ident, .. })
        | Item::Struct(ItemStruct { attrs, ident, .. })
        | Item::Enum(ItemEnum { attrs, ident, .. }) => Some((attrs, ident.clone())),
        Item::Use(ItemUse { attrs, tree, .. }) => {
            extract_ident_from_use_tree(tree).map(|ident| (attrs, ident))
        }
        _ => None,
    }
}

fn extract_context(item: &mut Item) -> Option<Ident> {
    let (attrs, ident) = extract_info(item)?;
    let id = attrs.iter().enumerate().find_map(|(i, attr)| {
        if let Meta::Path(path) = &attr.meta
            && path.is_ident("context")
        {
            return Some(i);
        }
        None
    })?;
    attrs.remove(id);
    Some(ident.clone())
}

fn extract_token(item: &mut Item) -> Option<EnrichedToken> {
    let (attrs, ident) = extract_info(item)?;
    let id = attrs.iter().enumerate().find_map(|(i, attr)| {
        if let Meta::NameValue(name_value) = &attr.meta
            && name_value.path.is_ident("token")
        {
            return Some(i);
        }
        None
    })?;
    // TODO: maybe `token` should be an actual attribute that automatically creates the
    // DFA?
    let attr = attrs.remove(id);
    let Meta::NameValue(name_value) = attr.meta else {
        unreachable!()
    };
    let syn::Expr::Lit(lit_value) = name_value.value else {
        emit_error!(
            ident.span(),
            "token attribute must define the corresponding regexpr, usage: #[token = r\"\\d\"]"
        );
        panic!()
    };
    let syn::Lit::Str(lit_str_value) = lit_value.lit else {
        emit_error!(
            ident.span(),
            "token regexpr must be a string literal, usage: #[token = r\"\\d\"]"
        );
        panic!()
    };
    Some(EnrichedToken::new(ident, lit_str_value.value()))
}

fn extract_non_terminal(item: &mut Item) -> Option<(EnrichedNonTerminal, bool)> {
    let (attrs, ident) = extract_info(item)?;
    let id = attrs.iter().enumerate().find_map(|(i, attr)| {
        if let Meta::Path(path) = &attr.meta
            && path.is_ident("non_terminal")
        {
            return Some(i);
        }
        None
    })?;
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
    Some((EnrichedNonTerminal::new(ident), is_start))
}

fn extract_production(item: &mut Item) -> Option<EnrichedProduction> {
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

    impl From<ProductionInternal> for EnrichedProduction {
        fn from(value: ProductionInternal) -> Self {
            let name = value.name;
            let head = value.head;
            let body = match value.body {
                Type::Path(type_path) => vec![type_path.path.get_ident().unwrap().clone()],
                Type::Tuple(type_tuple) => type_tuple
                    .elems
                    .iter()
                    .map(|t| {
                        let Type::Path(type_path) = t else {
                            panic!("body of production has to be a tuple of named types")
                        };
                        type_path.path.get_ident().unwrap().clone()
                    })
                    .collect(),
                _ => panic!("type must be a unit, a single type or a tuple"),
            };

            EnrichedProduction::new(name, head, body)
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
