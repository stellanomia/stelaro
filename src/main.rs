use stelaro::temp;

fn main() {

temp(r#"
fn main(x: i32, y: i32, z: i32) {
    let x = my_mod::my_mod2::a();
    let y = x + z;
    let z = {
        let a = x * y - z;
        a + my_mod::b()
    };

    let c: char = 'c';

    return z;
}

mod my_mod {
    mod my_mod2 {
        fn a(): i64 {}
    }

    fn b(): i32 {}
}
"#.trim().to_string());
}
