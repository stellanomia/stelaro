use stelaro::temp;

fn main() {

temp(r#"
fn main(z: i32) => i32 {
    let x: std::i32 = if true {
        let y = 2;
        { y + 123 }
    } else {
        3
    };

    return f(x+1);
}

fn f(x: i32) => i32 {
    return x * x;
}

"#.trim().to_string());
}