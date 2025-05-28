use stelaro::temp;

fn main() {

temp(r#"
fn main() {
    -x * y
}

fn main() {}

mod my_mod {
    fn f(): i32 {
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

mod my_mod {
    fn a() {

    }
}
"#.trim().to_string());
}
