use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub struct SourceMap {
    // TODO: 複数ファイル対応(filesに変更)
    file: RefCell<Rc<SourceFile>>,
    file_loader: Box<FileLoader>
}

impl SourceMap {
    pub fn new() -> Self {
        todo!()
    }

    pub fn load_file() -> Rc<SourceFile> {
        todo!() 
    }
}

pub struct SourceFile {
    pub name: PathBuf,
    pub src: Rc<String>,
}

struct FileLoader;

impl FileLoader {

}