# fancyargs

[![Crates.io](https://img.shields.io/crates/v/fancyargs.svg)](https://crates.io/crates/fancyargs)
[![Docs](https://docs.rs/fancyargs/badge.svg)](https://docs.rs/fancyargs)

fancyargs turns a regular Rust function into a macro that supports:

* keyword arguments
* arguments with a default value
* optional arguments that are Option::None if not provided
* varargs

Currently, Rust does not support keyword or default arguments.
The often-used builder pattern is very boilerplate heavy and often inconvenient.
This crate provides a more convenient solution until Rust (hopefully) gains
keyword and default argument support n the future.

## Notes

* The original function is preserved and can be used regularily.
* Both the macro and the function name need to be in scope.
* When calling the macro, positional arguments may not follow keyword arguments.
* You can specify multiple functions inside a single macro invocation.

 # Full example

```rust
#![feature(proc_macro_hygiene)]


fancyargs::fancyargs!(
    fn format_personal_info(
        // Every argument can be specified as a regular positional argument or a keyword arg.
        // Keyword args may be in any order.
        first_name: &str,
        last_name: &str,
        // Optional argument. Will be Some(x) if specified, None otherwise.
        // NOTE the ? after the argument name.
        middle_name?: Option<&str>,
        // Optional argument with a default value.
        is_superuser: bool = false,
        // A vararg argument.
        // Function takes an arbitrary amount of trailing values which will be collected to a Vec.
        // NOTE the * after the argument name.
        roles*: Vec<&str>,
    ) -> String {
        format!(
            "{}{} {}\nSuperuser: {}\nRoles: {}",
            first_name,
            middle_name.unwrap_or(""),
            last_name,
            if is_superuser { "yes" } else { "no" },
            roles.join(", "),
        )
    }
);

## License

MIT. See [LICENSE.txt](./blob/master/LICENSE.txt).
