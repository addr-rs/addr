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

use std::collections::HashSet;
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

    let mut funcs = TokenStream::new();
    let body = process(resources, string_match, &mut funcs);

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

                    info = lookup(labels, info);

                    if fqdn && info.len > 0 {
                        info.len += 1;
                    }

                    info
                }
            }

            #[inline]
            fn lookup<'a, T>(mut labels: T, mut info: Info) -> Info
                where T: Iterator<Item=&'a #iter>
            {
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
    };

    expanded.into()
}

#[derive(Debug, Clone, Copy)]
struct StringMatch(bool);

#[derive(Debug, Clone, Copy)]
struct Depth(usize);

fn process(resources: Vec<Uri>, string_match: bool, funcs: &mut TokenStream) -> TokenStream {
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
            let labels: Vec<_> = labels.into_iter().map(|label| {
                idna::domain_to_ascii(&label)
                    .expect(&format!("expected: a label that can be converted to ascii, found: {}", label))
            })
            .collect();
            tree.insert(labels.iter(), suffix.typ);
        }
    }

    build("lookup", tree.children_with_keys(), StringMatch(string_match), Depth(0), funcs)
}

// See https://www.reddit.com/r/rust/comments/91h6t8/generating_all_possible_case_variations_of_a/e2yw7qp/
// Thanks /u/ErichDonGubler!
fn all_cases(string: &str) -> HashSet<String> {
    let num_chars = string.chars().count();
    assert!(num_chars < 4, "found label `{}`: `anycase` feature currently supports labels with 3 characters or less only", string);

    let num_cases = usize::pow(2, num_chars as u32);
    let mut cases = HashSet::with_capacity(num_cases);

    let (upper, lower) = string.chars().fold(
        (Vec::with_capacity(num_chars), Vec::with_capacity(num_chars)),
        |(mut upper, mut lower), c| {
            upper.push(c.to_uppercase().to_string());
            lower.push(c.to_lowercase().to_string());
            (upper, lower)
        }
    );

    let len = string.len();
    for i in 0..num_cases {
        let mut s = String::with_capacity(len);
        for idx in 0..num_chars {
            if (i & (1 << idx)) == 0 {
                s.push_str(&lower[idx])
            } else {
                s.push_str(&upper[idx])
            }
        }
        cases.insert(s);
    }

    cases
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
        Func { name, len, iter, wild: TokenStream::new() }
    }

    fn root(self) -> TokenStream {
        let Func { name, len, wild, .. } = self;
        quote!{
            #[inline]
            fn #name(mut info: Info #wild) -> Info {
                info.len = #len;
                info
            }
        }
    }

    fn root_with_typ(self, typ: TokenStream) -> TokenStream {
        let Func { name, len, wild, .. } = self;
        quote!{
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
        let Func { name, len, iter, wild } = self;
        quote!{
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
        let Func { name, len, iter, wild } = self;
        quote!{
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

    fn leaf(self, typ: TokenStream) -> TokenStream {
        let Func { name, len, wild, .. } = self;
        quote!{
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
        quote!{
            #[inline]
            fn #name(#wild acc: usize) -> Info {
                Info {
                    len: acc,
                    typ: Some(Type::#typ),
                }
            }
        }
    }

    fn inner(self, body: TokenStream) -> TokenStream {
        let Func { name, len, iter, wild } = self;
        quote!{
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
        let Func { name, len, iter, wild } = self;
        quote!{
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
}

fn ident(name: &str) -> syn::Ident {
    syn::parse_str::<syn::Ident>(&name).unwrap()
}

fn pat(label: &str, StringMatch(string_match): StringMatch) -> (TokenStream, TokenStream) {
    let len = label.len();
    if label == "_" {
        (quote!(wild), quote!(wild.len()))
    } else if cfg!(feature = "anycase") {
        let label = label.trim_left_matches('!');
        let cases = all_cases(label).into_iter();
        if string_match {
            let pats = cases.map(|label| quote!(#label));
            (pat_opts(pats), quote!(#len))
        } else {
            let pats = cases.map(|x| array_expr(&x)).map(|pat| quote!(#pat));
            (pat_opts(pats), quote!(#len))
        }
    } else {
        if string_match {
            (quote!(#label), quote!(#len))
        } else {
            let pat = array_expr(label);
            (quote!(#pat), quote!(#len))
        }
    }
}

fn pat_opts<T>(opts: T) -> TokenStream
    where T: Iterator<Item=TokenStream> 
{
    let mut pat = TokenStream::new();
    for (i, x) in opts.enumerate() {
        if i > 0 {
            pat.append_all(quote!(|));
        }
        pat.append_all(x);
    }
    pat
}

fn build(
    fname: &str,
    list: Vec<(&String, &SequenceTrie<String, Type>)>,
    StringMatch(string_match): StringMatch,
    Depth(depth): Depth,
    funcs: &mut TokenStream,
) -> TokenStream {
    if list.is_empty() {
        if depth == 0 && !cfg!(test) {
            panic!("
                Found empty list. This implementation doesn't support empty lists.
                If you do want one, you can easily implement the trait `psl::Psl`
                by merely putting `None` in the body.
            ");
        }
    }

    let iter = if string_match { quote!(str) } else { quote!([u8]) };

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
        let children = build(&name, tree.children_with_keys(), StringMatch(string_match), Depth(depth + 1), funcs);
        let (pat, len) = pat(label, StringMatch(string_match));
        let mut func = Func::new(fident.clone(), len, iter.clone());

        // Exception rules
        if label.starts_with('!') {
            if !children.is_empty() {
                panic!("an exclamation mark must be at the end of an exception rule: {}", label)
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
                        footer.append_all(quote!{
                            wild => #fident(info, wild),
                        });
                    } else {
                        func.wild = quote!(wild: &#iter);
                        funcs.append_all(func.root_with_typ(typ));
                        footer.append_all(quote!{
                            wild => #fident(wild),
                        });
                    }
                } else {
                    if typ.is_empty() {
                        func.wild = quote!(wild: &#iter,);
                        funcs.append_all(func.nested_root(children));
                        footer.append_all(quote!{
                            wild => #fident(info, wild, labels),
                        });
                    } else {
                        func.wild = quote!(wild: &#iter,);
                        funcs.append_all(func.nested_root_with_typ(typ, children));
                        footer.append_all(quote!{
                            wild => #fident(wild, labels),
                        });
                    }
                }
            }
            
            else {
                if children.is_empty() {
                    func.wild = quote!(wild: &#iter,);
                    funcs.append_all(func.leaf(typ));
                    footer.append_all(quote!{
                        wild => #fident(wild, acc),
                    });
                } else {
                    if typ.is_empty() {
                        func.wild = quote!(wild: &#iter,);
                        funcs.append_all(func.inner(children));
                        footer.append_all(quote!{
                            wild => #fident(info, wild, labels, acc),
                        });
                    } else {
                        func.wild = quote!(wild: &#iter,);
                        funcs.append_all(func.inner_with_typ(typ, children));
                        footer.append_all(quote!{
                            wild => #fident(wild, labels, acc),
                        });
                    }
                }
            }
        }
        
        // Plain rules
        else {
            if depth == 0 {
                if children.is_empty() {
                    if typ.is_empty() {
                        funcs.append_all(func.root());
                        body.append_all(quote!{
                            #pat => #fident(info),
                        });
                    } else {
                        funcs.append_all(func.root_with_typ(typ));
                        body.append_all(quote!{
                            #pat => #fident(),
                        });
                    }
                }
                
                else {
                    if typ.is_empty() {
                        funcs.append_all(func.nested_root(children));
                        body.append_all(quote!{
                            #pat => #fident(info, labels),
                        });
                    } else {
                        funcs.append_all(func.nested_root_with_typ(typ, children));
                        body.append_all(quote!{
                            #pat => #fident(labels),
                        });
                    }
                }
            }
            
            else {
                if children.is_empty() {
                    funcs.append_all(func.leaf(typ));
                    body.append_all(quote!{
                        #pat => #fident(acc),
                    });
                }
                
                else {
                    if typ.is_empty() {
                        funcs.append_all(func.inner(children));
                        body.append_all(quote!{
                            #pat => #fident(info, labels, acc),
                        });
                    } else {
                        funcs.append_all(func.inner_with_typ(typ, children));
                        body.append_all(quote!{
                            #pat => #fident(labels, acc),
                        });
                    }
                }
            }
        }
    }

    if head.is_empty() && body.is_empty() && footer.is_empty() {
        return TokenStream::new();
    }

    if footer.is_empty() {
        if fname == "lookup" {
            footer.append_all(quote!{
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
