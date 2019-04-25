// The code is in this module is mostly copied 1-1 from [syn](https://github.com/dtolnay/syn) crate.
// Original license and copyright apply.

use proc_macro2::{Punct, Spacing, TokenTree};
use std::iter::FromIterator;
use syn::{
    parenthesized,
    parse::{self, Parse},
    punctuated::{Pair, Punctuated},
    token, Token,
};

#[derive(Debug, Clone)]
pub struct ArgDefault {
    pub eq: syn::token::Eq,
    pub default_token: Option<syn::token::Default>,
    pub value: syn::Expr,
}

impl quote::ToTokens for ArgDefault {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.eq.to_tokens(tokens);
        self.default_token.to_tokens(tokens);
        if let Some(tok) = self.default_token.as_ref() {
            tok.to_tokens(tokens);
        }
        self.value.to_tokens(tokens);
    }
}

#[derive(Clone, Debug)]
pub struct ArgCaptured {
    pub pat: syn::Pat,

    // Custom field.
    pub vararg_token: Option<Token![*]>,
    // Custom field.
    pub optional_token: Option<Token![?]>,

    pub colon_token: syn::token::Colon,
    pub ty: syn::Type,

    // Custom field.
    pub default: Option<ArgDefault>,
}

impl ArgCaptured {
    pub fn name(&self) -> Option<String> {
        match self.pat {
            syn::Pat::Ident(ref ident) => Some(ident.ident.to_string()),
            _ => None,
        }
    }

    pub fn is_vararg(&self) -> bool {
        self.vararg_token.is_some()
    }

    pub fn is_optional(&self) -> bool {
        self.optional_token.is_some()
    }
}

impl ArgCaptured {
    pub fn default(&self) -> Option<&syn::Expr> {
        self.default.as_ref().map(|def| &def.value)
    }
}

impl quote::ToTokens for ArgCaptured {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.pat.to_tokens(tokens);
        if let Some(tok) = self.vararg_token.as_ref() {
            tok.to_tokens(tokens);
        }
        if let Some(tok) = self.optional_token.as_ref() {
            tok.to_tokens(tokens);
        }
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);

        if let Some(def) = self.default.as_ref() {
            def.to_tokens(tokens);
        }
    }
}

#[derive(Debug)]
pub enum FnArg {
    SelfRef(syn::ArgSelfRef),
    SelfValue(syn::ArgSelf),
    Captured(ArgCaptured),
    #[allow(dead_code)]
    Inferred(syn::Pat),
    Ignored(syn::Type),
}

impl FnArg {
    fn into_upstream(self) -> syn::FnArg {
        match self {
            FnArg::SelfRef(x) => syn::FnArg::SelfRef(x),
            FnArg::SelfValue(x) => syn::FnArg::SelfValue(x),
            FnArg::Captured(cap) => syn::FnArg::Captured(syn::ArgCaptured {
                pat: cap.pat,
                colon_token: cap.colon_token,
                ty: cap.ty,
            }),
            FnArg::Inferred(x) => syn::FnArg::Inferred(x),
            FnArg::Ignored(x) => syn::FnArg::Ignored(x),
        }
    }

    pub fn captured(&self) -> Option<&ArgCaptured> {
        match self {
            FnArg::Captured(ref cap) => Some(cap),
            _ => None,
        }
    }

    // pub fn name(&self) -> Option<String> {
    //     self.captured().and_then(|cap| cap.name())
    // }
}

impl quote::ToTokens for FnArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FnArg::SelfRef(item) => item.to_tokens(tokens),
            FnArg::SelfValue(item) => item.to_tokens(tokens),
            FnArg::Captured(item) => item.to_tokens(tokens),
            FnArg::Inferred(item) => item.to_tokens(tokens),
            FnArg::Ignored(item) => item.to_tokens(tokens),
        }
    }
}

fn arg_self_ref(input: parse::ParseStream) -> parse::Result<syn::ArgSelfRef> {
    Ok(syn::ArgSelfRef {
        and_token: input.parse()?,
        lifetime: input.parse()?,
        mutability: input.parse()?,
        self_token: input.parse()?,
    })
}

fn arg_self(input: parse::ParseStream) -> parse::Result<syn::ArgSelf> {
    Ok(syn::ArgSelf {
        mutability: input.parse()?,
        self_token: input.parse()?,
    })
}

fn is_vararg_ty(ty: &syn::Type) -> bool {
    match ty {
        // syn::Type::Reference(inner) => {
        //     match *inner.elem {
        //         syn::Type::Slice(_) => { true }
        //         _ => false,
        //     }
        // },
        syn::Type::Path(path) => path.path.segments[0].ident == "Vec",
        _ => false,
    }
}

