use std::path::PathBuf;

use crate::{
    stelaro_common::FileLoader,
    stelaro_session::config::{self, Input},
};

/// コンパイラのコンフィグ
pub struct Config {
    pub opts: config::Options,

    pub input: Input,
    pub output_dir: Option<PathBuf>,
    pub output_file: Option<PathBuf>,

    pub file_loader: Option<Box<dyn FileLoader + Send + Sync>>,
}
