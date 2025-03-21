use stelaro::temp;

fn main() {

//     temp(r#"
// let str = "Hello, World!\r\n"
// let ch = 'abcdef;
//     print "";
// fn f() {}
// "#.trim().to_string());

temp(r#"
f(g(1, 2, 3, t(4+5) * t(6-7)))
"#.trim().to_string());
}