fn is_option_ty(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(path) => path.path.segments[0].ident == "Option",
        _ => false,
    }
}

fn arg_captured(input: parse::ParseStream) -> parse::Result<ArgCaptured> {
    let arg = ArgCaptured {
        pat: input.parse()?,
        vararg_token: {
            if input.peek(Token![*]) {
                Some(input.parse::<Token![*]>().unwrap())
            } else {
                None
            }
        },
        optional_token: {
            if input.peek(Token![?]) {
                Some(input.parse::<Token![?]>().unwrap())
            } else {
                None
            }
        },
        colon_token: input.parse()?,
        ty: match input.parse::<Token![...]>() {
            Ok(dot3) => {
                let args = vec![
                    TokenTree::Punct(Punct::new('.', Spacing::Joint)),
                    TokenTree::Punct(Punct::new('.', Spacing::Joint)),
                    TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                ];
                let tokens = proc_macro2::TokenStream::from_iter(
                    args.into_iter().zip(&dot3.spans).map(|(mut arg, span)| {
                        arg.set_span(*span);
                        arg
                    }),
                );
                syn::Type::Verbatim(syn::TypeVerbatim { tts: tokens })
            }
            Err(_) => input.parse()?,
        },
        // CUSTOM CODE starts here.
        default: {
            if input.peek(syn::token::Eq) {
                let eq = input.parse()?;

                let (default_token, value) = if input.peek(syn::token::Default) {
                    (
                        Some(input.parse()?),
                        syn::parse_str("Default::default()").unwrap(),
                    )
                } else {
                    (None, input.parse()?)
                };

                Some(ArgDefault {
                    eq,
                    default_token,
                    value,
                })
            } else {
                None
            }
        },
    };
    if arg.is_vararg() {
        if !is_vararg_ty(&arg.ty) {
            let name = arg.name().unwrap_or(format!("UNKNOWN"));
            panic!(
                "Invalid vararg argument {:?}: varargs must have type Vec<_>",
                name
            );
        }
    }
    if arg.is_optional() {
        if !is_option_ty(&arg.ty) {
            let name = arg.name().unwrap_or(format!("UNKNOWN"));
            panic!(
                "Invalid optional argument {:?}: must have type Option<_>",
                name
            );
        }
    }
    if arg.is_optional() && (arg.is_vararg() || arg.default.is_some()) {
        let name = arg.name().unwrap_or(format!("UNKNOWN"));
        panic!(
            "Invalid argument {}: optional arguments may not have a default value or be a vararg",
            name
        );
    }
    if arg.is_vararg() && arg.default.is_some() {
        let name = arg.name().unwrap_or(format!("UNKNOWN"));
        panic!(
            "Invalid argument {}: vararg arguments may not have a default value",
            name
        );
    }
    Ok(arg)
}

impl parse::Parse for FnArg {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        if input.peek(Token![&]) {
            let ahead = input.fork();
            if ahead.call(arg_self_ref).is_ok() && !ahead.peek(Token![:]) {
                return input.call(arg_self_ref).map(FnArg::SelfRef);
            }
        }

        if input.peek(Token![mut]) || input.peek(Token![self]) {
            let ahead = input.fork();
            if ahead.call(arg_self).is_ok() && !ahead.peek(Token![:]) {
                return input.call(arg_self).map(FnArg::SelfValue);
            }
        }

        let ahead = input.fork();
        let err = match ahead.call(arg_captured) {
            Ok(_) => return input.call(arg_captured).map(FnArg::Captured),
            Err(err) => err,
        };

        let ahead = input.fork();
        if ahead.parse::<syn::Type>().is_ok() {
            return input.parse().map(FnArg::Ignored);
        }

        Err(err)
    }
}

#[derive(Debug)]
pub struct FnDecl {
    pub fn_token: syn::token::Fn,
    pub generics: syn::Generics,
    pub paren_token: syn::token::Paren,
    pub inputs: syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    pub variadic: Option<syn::token::Dot3>,
    pub output: syn::ReturnType,
}

#[derive(Debug)]
pub struct ItemFn {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub constness: Option<syn::token::Const>,
    pub unsafety: Option<syn::token::Unsafe>,
    pub asyncness: Option<syn::token::Async>,
    pub abi: Option<syn::Abi>,
    pub ident: syn::Ident,
    pub decl: Box<FnDecl>,
    pub block: Box<syn::Block>,
}

