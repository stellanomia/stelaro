use stelaro::temp;

fn main() {
    temp(r#"
let str = "Hello, World!";
    print "";
fn f() {}
"#.trim().to_string());
}