use stelaro::temp;

fn main() {

temp(r#"
fn main(x: _, y: _, z: _) {
    let x = a();
    let z = x;
}

fn a() {

}
"#.trim().to_string());
}
