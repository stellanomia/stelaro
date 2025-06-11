use stelaro::temp;

fn main() {

temp(r#"
fn main(x: _, y: _, z: _) {
    let x;
    let z;
}
"#.trim().to_string());
}
