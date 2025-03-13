use stelaro::temp;

fn main() {

//     temp(r#"
// let str = "Hello, World!\r\n"
// let ch = 'abcdef;
//     print "";
// fn f() {}
// "#.trim().to_string());

temp(r#"
x = (1 + 2) * 3 == 4 and 5 == 6 or 7 != 8 or 9 == 10 and true
"#.trim().to_string());
}