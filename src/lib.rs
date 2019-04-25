//! fancyargs turns a regular Rust function into a macro that supports:
//!
//! * keyword arguments
//! * arguments with a default value
//! * optional arguments that are Option::None if not provided
//! * varargs
//! 
//! Currently, Rust does not support keyword or default arguments. 
//! The often-used builder pattern is very boilerplate heavy and often inconvenient.
//! This crate provides a more convenient solution until Rust (hopefully) gains 
//! keyword and default argument support n the future.
//! 
//! # Notes
//!
//! * The original function is preserved and can be used regularily.
//! * Both the macro and the function name need to be in scope.
//! * When calling the macro, positional arguments may not follow keyword arguments.
//! * You can specify multiple functions inside a single macro invocation.
//! 
//!  # Full example
//!
//! ```rust
//! #![feature(proc_macro_hygiene)]
//! 
//! # extern crate fancyargs;
//! 
//! fancyargs::fancyargs!(
//!     fn format_personal_info(
//!         // Every argument can be specified as a regular positional argument or a keyword arg.
//!         // Keyword args may be in any order.
//!         first_name: &str, 
//!         last_name: &str, 
//!         // Optional argument. Will be Some(x) if specified, None otherwise.
//!         // NOTE the ? after the argument name.
//!         middle_name?: Option<&str>, 
//!         // Optional argument with a default value.
//!         is_superuser: bool = false, 
//!         // A vararg argument. 
//!         // Function takes an arbitrary amount of trailing values which will be collected to a Vec.
//!         // NOTE the * after the argument name.
//!         roles*: Vec<&str>,
//!     ) -> String {
//!         format!(
//!             "{}{} {}\nSuperuser: {}\nRoles: {}", 
//!             first_name,
//!             middle_name.unwrap_or(""),
//!             last_name,
//!             if is_superuser { "yes" } else { "no" },
//!             roles.join(", "),
//!         )
//!     }
//! );
// ! 
//! format_personal_info!("John", "Doe");
//! format_personal_info!("John", "Doe", "Franklin", true, "CTO", "CIO");
//! format_personal_info!("John", "Doe", is_superuser = true);
//! format_personal_info!(
//!     is_superuser = false,
//!     first_name = "John", 
//!     last_name = "Doe", 
//!     "Role 1",
//!     "Role 2",
//! );
//! ```
//!
//!  # Limitations
//! 
//!  * Nightly only due to the required [proc_macro_hygiene](https://doc.rust-lang.org/unstable-book/language-features/proc-macro-hygiene.html) feature.
//! * Both the macro and the original function must be in scope.
//!  * Must be a regular macro rather than a attribute, because attribute proc macros require valid Rust syntax.
//!  * Compilation errors are not that great.
//!

extern crate proc_macro;

mod parse;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