impl ItemFn {
    pub fn into_upstream(self) -> syn::ItemFn {
        let decl = *self.decl;
        syn::ItemFn {
            attrs: self.attrs,
            vis: self.vis,
            constness: self.constness,
            unsafety: self.unsafety,
            asyncness: self.asyncness,
            abi: self.abi,
            ident: self.ident,
            decl: Box::new(syn::FnDecl {
                fn_token: decl.fn_token,
                generics: decl.generics,
                paren_token: decl.paren_token,
                inputs: {
                    let pairs = decl.inputs.into_pairs().map(|pair| match pair {
                        Pair::Punctuated(item, sep) => Pair::Punctuated(item.into_upstream(), sep),
                        Pair::End(item) => {
                            Pair::<syn::FnArg, syn::token::Comma>::End(item.into_upstream())
                        }
                    });
                    Punctuated::from_iter(pairs)
                },
                variadic: decl.variadic,
                output: decl.output,
            }),
            block: self.block,
        }
    }
}

pub struct MacroBody {
    pub fns: Vec<ItemFn>,
}

impl parse::Parse for MacroBody {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let mut fns = Vec::new();
        while !input.is_empty() {
            fns.push(input.parse()?);
        }
        Ok(Self { fns })
    }
}

fn attrs(outer: Vec<syn::Attribute>, inner: Vec<syn::Attribute>) -> Vec<syn::Attribute> {
    let mut attrs = outer;
    attrs.extend(inner);
    attrs
}

impl parse::Parse for ItemFn {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let outer_attrs = input.call(syn::Attribute::parse_outer)?;
        let vis: syn::Visibility = input.parse()?;
        let constness: Option<Token![const]> = input.parse()?;
        let unsafety: Option<Token![unsafe]> = input.parse()?;
        let asyncness: Option<Token![async]> = input.parse()?;
        let abi: Option<syn::Abi> = input.parse()?;
        let fn_token: Token![fn] = input.parse()?;
        let ident: syn::Ident = input.parse()?;
        let generics: syn::Generics = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);
        let inputs = content.parse_terminated(FnArg::parse)?;
        let variadic: Option<Token![...]> = match inputs.last() {
            Some(syn::punctuated::Pair::End(&FnArg::Captured(ArgCaptured {
                ty: syn::Type::Verbatim(syn::TypeVerbatim { ref tts }),
                ..
            }))) => syn::parse2(tts.clone()).ok(),
            _ => None,
        };

        let output: syn::ReturnType = input.parse()?;
        let where_clause: Option<syn::WhereClause> = input.parse()?;

        let content;
        let brace_token = syn::braced!(content in input);
        let inner_attrs = content.call(syn::Attribute::parse_inner)?;
        let stmts = content.call(syn::Block::parse_within)?;

        Ok(ItemFn {
            attrs: attrs(outer_attrs, inner_attrs),
            vis: vis,
            constness: constness,
            unsafety: unsafety,
            asyncness: asyncness,
            abi: abi,
            ident: ident,
            decl: Box::new(FnDecl {
                fn_token: fn_token,
                paren_token: paren_token,
                inputs: inputs,
                output: output,
                variadic: variadic,
                generics: syn::Generics {
                    where_clause: where_clause,
                    ..generics
                },
            }),
            block: Box::new(syn::Block {
                brace_token: brace_token,
                stmts: stmts,
            }),
        })
    }
}

#[derive(Debug)]
pub struct InvokationArg {
    pub name: Option<syn::Ident>,
    pub expr: syn::Expr,
}

impl Parse for InvokationArg {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let name = if input.peek(syn::Ident) && input.peek2(token::Eq) {
            let val = input.parse()?;
            input.parse::<token::Eq>()?;
            Some(val)
        } else {
            None
        };

        let expr = input.parse()?;

        Ok(InvokationArg { name, expr })
    }
}

#[derive(Debug)]
pub struct InvokationInput {
    pub target_fn_path: syn::Path,
    pub arg_definitions: Punctuated<FnArg, token::Comma>,
    pub args: Punctuated<InvokationArg, token::Comma>,
}

impl InvokationInput {
    pub fn args_captured(&self) -> Vec<ArgCaptured> {
        self.arg_definitions
            .iter()
            .filter_map(|item| item.captured().cloned())
            .collect()
    }
}

impl Parse for InvokationInput {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let target_fn_path = input.parse()?;

        let mut inner;
        parenthesized!(inner in input);
        let arg_definitions = Punctuated::parse_terminated(&inner)?;
        input.parse::<token::Semi>()?;
        let args = Punctuated::parse_terminated(input)?;

        Ok(Self {
            target_fn_path,
            arg_definitions,
            args,
        })
    }
}

impl InvokationInput {}
