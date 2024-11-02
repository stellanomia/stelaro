use compiler::eval;


fn main() {
    eval(r#"
let str = "Hello, World!";
    print "";
fn f() {}
"#.trim());
}