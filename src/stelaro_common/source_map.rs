// FIXME: 全体的な SourceMap の実装改善

use std::io;
use std::rc::Rc;
use std::{fs, hash::Hash, path::{Path, PathBuf}};

use super::stable_hasher::HashStable;
use super::{Hash128, Span, StableHasher};

#[allow(unused)]
pub struct SourceMap {
    // TODO: 単一ファイルで codegen が可能になったら複数ファイル対応(files: SourceMapFilesに変更)
    file: Rc<SourceFile>,
    file_loader: FileLoader,
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

    pub fn truncate_span_to_item_header(&self, span: Span) -> Span {
        self.span_until_char(span, '{')
    }

    // FIXME: 複数ファイルの実装になると完全に破綻する
    pub fn span_until_char(&self, span: Span, c: char) -> Span {
        let snippet = &self.file
            .src[span.as_range_usize()]
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


struct FileLoader;

#[allow(unused)]
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
    pub file_id: SourceFileId,
}

impl SourceFile {
    pub fn new(name: PathBuf, src: String) -> Self {
        let file_id = SourceFileId::from_file_name(&name);

        SourceFile { name, src: Rc::new(src), file_id }
    }
}

#[derive(Debug, Default)]
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