use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::{
    fs,
    hash::Hash,
    path::{Path, PathBuf},
};

use super::stable_hasher::HashStable;
use super::{Hash128, SESSION_GLOBALS, Span, StableHasher};

pub struct SourceMap {
    // TODO: 単一ファイルで codegen が可能になったら複数ファイル対応(files: SourceMapFilesに変更)
    pub file: RefCell<Rc<SourceFile>>,
    file_loader: Box<dyn FileLoader + Sync + Send>,
}

impl SourceMap {
    pub fn new() -> Self {
        SourceMap {
            file: Default::default(),
            file_loader: Box::new(RealFileLoader),
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
        let file = Rc::new(SourceFile::new(path, src));
        *self.file.borrow_mut() = Rc::clone(&file);
        file
    }

    pub fn with_inputs(SourceMapInputs { file_loader }: SourceMapInputs) -> SourceMap {
        SourceMap {
            file: Default::default(),
            file_loader,
        }
    }

    pub fn truncate_span_to_item_header(&self, span: Span) -> Span {
        self.span_until_char(span, '{')
    }

    pub fn span_until_char(&self, span: Span, c: char) -> Span {
        let file = self.file.borrow();
        let snippet = file.src[span.as_range_usize()]
            .split(c)
            .next()
            .unwrap_or("")
            .trim_end();

        if !snippet.is_empty() && !snippet.contains('\n') {
            (span.start, span.start + snippet.len() as u32).into()
        } else {
            span
        }
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

/// ファイルシステム操作の抽象。
pub trait FileLoader {
    /// ファイルが存在するかどうかを問い合わせる。
    fn file_exists(&self, path: &Path) -> bool;

    /// UTF-8 ファイルの内容をメモリに読み込む。
    /// ソースファイルを正規化するため、サイズ変更が必要になる場合があり、
    /// この関数は String を返さなければならない。
    fn read_file(&self, path: &Path) -> io::Result<String>;
}

/// `std::fs` を使用して実際のファイルを読み込む `FileLoader`
pub struct RealFileLoader;

impl FileLoader for RealFileLoader {
    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        // path のメタデータが取得でき、かつファイルサイズが SourceFile::MAX_FILE_SIZE より大きい場合
        if path
            .metadata()
            .is_ok_and(|metadata| metadata.len() > SourceFile::MAX_FILE_SIZE.into())
        {
            return Err(io::Error::other(format!(
                "{} バイトより大きいテキストファイルはサポートされていません",
                SourceFile::MAX_FILE_SIZE
            )));
        }
        fs::read_to_string(path)
    }
}

#[derive(Debug, Default)]
pub struct SourceFile {
    pub name: PathBuf,
    pub src: Rc<String>,
    pub file_id: SourceFileId,
}

impl SourceFile {
    const MAX_FILE_SIZE: u32 = u32::MAX - 1;

    pub fn new(name: PathBuf, src: String) -> Self {
        let file_id = SourceFileId::from_file_name(&name);

        SourceFile {
            name,
            src: Rc::new(src),
            file_id,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceFileId(pub Hash128);

impl SourceFileId {
    pub fn from_file_name(filename: &Path) -> Self {
        let mut hasher = StableHasher::new();
        filename.hash(&mut hasher);
        SourceFileId(hasher.finish())
    }
}

impl HashStable for SourceFileId {
    fn hash_stable(&self, hasher: &mut StableHasher) {
        self.0.hash_stable(hasher);
    }
}

pub fn get_source_map() -> Option<Rc<SourceMap>> {
    SESSION_GLOBALS.with(|session_globals| session_globals.source_map.clone())
}

pub struct SourceMapInputs {
    pub file_loader: Box<dyn FileLoader + Send + Sync>,
}
