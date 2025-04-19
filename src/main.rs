use stelaro::temp;

fn main() {

temp(r#"
fn main() {
    -x * y
}

mod my_mod {
    fn f() => i32 {
        0
    }

    fn a() {
        print("");
    }
}
"#.trim().to_string());
}