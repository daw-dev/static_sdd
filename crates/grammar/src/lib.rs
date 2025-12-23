use crate::{grammar_extraction::extract_grammar, item_injections::inject_items};
use dyn_grammar::{slr::automaton::SlrAutomaton, symbolic_grammar::SymbolicGrammar};
use proc_macro::TokenStream;
use proc_macro_error::{emit_call_site_error, proc_macro_error};
use quote::quote;
use syn::{File, ItemMod, parse_quote};

mod grammar_extraction;
mod item_injections;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn grammar(_attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(mut module) = syn::parse::<ItemMod>(item.clone()) {
        let (_, items) = module.content.as_mut().expect("grammar module must be inline (contain braces)");

        let enriched_grammar = extract_grammar(items);
        let symbolic_grammar = SymbolicGrammar::from(&enriched_grammar);

        let automaton = SlrAutomaton::compute(&symbolic_grammar);

        inject_items(items, enriched_grammar);

        let parse_fn = parse_quote! {
            pub fn parse(word: impl Into<String>) {
                println!("{}", word.into());
            }
        };

        items.push(parse_fn);

        quote! { #module }.into()
    } else if let Ok(File { items, .. }) = &mut syn::parse(item) {
        todo!()
    } else {
        emit_call_site_error!("a grammar is either an inline module or a file");
        panic!()
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

dummy_attribute!(token, "type aliases, structs, enums or use directives");
dummy_attribute!(start_symbol, "type aliases, structs, enums or use directives");
dummy_attribute!(non_terminal, "type aliases, structs, enums or use directives");
dummy_attribute!(left_associative, "production macros");
dummy_attribute!(right_associative, "production macros");
dummy_attribute!(precedence, "production marcos");
dummy_attribute!(context, "ONLY ONE type alias, struct, enum or use directive");
