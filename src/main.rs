use stelaro::temp;

fn main() {

//     temp(r#"
// let str = "Hello, World!\r\n"
// let ch = 'abcdef;
//     print "";
// fn f() {}
// "#.trim().to_string());

temp(r#"
a = (b = (c = d * (e = f + (g = h + (i = j * k))))) + l;
"#.trim().to_string());
}