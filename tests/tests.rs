#![feature(proc_macro_hygiene)]

extern crate fancyargs;

use fancyargs::fancyargs;

fancyargs!(
    fn kw1(a: &str, b: &str = "b", d: &str = "d") -> String {
        format!("{}{}{}", a, b, d)
    }

    fn varargs1(items*: Vec<bool>) -> Vec<bool> {
        items
    }

    fn opt1(a?: Option<bool>) -> bool {
        a.unwrap_or(false)
    }

    fn full1(a: &str, b: &str = "b_default", c?: Option<&str>, d*: Vec<&str>) -> String {
        format!("{}{}{}{}", a, b, c.unwrap_or(""), d.join(""))
    }
);

#[test]
fn test_full1() {
    assert_eq!(full1!("a", "b1", "c", "d1", "d2"), "ab1cd1d2".to_string());
    assert_eq!(
        full1!(a = "a", b = "b1", c = "c", "d1", "d2"),
        "ab1cd1d2".to_string()
    );
    assert_eq!(full1!("a"), "ab_default".to_string());
    assert_eq!(full1!("a", b = "b1"), "ab1".to_string());
    assert_eq!(full1!("a", c = "c"), "ab_defaultc".to_string());
    assert_eq!(
        full1!("a", c = "c", "d1", "d2"),
        "ab_defaultcd1d2".to_string()
    );
}

#[test]
fn test_kw1() {
    assert_eq!(kw1("a", "b", "c"), "abc");
    assert_eq!(kw1!("a"), "abd");
    assert_eq!(kw1!(a = "a"), "abd");
    assert_eq!(kw1!("a", "b1"), "ab1d");
    assert_eq!(kw1!("a", d = "d1", b = "b1"), "ab1d1");
    assert_eq!(kw1!(d = "d1", b = "b1", a = "a1"), "a1b1d1");
}

#[test]
fn test_varargs1() {
    assert_eq!(varargs1!(), vec![]);
    assert_eq!(varargs1!(true), vec![true]);
    assert_eq!(varargs1!(true, false, true), vec![true, false, true]);
}

#[test]
fn test_opt1() {
    assert_eq!(opt1!(), false);
    assert_eq!(opt1!(false), false);
    assert_eq!(opt1!(true), true);
}

mod child {
    #[test]
    fn test_nested() {
        use super::kw1;
        assert_eq!(kw1!("a"), "abd");
    }
}
