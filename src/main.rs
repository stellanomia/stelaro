use stelaro::temp;

fn main() {

temp(r#"
fn main(x: _, y: _, z: _) {
    let x = 0;
    let z = x;
}
"#.trim().to_string());
}
