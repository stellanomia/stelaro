use stelaro::temp;

fn main() {

temp(r#"
fn main(x: _, x: _) {
    -x * y
}

mod my_mod {
    fn f(): i32 {
        let x = if 123 < 456 {
            789
        } else {
            0
        };

        x
    }

    fn a(x: i32) {
        print("Hello");
    }
}
"#.trim().to_string());
}
