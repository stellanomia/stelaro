use stelaro::temp;

fn main() {

temp(r#"
fn main(x: _, y: _, z: _) {
    let x;
    let z;
}

mod my_mod {
    fn f(a: _, b:_): _ {
        let a;
        let x;
        let b;
    }

    fn ff() {
        let n;
    }
}
"#.trim().to_string());
}
