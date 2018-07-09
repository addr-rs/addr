#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate psl_lexer;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate sequence_trie;
extern crate idna;

use std::env;
use idna::domain_to_unicode;

use psl_lexer::{List, Type};
use proc_macro2::TokenStream;
use syn::{DeriveInput, Attribute, Meta, NestedMeta, Lit};
use quote::TokenStreamExt;
use sequence_trie::SequenceTrie;

#[cfg(feature = "prefix")]
fn krate() -> TokenStream {
    quote!(::psl::)
}

#[cfg(not(feature = "prefix"))]
fn krate() -> TokenStream {
    quote!(::)
}

#[proc_macro_derive(Psl, attributes(psl))]
pub fn derive_psl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = body(&input.attrs);

    let krate = krate();

    let expanded = quote! {
        impl #impl_generics #krate Psl for #name #ty_generics #where_clause {
            #[allow(unused_assignments)]
            fn find<'a, T>(&self, labels: T) -> Option<#krate Info>
                where T: IntoIterator<Item=&'a str>
            {
                let mut suffix = #krate Info::Incomplete;
                let mut index = 1;

                let mut labels = labels.into_iter();
                #body
            }
        }
    };

    expanded.into()
}

struct AtRoot(bool);

fn body(attrs: &[Attribute]) -> TokenStream {
    use self::Uri::*;

    let resources = uri(attrs);

    let mut list = if resources.is_empty() {
        List::fetch()
            .unwrap_or_else(|error| panic!("failed to download the list: {}", error))
    } else {
        let mut list = None;

        for resource in resources {
            let (res, uri, from_url) = match resource {
                Url(url) => { (List::from_url(&url), url, true) }
                Path(path) => { (List::from_path(&path), path, false) }
            };
            match res {
                Ok(l) => {
                    list = Some(l);
                    break;
                }
                Err(error) => {
                    if from_url {
                        eprintln!("failed to download the list from {}: {}", uri, error);
                    } else {
                        eprintln!("failed to open the list from {}: {}", uri, error);
                    }
                }
            }
        }

        list.expect("could not get the list from any of the supplied resource(s)")
    };

    let mut tlds = Vec::new();
    for key in &["PSL_TLD", "PSL_TLDS"] {
        if let Ok(val) = env::var(key) {
            for input in val.split(',').map(|x| x.trim().to_lowercase()).filter(|x| !x.is_empty()) {
                let (tld, res) = domain_to_unicode(&input);
                if res.is_err() {
                    panic!("failed to parse `{}` as valid unicode domain", input);
                }
                let val = list.rules.remove(&tld)
                    .unwrap_or_else(|| panic!("`{}` not found in the list", input));
                tlds.push((tld, val));
            }
        }
    }
    if !tlds.is_empty() {
        list.rules = tlds.into_iter().collect();
    }

    let mut tree = SequenceTrie::new();
    for val in list.rules.values() {
        for suffix in val {
            let rule = suffix.rule.replace("*", "_");
            let labels = rule.split('.').rev();
            tree.insert(labels, suffix.typ);
        }
    }

    build(tree.children_with_keys(), AtRoot(true))
}

fn build(list: Vec<(&String, &SequenceTrie<String, Type>)>, AtRoot(at_root): AtRoot) -> TokenStream {
    if list.is_empty() {
        if at_root {
            panic!("
                Found empty list. This implementation doesn't support empty lists.
                If you do want one, you can easily implement the trait `psl::Psl`
                by merely putting `None` in the body.
            ");
        }
        return TokenStream::new();
    }

    let krate = krate();

    let mut head = TokenStream::new();
    let mut body = TokenStream::new();
    let mut footer = TokenStream::new();
    for (label, tree) in list {
        let mut suffix = TokenStream::new();
        if let Some(typ) = tree.value() {
            let typ = match typ {
                Type::Icann => syn::parse_str::<syn::Type>("Icann").unwrap(),
                Type::Private => syn::parse_str::<syn::Type>("Private").unwrap(),
            };
            suffix = quote!(suffix = #krate Info::Suffix(index, #krate Type::#typ););
        }
        let children = if tree.children().is_empty() {
            quote! {
                Some(suffix)
            }
        } else {
            build(tree.children_with_keys(), AtRoot(false))
        };
        if label.starts_with('!') {
            let label = label.trim_left_matches('!');
            head.append_all(quote! {
                #label => {
                    index -= 1;
                    #suffix
                    #children
                }
            });
        } else if label == "_" {
            footer.append_all(quote! {
                _ => {
                    #suffix
                    #children
                }
            });
        } else {
            body.append_all(quote! {
                #label => {
                    #suffix
                    #children
                }
            });
        }
    }

    let (index_incr, end_of_matches) = if at_root {
        (TokenStream::new(), quote!(None))
    } else {
        let index = quote!(index += 1;);
        let eom = quote!(Some(suffix));
        (index, eom)
    };

    if footer.is_empty() {
        footer.append_all(quote! {
            _ => { #end_of_matches }
        });
    }

    quote! {
        #index_incr
        match labels.next() {
            Some(label) => {
                match label {
                    #head
                    #body
                    #footer
                }
            }
            None => { #end_of_matches }
        }
    }
}

enum Uri {
    Url(String),
    Path(String),
}

fn lit_str(token: syn::Ident, lit: &Lit) -> Uri {
    match lit {
        Lit::Str(s) => {
            let resource = s.value();
            if token == "url" {
                Uri::Url(resource)
            } else {
                Uri::Path(resource)
            }
        }
        _ => panic!("`{}` must be a UTF-8 string literal", token),
    }
}

fn uri(attrs: &[Attribute]) -> Vec<Uri> {
    use self::Meta::*;

    let mut resources = Vec::new();

    for key in &["PSL_PATH", "PSL_PATHS", "PSL_URL", "PSL_URLS"] {
        if let Ok(val) = env::var(key) {
            for resource in val.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
                if key.contains("URL") {
                    resources.push(Uri::Url(resource.to_owned()));
                } else {
                    resources.push(Uri::Path(resource.to_owned()));
                }
            }
        }
    }

    if !resources.is_empty() {
        return resources;
    }

    for attr in attrs {
        if let Some(List(ml)) = attr.interpret_meta() {
            if ml.ident == "psl" {
                for nm in ml.nested {
                    match nm {
                        NestedMeta::Meta(meta) => {
                            match meta {
                                NameValue(nv) => {
                                    let token = nv.ident;
                                    if token == "url" || token == "path" {
                                        resources.push(lit_str(token, &nv.lit));
                                    }
                                }
                                List(list) => {
                                    use self::NestedMeta::*;

                                    let token = list.ident;
                                    if token == "url" || token == "path" {
                                        for item in list.nested {
                                            match item {
                                                Literal(lit) => {
                                                    resources.push(lit_str(token.clone(), &lit));
                                                }
                                                Meta(_) => {
                                                    panic!("expected a {}, found an object", token);
                                                }
                                            }
                                        }
                                    }
                                }
                                Word(token) => {
                                    if token == "url" || token == "path" {
                                        panic!("expected either a list of {}s or a key value pair, found an identifier", token);
                                    }
                                }
                            }
                        }
                        NestedMeta::Literal(_) => {
                            panic!("expected a key value pair of urls or paths, found a literal");
                        }
                    }
                }
            }
        }
    }

    resources
}
