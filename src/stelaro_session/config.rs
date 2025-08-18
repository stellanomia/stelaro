use std::path::PathBuf;

use crate::Args;

pub struct Options {
    pub stelo_name: Option<String>,
    pub working_dir: PathBuf, 
    // pub error_format: ErrorOutputType,
    // pub target_triple: TargetTuple,
}

pub(crate) fn build_session_options(args: &Args) -> Options {
    let stelo_name = args.stelo_name.clone();

    let working_dir = std::env::current_dir().unwrap_or_else(|e| {
        panic!("Current directory is invalid: {e}");
    });

    Options {
        stelo_name,
        working_dir,
    }
}

pub enum Input {
    File(PathBuf),
    Str { name: String, input: String },
}
