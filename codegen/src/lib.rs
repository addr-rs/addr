//! Download and compile the Public Suffix List to native Rust code

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

#[proc_macro_derive(Psl, attributes(psl))]
pub fn derive_psl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Attrs { resources } = attrs(&input.attrs);

    let krate = if cfg!(feature = "prefix") {
        quote!(psl::)
    } else {
        TokenStream::new()
    };

    let string_match = if let Ok(val) = env::var("PSL_STRING_MATCH") {
        if val == "1" { true } else { false }
    } else {
        false
    };

    let (labels, iter) = if string_match {
        let labels = quote! {
            match ::core::str::from_utf8(domain) {
                Ok(domain) => domain.rsplit('.'),
                Err(_) => {
                    return info;
                }
            }
        };
        (labels, quote!(str))
    } else {
        let labels = quote!(domain.rsplit(|x| *x == b'.'));
        (labels, quote!([u8]))
    };

    let body = body(resources, string_match);

    let expanded = quote! {
        mod __psl_impl {
            use #krate {Psl, Type, Info};

            impl #impl_generics Psl for super::#name #ty_generics #where_clause {
                fn find(&self, domain: &[u8]) -> Info {
                    let mut info = Info { len: 0, typ: None };

                    let mut labels = #labels;

                    let fqdn = if domain.ends_with(b".") {
                        labels.next();
                        true
                    } else {
                        false
                    };

                    info = lookup(labels, info.len, info);

                    if fqdn && info.len > 0 {
                        info.len += 1;
                    }

                    info
                }
            }

            #[inline]
            fn lookup<'a>(mut labels: impl Iterator<Item=&'a #iter>, mut len: usize, mut info: Info) -> Info {
                #body
                info
            }
        }
    };

    expanded.into()
}

#[derive(Debug)]
struct Depth(usize);

#[derive(Debug)]
struct StringMatch(bool);

fn body(resources: Vec<Uri>, string_match: bool) -> TokenStream {
    use self::Uri::*;

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
            let labels: Vec<_> = rule.split('.')
                .map(|s| s.to_owned())
                .rev()
                .collect();
            tree.insert(labels.iter(), suffix.typ);
            if cfg!(feature = "punycode") {
                let labels: Vec<_> = labels.into_iter().map(|label| {
                    idna::domain_to_ascii(&label)
                        .expect(&format!("expected: a label that can be converted to ascii, found: {}", label))
                })
                .collect();
                tree.insert(labels.iter(), suffix.typ);
            }
        }
    }

    build(tree.children_with_keys(), Depth(0), StringMatch(string_match))
}

fn build(list: Vec<(&String, &SequenceTrie<String, Type>)>, Depth(depth): Depth, StringMatch(string_match): StringMatch) -> TokenStream {
    if list.is_empty() {
        if depth == 0 {
            panic!("
                Found empty list. This implementation doesn't support empty lists.
                If you do want one, you can easily implement the trait `psl::Psl`
                by merely putting `None` in the body.
            ");
        }
        return TokenStream::new();
    }

    let mut head = TokenStream::new();
    let mut body = TokenStream::new();
    let mut footer = TokenStream::new();
    for (label, tree) in list {
        let mut info = if depth == 0 {
            // invoke the wildcard rule
            quote!(info.len = len;)
        } else {
            TokenStream::new()
        };
        if let Some(val) = tree.value() {
            let t = match *val {
                Type::Icann => syn::parse_str::<syn::Type>("Icann").unwrap(),
                Type::Private => syn::parse_str::<syn::Type>("Private").unwrap(),
            };
            info = quote! {
                info = Info { len, typ: Some(Type::#t) };
            };
        }
        let children = build(tree.children_with_keys(), Depth(depth + 1), StringMatch(string_match));
        let pat = |label| {
            if string_match {
                quote!(#label)
            } else {
                let pat = array_expr(label);
                quote!(#pat)
            }
        };
        let plus_1 = if depth > 0 { quote!(+ 1) } else { TokenStream::new() };
        if label.starts_with('!') {
            let label = label.trim_left_matches('!');
            let pat = pat(label);
            head.append_all(quote! {
                #pat => {
                    #info
                }
            });
        } else if label == "_" {
            footer.append_all(quote! {
                wild => {
                    len += wild.len() #plus_1;
                    #info
                    #children
                }
            });
        } else {
            let pat = pat(label);
            body.append_all(quote! {
                #pat => {
                    len += #pat.len() #plus_1;
                    #info
                    #children
                }
            });
        }
    }

    if footer.is_empty() {
        let eom = if depth == 0 {
            quote! {
                val => {
                    info.len = val.len();
                }
            }
        } else {
            quote!(_ => {})
        };
        footer.append_all(eom);
    }

    quote! {
        match labels.next() {
            Some(label) => {
                match label {
                    #head
                    #body
                    #footer
                }
            }
            None => {}
        }
    }
}

fn array_expr(label: &str) -> syn::ExprArray {
    let label = format!("{:?}", label.as_bytes());
    syn::parse_str(&label).unwrap()
}

#[derive(Debug)]
enum Uri {
    Url(String),
    Path(String),
}

#[derive(Debug)]
struct Attrs {
    resources: Vec<Uri>,
}

fn lit_str(token: syn::Ident, lit: &Lit) -> Uri {
    match *lit {
        Lit::Str(ref s) => {
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

fn attrs(list: &[Attribute]) -> Attrs {
    use self::Meta::*;

    let mut attrs = Attrs { resources: Vec::new() };

    for key in &["PSL_PATH", "PSL_PATHS", "PSL_URL", "PSL_URLS"] {
        if let Ok(val) = env::var(key) {
            for resource in val.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
                if key.contains("URL") {
                    attrs.resources.push(Uri::Url(resource.to_owned()));
                } else {
                    attrs.resources.push(Uri::Path(resource.to_owned()));
                }
            }
        }
    }

    if !attrs.resources.is_empty() {
        return attrs;
    }

    for attr in list {
        if let Some(List(ml)) = attr.interpret_meta() {
            if ml.ident == "psl" {
                for nm in ml.nested {
                    match nm {
                        NestedMeta::Meta(meta) => {
                            match meta {
                                NameValue(nv) => {
                                    let token = nv.ident;
                                    if token == "url" || token == "path" {
                                        attrs.resources.push(lit_str(token, &nv.lit));
                                    }
                                }
                                List(list) => {
                                    use self::NestedMeta::*;

                                    let token = list.ident;
                                    if token == "url" || token == "path" {
                                        for item in list.nested {
                                            match item {
                                                Literal(lit) => {
                                                    attrs.resources.push(lit_str(token.clone(), &lit));
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

    attrs
}
