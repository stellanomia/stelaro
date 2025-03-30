use stelaro::temp;

fn main() {

temp(r#"
fn main() {
    -x * y
}
"#.trim().to_string());
}