fn build_fn(ast: parse::ItemFn) -> TokenStream2 {
    // Verify that function is not a method.
    let last_index = ast.decl.inputs.len() - 1;

    let clean_args = ast
        .decl
        .inputs
        .iter()
        .enumerate()
        .map(|(index, arg)| match arg {
            parse::FnArg::SelfRef(_) | parse::FnArg::SelfValue(_) => {
                panic!("fancyargs!() macro may not be used on methods, only standalone functions");
            }
            parse::FnArg::Captured(ref cap) => {
                if cap.is_vararg() && index != last_index {
                    panic!(
                        "Invalid vararg argument {}: varargs must be the last argument",
                        cap.name().unwrap_or(format!("UNKNOWN"))
                    );
                }
                quote!( #arg )
            }
            _ => {
                panic!("Unsupported function argument");
            }
        })
        .collect::<Vec<_>>();

    let is_pub = match ast.vis {
        syn::Visibility::Public(_) => true,
        _ => false,
    };

    let clean_definition = ast.into_upstream();
    let ident = &clean_definition.ident;

    let macro_export = if is_pub {
        quote!( #[macro_export] )
    } else {
        quote!()
    };

    let output = quote!(
        #clean_definition

        #macro_export
        macro_rules! #ident {
            ( $($input:tt)* ) => {
                fancyargs::invoke!( #ident  (  #( #clean_args ),* ) ;  $($input)* )
            }
        }
    );
    output
}

// pub fn fancyargs(_attrs: TokenStream, input: TokenStream) -> TokenStream {
#[proc_macro]
pub fn fancyargs(input: TokenStream) -> TokenStream {
    let body: parse::MacroBody = match syn::parse(input) {
        Ok(body) => body,
        Err(e) => {
            panic!("Could not parse macro body: \n{}", e);
        }
    };
    let items = body.fns.into_iter().map(build_fn);
    quote!( #( #items )*).into()
}

fn arg_pos_by_name<'a>(
    args: &'a [parse::ArgCaptured],
    name: &str,
) -> Option<(usize, &'a parse::ArgCaptured)> {
    args.iter()
        .enumerate()
        .find(|(_index, arg)| arg.name().map(|argname| argname == name).unwrap_or(false))
        .map(|(index, val)| (index, val))
}

#[doc(hidden)]
#[proc_macro]
pub fn invoke(input: TokenStream) -> TokenStream {
    // Parse arguments.
    let invokation = match syn::parse::<parse::InvokationInput>(input) {
        Ok(ast) => ast,
        Err(e) => panic!("Could not parse arguments: {}", e),
    };
    let arg_definitions = invokation.args_captured();
    let have_vararg = arg_definitions
        .iter()
        .last()
        .map(|arg| arg.is_vararg())
        .unwrap_or(false);

    let mut args: Vec<Option<TokenStream2>> = Vec::new();
    for _ in &arg_definitions {
        args.push(None);
    }
    let mut varargs: Vec<syn::Expr> = Vec::new();

    let mut reached_keyword_args = false;

    for (index, arg) in invokation.args.into_iter().enumerate() {
        let (arg_index, arg_decl) = match arg.name {
            Some(name) => {
                match arg_pos_by_name(&arg_definitions, &name.to_string()) {
                    Some(x) => {
                        // Check if the keyword argument was already specified.
                        if args.get(x.0).map(|x| x.is_some()).unwrap_or(false) {
                            panic!("Duplicate keyword argument '{}'", name);
                        }
                        reached_keyword_args = true;
                        x
                    }
                    None => {
                        panic!("Unknown keyword argument '{}'", name);
                    }
                }
            }
            None => {
                if reached_keyword_args {
                    if have_vararg {
                        varargs.push(arg.expr);
                        continue;
                    } else {
                        panic!("Invalid argument number {}: positional arguments may not follow after keyword arguments", index);
                    }
                } else {
                    if have_vararg && index >= arg_definitions.len() - 1 {
                        varargs.push(arg.expr);
                        continue;
                    } else if index >= arg_definitions.len() {
                        panic!("Invalid positional argument number {}:  function only takes {} arguments", index + 1, arg_definitions.len());
                    } else {
                        (index, &arg_definitions[index])
                    }
                }
            }
        };

        let expr = if arg_decl.is_optional() {
            let expr = arg.expr;
            quote!( Some(#expr) )
        } else {
            let expr = arg.expr;
            quote!( #expr )
        };
        args[arg_index] = Some(expr);
    }

    let mut finished_args = Vec::new();

    let varargs = &varargs;
    for (index, def) in arg_definitions.iter().enumerate() {
        if let Some(Some(value)) = args.get(index) {
            // Already have an argument.
            finished_args.push(value.clone());
        } else if def.is_vararg() {
            finished_args.push(quote!(vec![ #( #varargs ),* ]));
        } else if def.is_optional() {
            finished_args.push(quote!(None));
        } else {
            if let Some(default_expr) = def.default() {
                finished_args.push(quote!( #default_expr ));
            } else {
                panic!(
                    "Missing required argument '{}'",
                    def.name().unwrap_or("??".into())
                );
            }
        }
    }

    let path = &invokation.target_fn_path;

    quote!(
        #path ( #( #finished_args ),* )
    )
    .into()
}
