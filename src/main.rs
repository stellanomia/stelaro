use stelaro::temp;

fn main() {

//     temp(r#"
// let str = "Hello, World!\r\n"
// let ch = 'abcdef;
//     print "";
// fn f() {}
// "#.trim().to_string());

temp(r#"
fn main(z: i32) => i32 {
    let x: std::i32 = if true {
        let y = 2;
        { y + 123 }
    } else {
        3
    };
}
"#.trim().to_string());
}