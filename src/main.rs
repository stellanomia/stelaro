use stelaro::temp;

fn main() {

//     temp(r#"
// let str = "Hello, World!\r\n"
// let ch = 'abcdef;
//     print "";
// fn f() {}
// "#.trim().to_string());

temp(r#"
let x = if true {
    let y = 2;
    y
} else {
    3
};
"#.trim().to_string());

temp(r#"
let x = while true {
    let x = 0;
}
"#.trim().to_string());
}