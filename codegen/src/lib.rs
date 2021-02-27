//! Download and compile the Public Suffix List to native Rust code

use idna::domain_to_unicode;
use proc_macro2::TokenStream;
use psl_lexer::{List, Type};
use quote::quote;
use quote::TokenStreamExt;
use sequence_trie::SequenceTrie;
use std::env;
use std::path::Path;
use std::str::FromStr;

pub fn compile_psl<P: AsRef<Path>>(path: P) -> proc_macro2::TokenStream {
    let mut funcs = TokenStream::new();
    let body = process(&mut funcs, path);

    quote! {
            use crate::{Type, Info};

            #[inline]
            pub(super) fn lookup<'a, T>(mut labels: T) -> Info
                where T: Iterator<Item=&'a [u8]>
            {
                let mut info = Info { len: 0, typ: None };
                match labels.next() {
                    Some(label) => {
                        match label {
                            #body
                        }
                    }
                    None => info,
                }
            }

            #funcs
    }
}

#[derive(Debug, Clone, Copy)]
struct Depth(usize);

fn process<P: AsRef<Path>>(funcs: &mut TokenStream, path: P) -> TokenStream {
    let data = psl_lexer::request(psl_lexer::LIST_URL).expect("failed to download the list");
    std::fs::write(path, &data).expect("failed to write the list to disk");
    let mut list = List::from_str(&data).expect("failed to build the list");

    let mut tlds = Vec::new();
    for key in &["PSL_TLD", "PSL_TLDS"] {
        if let Ok(val) = env::var(key) {
            for input in val
                .split(',')
                .map(|x| x.trim().to_lowercase())
                .filter(|x| !x.is_empty())
            {
                let (tld, res) = domain_to_unicode(&input);
                if res.is_err() {
                    panic!("failed to parse `{}` as valid unicode domain", input);
                }
                let val = list
                    .rules
                    .remove(&tld)
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
            let rule = suffix.rule.replace('*', "_");
            let labels: Vec<_> = rule.split('.').map(|s| s.to_owned()).rev().collect();
            tree.insert(labels.iter(), suffix.typ);
            let labels: Vec<_> = labels
                .into_iter()
                .map(|label| {
                    idna::domain_to_ascii(&label).unwrap_or_else(|_| {
                        panic!(
                            "expected: a label that can be converted to ascii, found: {}",
                            label
                        )
                    })
                })
                .collect();
            tree.insert(labels.iter(), suffix.typ);
        }
    }

    build("lookup", tree.children_with_keys(), Depth(0), funcs)
}

#[derive(Debug, Clone)]
struct Func {
    name: syn::Ident,
    len: TokenStream,
    iter: TokenStream,
    wild: TokenStream,
}

impl Func {
    fn new(name: syn::Ident, len: TokenStream, iter: TokenStream) -> Self {
        Func {
            name,
            len,
            iter,
            wild: TokenStream::new(),
        }
    }

    fn root(self) -> TokenStream {
        let Func {
            name, len, wild, ..
        } = self;
        quote! {
            #[inline]
            fn #name(mut info: Info #wild) -> Info {
                info.len = #len;
                info
            }
        }
    }

    fn root_with_typ(self, typ: TokenStream) -> TokenStream {
        let Func {
            name, len, wild, ..
        } = self;
        quote! {
            #[inline]
            fn #name(#wild) -> Info {
                Info {
                    len: #len,
                    typ: Some(Type::#typ),
                }
            }
        }
    }

    fn nested_root(self, body: TokenStream) -> TokenStream {
        let Func {
            name,
            len,
            iter,
            wild,
        } = self;
        quote! {
            #[inline]
            fn #name<'a, T>(mut info: Info, #wild mut labels: T) -> Info
                where T: Iterator<Item=&'a #iter>
            {
                let acc = #len;
                info.len = acc;
                match labels.next() {
                    Some(label) => {
                        match label {
                            #body
                        }
                    }
                    None => info,
                }
            }
        }
    }

    fn nested_root_with_typ(self, typ: TokenStream, body: TokenStream) -> TokenStream {
        let Func {
            name,
            len,
            iter,
            wild,
        } = self;
        quote! {
            #[inline]
            fn #name<'a, T>(#wild mut labels: T) -> Info
                where T: Iterator<Item=&'a #iter>
            {
                let acc = #len;
                let info = Info {
                    len: acc,
                    typ: Some(Type::#typ),
                };
                match labels.next() {
                    Some(label) => {
                        match label {
                            #body
                        }
                    }
                    None => info,
                }
            }
        }
    }

    fn inner(self, body: TokenStream) -> TokenStream {
        let Func {
            name,
            len,
            iter,
            wild,
        } = self;
        quote! {
            #[inline]
            fn #name<'a, T>(info: Info, #wild mut labels: T, mut acc: usize) -> Info
                where T: Iterator<Item=&'a #iter>
            {
                acc += 1 + #len;
                match labels.next() {
                    Some(label) => {
                        match label {
                            #body
                        }
                    }
                    None => info,
                }
            }
        }
    }

    fn inner_with_typ(self, typ: TokenStream, body: TokenStream) -> TokenStream {
        let Func {
            name,
            len,
            iter,
            wild,
        } = self;
        quote! {
            #[inline]
            fn #name<'a, T>(#wild mut labels: T, mut acc: usize) -> Info
                where T: Iterator<Item=&'a #iter>
            {
                acc += 1 + #len;
                let info = Info {
                    len: acc,
                    typ: Some(Type::#typ),
                };
                match labels.next() {
                    Some(label) => {
                        match label {
                            #body
                        }
                    }
                    None => info,
                }
            }
        }
    }

    fn leaf(self, typ: TokenStream) -> TokenStream {
        let Func {
            name, len, wild, ..
        } = self;
        quote! {
            #[inline]
            fn #name(#wild acc: usize) -> Info {
                Info {
                    len: acc + 1 + #len,
                    typ: Some(Type::#typ),
                }
            }
        }
    }

    fn bang_leaf(self, typ: TokenStream) -> TokenStream {
        let Func { name, wild, .. } = self;
        quote! {
            #[inline]
            fn #name(#wild acc: usize) -> Info {
                Info {
                    len: acc,
                    typ: Some(Type::#typ),
                }
            }
        }
    }
}

fn ident(name: &str) -> syn::Ident {
    syn::parse_str::<syn::Ident>(&name).unwrap()
}

fn pat(label: &str) -> (TokenStream, TokenStream) {
    let label = label.trim_start_matches('!');
    let len = label.len();
    if label == "_" {
        (quote!(wild), quote!(wild.len()))
    } else {
        let pat = array_expr(label);
        (quote!(#pat), quote!(#len))
    }
}

fn build(
    fname: &str,
    list: Vec<(&String, &SequenceTrie<String, Type>)>,
    Depth(depth): Depth,
    funcs: &mut TokenStream,
) -> TokenStream {
    if list.is_empty() && depth == 0 && !cfg!(test) {
        panic!("Found empty list. This implementation doesn't support empty lists.");
    }

    let iter = quote!([u8]);

    let mut head = TokenStream::new();
    let mut body = TokenStream::new();
    let mut footer = TokenStream::new();

    for (i, (label, tree)) in list.into_iter().enumerate() {
        let typ = match tree.value() {
            Some(val) => {
                let typ = match *val {
                    Type::Icann => quote!(Icann),
                    Type::Private => quote!(Private),
                };
                quote!(#typ)
            }
            None => TokenStream::new(),
        };

        let name = format!("{}_{}", fname, i);
        let fident = ident(&name);
        let children = build(&name, tree.children_with_keys(), Depth(depth + 1), funcs);
        let (pat, len) = pat(label);
        let mut func = Func::new(fident.clone(), len, iter.clone());

        // Exception rules
        if label.starts_with('!') {
            if !children.is_empty() {
                panic!(
                    "an exclamation mark must be at the end of an exception rule: {}",
                    label
                )
            }
            funcs.append_all(func.bang_leaf(typ));
            if depth == 0 {
                panic!("an exception rule cannot be in TLD position: {}", label);
            } else {
                head.append_all(quote! {
                    #pat => #fident(acc),
                });
            }
        }
        // Wildcard rules
        else if label == "_" {
            if depth == 0 {
                if children.is_empty() {
                    if typ.is_empty() {
                        func.wild = quote!(, wild: &#iter);
                        funcs.append_all(func.root());
                        footer.append_all(quote! {
                            wild => #fident(info, wild),
                        });
                    } else {
                        func.wild = quote!(wild: &#iter);
                        funcs.append_all(func.root_with_typ(typ));
                        footer.append_all(quote! {
                            wild => #fident(wild),
                        });
                    }
                } else if typ.is_empty() {
                    func.wild = quote!(wild: &#iter,);
                    funcs.append_all(func.nested_root(children));
                    footer.append_all(quote! {
                        wild => #fident(info, wild, labels),
                    });
                } else {
                    func.wild = quote!(wild: &#iter,);
                    funcs.append_all(func.nested_root_with_typ(typ, children));
                    footer.append_all(quote! {
                        wild => #fident(wild, labels),
                    });
                }
            } else if children.is_empty() {
                func.wild = quote!(wild: &#iter,);
                funcs.append_all(func.leaf(typ));
                footer.append_all(quote! {
                    wild => #fident(wild, acc),
                });
            } else if typ.is_empty() {
                func.wild = quote!(wild: &#iter,);
                funcs.append_all(func.inner(children));
                footer.append_all(quote! {
                    wild => #fident(info, wild, labels, acc),
                });
            } else {
                func.wild = quote!(wild: &#iter,);
                funcs.append_all(func.inner_with_typ(typ, children));
                footer.append_all(quote! {
                    wild => #fident(wild, labels, acc),
                });
            }
        }
        // Plain rules
        else if depth == 0 {
            if children.is_empty() {
                if typ.is_empty() {
                    funcs.append_all(func.root());
                    body.append_all(quote! {
                        #pat => #fident(info),
                    });
                } else {
                    funcs.append_all(func.root_with_typ(typ));
                    body.append_all(quote! {
                        #pat => #fident(),
                    });
                }
            } else if typ.is_empty() {
                funcs.append_all(func.nested_root(children));
                body.append_all(quote! {
                    #pat => #fident(info, labels),
                });
            } else {
                funcs.append_all(func.nested_root_with_typ(typ, children));
                body.append_all(quote! {
                    #pat => #fident(labels),
                });
            }
        } else if children.is_empty() {
            funcs.append_all(func.leaf(typ));
            body.append_all(quote! {
                #pat => #fident(acc),
            });
        } else if typ.is_empty() {
            funcs.append_all(func.inner(children));
            body.append_all(quote! {
                #pat => #fident(info, labels, acc),
            });
        } else {
            funcs.append_all(func.inner_with_typ(typ, children));
            body.append_all(quote! {
                #pat => #fident(labels, acc),
            });
        }
    }

    if head.is_empty() && body.is_empty() && footer.is_empty() {
        return TokenStream::new();
    }

    if footer.is_empty() {
        if fname == "lookup" {
            footer.append_all(quote! {
                wild => {
                    info.len = wild.len();
                    info
                }
            });
        } else {
            footer.append_all(quote!(_ => info,));
        }
    }

    quote! {
        #head
        #body
        #footer
    }
}

fn array_expr(label: &str) -> syn::ExprArray {
    let label = format!("{:?}", label.as_bytes());
    syn::parse_str(&label).unwrap()
}
