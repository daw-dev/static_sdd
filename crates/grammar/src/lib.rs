use proc_macro::TokenStream;
use proc_macro_error::{emit_call_site_warning, emit_error, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Item, ItemMod, Meta};

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
    let mut start_symbol = None;

    for item in items.iter_mut() {
        match item {
            // Case A: It is a Struct (Potential Token or NonTerminal)
            Item::Struct(s) => {
                // Helper to check attributes (defined below)
                if let Some(attr) = remove_attribute(&mut s.attrs, "token") {
                    // It's a Token!
                    tokens.push(s.ident.clone()); // Save the name (e.g., "Plus")
                    eprintln!("Found Token: {}", s.ident);
                } else if let Some(attr) = remove_attribute(&mut s.attrs, "non_terminal") {
                    // It's a NonTerminal!
                    non_terminals.push(s.ident.clone());
                    eprintln!("Found NonTerminal: {}", s.ident);
                }
            }

            Item::Type(t) => {
                if let Some(attr) = remove_attribute(&mut t.attrs, "token") {
                    // It's a Token!
                    tokens.push(t.ident.clone()); // Save the name (e.g., "Plus")
                    eprintln!("Found Token: {}", t.ident);
                } else if let Some(attr) = remove_attribute(&mut t.attrs, "non_terminal") {
                    if let Some(start_attr) = remove_attribute(&mut t.attrs, "start_symbol") {
                        if let Some(sym) = start_symbol {
                            emit_error!(
                                start_attr.span(),
                                "only one start symbol is accepted, both {} and {} are declared as start symbol",
                                sym, t.ident
                            );
                        }
                        start_symbol = Some(t.ident.to_string());
                    }
                    // It's a NonTerminal!
                    non_terminals.push(t.ident.clone());
                    eprintln!("Found NonTerminal: {}", t.ident);
                }
            }

            Item::Enum(e) => {
                if let Some(attr) = remove_attribute(&mut e.attrs, "token") {
                    // It's a Token!
                    tokens.push(e.ident.clone()); // Save the name (e.g., "Plus")
                    eprintln!("Found Token: {}", e.ident);
                } else if let Some(attr) = remove_attribute(&mut e.attrs, "non_terminal") {
                    // It's a NonTerminal!
                    non_terminals.push(e.ident.clone());
                    eprintln!("Found NonTerminal: {}", e.ident);
                }
            }

            // Case B: It represents a Macro invocation (The production! calls)
            Item::Macro(m) => {
                if m.mac.path.is_ident("production") {
                    // It's a production rule!
                    // We will parse the body later.
                    productions.push(m.clone());
                    eprintln!("Found Production Macro");
                }
            }

            _ => {
                eprintln!("Something else found");
            }
        }
    }

    if start_symbol.is_none() {
        emit_call_site_warning!("no start symbol was declared, using {}", non_terminals[0]);
    }

    quote! {
        #module

        pub fn parse() {
            println!("Hello World!");
        }
    }.into()
}

/// Checks if a list of attributes contains a specific identifier (e.g., #[token])
fn remove_attribute(attrs: &mut Vec<syn::Attribute>, name: &str) -> Option<syn::Attribute> {
    let index = attrs.iter().enumerate().find_map(|(i, attr)| {
        // Parse the attribute meta to check its path
        if let Meta::Path(path) = &attr.meta {
            if path.is_ident(name) { Some(i) } else { None }
        } else if let Meta::NameValue(name_value) = &attr.meta {
            if name_value.path.is_ident(name) {
                Some(i)
            } else {
                None
            }
        } else {
            None
        }
    });

    index.map(|i| attrs.remove(i))
}
