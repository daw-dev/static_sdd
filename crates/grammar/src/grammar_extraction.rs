use std::{collections::HashSet, rc::Rc};

use dyn_grammar::{
    EnrichedGrammar,
    lalr::LalrAutomaton,
    non_terminal::EnrichedNonTerminal,
    production::EnrichedBaseProduction,
    symbolic_grammar::{self, SymbolicGrammar},
    token::{EnrichedToken, Match},
};
use ebnf_parser::EbnfProduction;
use itertools::Itertools;
use proc_macro_error::{emit_call_site_error, emit_call_site_warning, emit_error};
use syn::{
    Attribute, Ident, Item, ItemEnum, ItemStruct, ItemType, ItemUse, LitStr, Meta, Type, UseGroup,
    UseTree,
};

use crate::constructor::Constructor;

impl Constructor {
    pub fn extract(items: &mut [Item], internal_mod_name: Option<Ident>) -> Self {
        let mut tokens = Vec::new();
        let mut non_terminals = Vec::new();
        let mut ebnf_extra_non_terminals = HashSet::new();
        let mut productions = Vec::new();
        let mut start_symbol = None;
        let mut compiler_ctx = None;

        for item in items.iter_mut() {
            if let Some(ctx) = Self::extract_context(item) {
                if compiler_ctx.is_some() {
                    emit_error!(
                        ctx.span(),
                        "Compiler context defined for the second time here"
                    );
                    panic!();
                }
                compiler_ctx = Some(ctx);
            } else if let Some(token) = Self::extract_token(item) {
                tokens.push(token);
            } else if let Some((non_terminal, is_start)) = Self::extract_non_terminal(item) {
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
            } else if let Some(production) = Self::extract_production(item) {
                productions.push(production);
            } else if let Some(ebnf) = Self::extract_ebnf_production(item) {
                let extra_prods = ebnf.compile().0.into_iter().map(Into::into).collect_vec();
                let extra_nts = extra_prods
                    .iter()
                    .map(EnrichedBaseProduction::head)
                    .cloned()
                    .map(EnrichedNonTerminal::new);
                ebnf_extra_non_terminals.extend(extra_nts);
                productions.extend(extra_prods);
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

        non_terminals.extend(ebnf_extra_non_terminals);

        let enriched_grammar = Rc::new(EnrichedGrammar::new(
            compiler_ctx,
            non_terminals.into_iter().unique().collect(),
            tokens,
            start_symbol,
            productions,
        ));

        let sym_grammar = SymbolicGrammar::from(enriched_grammar.clone());

        let automaton = LalrAutomaton::compute(sym_grammar);

        Self {
            enriched_grammar,
            automaton,
            internal_mod_name,
        }
    }

    fn extract_ident_from_use_tree(tree: &mut UseTree) -> Option<Ident> {
        match tree {
            UseTree::Path(use_path) => Self::extract_ident_from_use_tree(&mut use_path.tree),
            UseTree::Name(use_name) => Some(use_name.ident.clone()),
            UseTree::Rename(use_rename) => Some(use_rename.rename.clone()),
            UseTree::Group(UseGroup { items, .. }) if items.len() == 1 => {
                Self::extract_ident_from_use_tree(items.pop().unwrap().value_mut())
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
                Self::extract_ident_from_use_tree(tree).map(|ident| (attrs, ident))
            }
            _ => None,
        }
    }

    fn extract_context(item: &mut Item) -> Option<Ident> {
        let (attrs, ident) = Self::extract_info(item)?;
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
        let (attrs, ident) = Self::extract_info(item)?;
        let mut res = None;
        attrs.retain(|attr| {
            if !attr.path().is_ident("token") {
                return true;
            }
            let match_string = attr.parse_args_with(|input: syn::parse::ParseStream| {
                if input.peek(syn::Ident) && input.peek2(syn::Token![=]) {
                    let regex_ident: Ident = input.parse()?;
                    if regex_ident != "regex" {
                        return Err(syn::Error::new(
                            regex_ident.span(),
                            "expected optional \"regex\"",
                        ));
                    }
                    input.parse::<syn::Token![=]>()?;
                    let regex: LitStr = input.parse()?;
                    Ok(Match::Regex(regex.value()))
                } else {
                    Ok(Match::Literal(input.parse::<LitStr>()?.value()))
                }
            });
            if let Ok(match_string) = match_string {
                res = Some(match_string);
                return false;
            }
            true
        });
        res.map(|match_string| EnrichedToken::new(ident, match_string.into()))
    }

    fn extract_non_terminal(item: &mut Item) -> Option<(EnrichedNonTerminal, bool)> {
        let (attrs, ident) = Self::extract_info(item)?;
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

    fn extract_production(item: &mut Item) -> Option<EnrichedBaseProduction> {
        match item {
            Item::Macro(mac) if mac.mac.path.is_ident("production") => mac
                .mac
                .parse_body_with(|input: syn::parse::ParseStream| {
                    let name = input.parse()?;
                    input.parse::<syn::Token![,]>()?;
                    let head = input.parse()?;
                    input.parse::<syn::Token![->]>()?;
                    let body = input.parse()?;
                    let body = match body {
                        Type::Path(type_path) => vec![
                            type_path
                                .path
                                .get_ident()
                                .expect("use only one type")
                                .clone(),
                        ],
                        Type::Tuple(type_tuple) => type_tuple
                            .elems
                            .iter()
                            .map(|t| {
                                let Type::Path(type_path) = t else {
                                    panic!("body of production has to be a tuple of named types")
                                };
                                type_path
                                    .path
                                    .get_ident()
                                    .expect("tuple of named types")
                                    .clone()
                            })
                            .collect(),
                        _ => panic!("type must be a unit, a single type or a tuple"),
                    };
                    let res = Ok(EnrichedBaseProduction::new(name, head, body));
                    if input.is_empty() {
                        return res;
                    }
                    input.parse::<syn::Token![,]>()?;
                    input.parse::<syn::Expr>()?;
                    res
                })
                .ok(),
            _ => None,
        }
    }

    fn extract_ebnf_production(item: &mut Item) -> Option<EbnfProduction> {
        match item {
            Item::Macro(mac) if mac.mac.path.is_ident("ebnf") => {
                mac.mac.parse_body::<EbnfProduction>().ok()
            }
            _ => None,
        }
    }
}
