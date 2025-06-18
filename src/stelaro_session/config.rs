use std::path::PathBuf;


pub enum Input {
    File(PathBuf),
    Str {
        name: String,
        input: String,
    },
}

pub enum OutFileName {
    Real(PathBuf),
    Stdout,
}
