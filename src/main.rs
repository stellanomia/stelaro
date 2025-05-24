use stelaro::temp;

fn main() {

temp(r#"
fn main() {
    -x * y
}

mod my_mod {
    fn f() : i32 {
        let x = if 123 < 456 {
            789
        } else {
            0
        };

        x
    }

    fn a() {
        print("Hello");
    }
}
"#.trim().to_string());
}
