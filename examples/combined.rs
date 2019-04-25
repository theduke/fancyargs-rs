#![feature(proc_macro_hygiene)]

extern crate fancyargs;

fancyargs::fancyargs!(

    fn format_personal_info(
        first_name: &str, 
        last_name: &str, 
        // Optional argument. Will be Some(x) if specified, None otherwise.
        middle_name?: Option<&str>, 
        // Optional argument with a default value.
        is_superuser: bool = false, 
        // A vararg argument. 
        // Function takes an arbitrary amount of trailing values which will be collected to a Vec.
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
    format_personal_info!("John", "Doe", "Franklin", true, "CTO", "CIO");
    format_personal_info!("John", "Doe", is_superuser = true);
    format_personal_info!("John", middle_name = "Franklin", last_name = "Doe");
    format_personal_info!(
        first_name = "John", 
        last_name = "Doe", 
        is_superuser = false,
        "Role 1",
        "Role 2",
    );
}
