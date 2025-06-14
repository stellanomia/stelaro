use stelaro::temp;

fn main() {

temp(r#"
fn main(x: _, y: _, z: _) {
    let x = my_mod::my_mod2::a();
    let z = x;
}

mod my_mod {
    mod my_mod2 {
        fn a() {}
    }
}
"#.trim().to_string());
}
