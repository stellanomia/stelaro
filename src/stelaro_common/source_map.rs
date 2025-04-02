use std::io;
use std::rc::Rc;
use std::{cell::RefCell, fs, hash::{DefaultHasher, Hash, Hasher}, path::{Path, PathBuf}};

pub struct SourceMap {
    // TODO: 単一ファイルでコードが評価出来たら複数ファイル対応(files: SourceMapFilesに変更)
    file: RefCell<Rc<SourceFile>>,
    file_loader: FileLoader
}

impl SourceMap {
    pub fn new() -> Self {
        SourceMap {
            file: Default::default(),
            file_loader: FileLoader,
        }
    }

    pub fn load_file(&self, path: &Path) -> io::Result<Rc<SourceFile>> {
        let src = self.file_loader.read_file(path)?;
        let filename = path.to_owned();
        Ok(self.new_source_file(filename, src))
    }

    pub fn new_source_file(&self, path: PathBuf, src: String) -> Rc<SourceFile> {
        // 複数ファイルの場合、filesにこのpathから得たFileIdが存在しないか確認する
        // 存在する場合、filesからRc<SourceFile>を取得し、
        // 存在しない場合、SourceFile::new()し、filesにregisterする。
        Rc::new(SourceFile::new(path, src))
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}


struct FileLoader;

impl FileLoader {
    pub fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }
}

#[derive(Debug, Default)]
pub struct SourceFile {
    pub name: PathBuf,
    pub src: Rc<String>,
    pub file_id: FileId,
}

impl SourceFile {
    pub fn new(name: PathBuf, src: String) -> Self {
        let file_id = FileId::from_file_name(&name);

        SourceFile { name, src: Rc::new(src), file_id }
    }
}

#[derive(Debug, Default)]
pub struct FileId(u32);

impl FileId {
    pub fn from_file_name(filename: &Path) -> FileId {
        let mut hasher = DefaultHasher::new();
        filename.hash(&mut hasher);
        FileId(hasher.finish())
    }
}