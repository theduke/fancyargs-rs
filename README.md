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
keyword and default argument support in the future.

### Usage Notes

* The original function is preserved and can be used regularily.
* **Both the macro and the function name need to be in scope.**
* When calling the macro, positional arguments may not follow keyword arguments.
* You can specify multiple functions inside a single macro invocation.

 ## Full example

```rust
// Nightly and the `proc_macro_hygiene` feature areis required because proc
// macros can't yet produce expressions on stable.
#![feature(proc_macro_hygiene)]


fancyargs::fancyargs!(
    pub fn format_personal_info(
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

fn main() {
  format_personal_info!("John", "Doe");
  format_personal_info!("John", "Doe", "Franklin", true, "CTO", "CIO");
  format_personal_info!("John", "Doe", is_superuser = true);
  format_personal_info!(
    is_superuser = false,
    first_name = "John",
    last_name = "Doe",
    "Role 1",
    "Role 2",
  );
}
```

 ## Limitations

 * Nightly only due to the required [proc_macro_hygiene](https://doc.rust-lang.org/unstable-book/language-features/proc-macro-hygiene.html) feature.
* Both the macro and the original function must be in scope.
 * Must be a regular macro rather than a attribute, because attribute proc macros require valid Rust syntax.
 * Compilation errors are not that great.


## License

MIT. See [LICENSE.txt](./LICENSE.txt).
