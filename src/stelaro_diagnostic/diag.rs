use crate::stelaro_common::{Span, Hash128, StableHasher};
use super::{emitter::DynEmitter, DiagCtxt};

use std::{collections::HashSet, hash::{Hasher, Hash}, marker::PhantomData, ops::Deref, process};


/// 診断メッセージの出力保証を表すトレイト
pub trait EmissionGuarantee: Sized {
    /// 出力操作の結果型
    type EmitResult = Self;

    /// 診断を出力し、保証トークンを生成
    fn emit_producing_guarantee(diag: Diag<'_, Self>) -> Self::EmitResult;
}

#[derive(Debug)]
pub enum FatalError {}

impl EmissionGuarantee for ErrorEmitted {
    fn emit_producing_guarantee(diag: Diag<'_, Self>) -> Self::EmitResult {
        diag.emit_producing_error_guaranteed()
    }
}

impl EmissionGuarantee for () {
    fn emit_producing_guarantee(diag: Diag<'_, Self>) -> Self::EmitResult {
        diag.emit_producing_nothing();
    }
}

/// 致命的エラーは発散する (プログラムを終了)
impl EmissionGuarantee for FatalError {
    type EmitResult = !;

    fn emit_producing_guarantee(diag: Diag<'_, Self>) -> ! {
        diag.emit_without_guarantee();
        std::process::exit(1)
    }
}

pub struct DiagCtxtInner {
    /// 発行されたエラーを保持する
    pub errors: Vec<ErrorEmitted>,

    /// 送信された診断のハッシュ値を保持する。
    pub emitted_diagnostics: HashSet<Hash128>,

    /// 送信された診断のエラーコード。
    pub emitted_diagnostic_codes: HashSet<i32>,

    /// 重複が排除され、実際に表示されたエラーの個数を表す。
    pub emitted_err_count: usize,

    /// 重複が排除され、実際に表示された警告の個数を表す。
    pub emitted_warn_count: usize,

    pub emitter: Box<DynEmitter>,
}

impl DiagCtxtInner {
    pub fn new(emitter: Box<DynEmitter>) -> Self {
        Self {
            errors: Vec::new(),
            emitted_diagnostics: HashSet::new(),
            emitted_diagnostic_codes: HashSet::new(),
            emitted_err_count: 0,
            emitted_warn_count: 0,
            emitter
        }
    }

    pub fn emit_diagnostic(&mut self, diag: DiagInner) -> Option<ErrorEmitted> {

        if let Some(code) = diag.code {
            self.emitted_diagnostic_codes.insert(code);
        }

        let already_emitted = {
            let mut hasher = StableHasher::new();
            diag.hash(&mut hasher);
            let diagnostic_hash = hasher.finish();
            !self.emitted_diagnostics.insert(diagnostic_hash)
        };


        let is_error = diag.is_error();

        if !already_emitted {
            if is_error {
                self.emitted_err_count += 1;
            } else if matches!(diag.level, Level::Warning) {
                self.emitted_warn_count += 1;
            }

            self.emitter.emit_diagnostic(diag);
        }


        if is_error {
            let guarantee = ErrorEmitted(());
            self.errors.push(guarantee);
            Some(guarantee)
        } else {
            None
        }

    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct ErrorEmitted(());

#[derive(Clone, Copy)]
pub struct DiagCtxtHandle<'a> {
    pub dcx: &'a DiagCtxt,
}

impl Deref for DiagCtxtHandle<'_> {
    type Target = DiagCtxt;

    fn deref(&self) -> &Self::Target {
        self.dcx
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    FatalError,
    Error,
    Warning,
    Help,
}

impl<'a> DiagCtxtHandle<'a> {
    pub fn struct_err(self, span: Span) -> Diag<'a, ErrorEmitted> {
        Diag::new(self, span, Level::Error)
    }

    pub fn struct_warn(self, span: Span) -> Diag<'a, ErrorEmitted> {
        Diag::new(self, span, Level::Warning)
    }

    pub fn struct_help(self, span: Span) -> Diag<'a, ErrorEmitted> {
        Diag::new(self, span, Level::Help)
    }

    pub fn struct_fatal(self, span: Span) -> Diag<'a, FatalError> {
        Diag::new(self, span, Level::Help)
    }

    pub fn emit_fatal(self, msg: String) -> ! {
        eprintln!("\x1b[31merror:\x1b[0m {}", msg);
        process::exit(1)
    }

    fn emit_diagnostic(&self, diag: DiagInner) -> Option<ErrorEmitted> {
        self.inner.borrow_mut().emit_diagnostic(diag)
    }

    pub fn has_err_code(self, code: i32) -> bool {
        self.inner.borrow().emitted_diagnostic_codes.contains(&code)
    }
}

pub struct Diag<'dcx, G:EmissionGuarantee> {
    dcx: DiagCtxtHandle<'dcx>,
    diag: Option<Box<DiagInner>>,
    _marker: PhantomData<G>
}

impl<'a, G:EmissionGuarantee> Diag<'a, G> {
    pub fn new(dcx: DiagCtxtHandle<'a>, span: Span, level: Level) -> Diag<'a, G> {
        Diag {
            dcx,
            diag: Some(Box::new(DiagInner::new(level, span))),
            _marker: PhantomData,
        }
    }

    pub fn take_diag(&mut self) -> DiagInner {
        *self.diag.take().unwrap()
    }

    fn emit_without_guarantee(mut self) {
        let diag = self.take_diag();
        self.dcx.emit_diagnostic(diag);
    }

    fn emit_producing_error_guaranteed(mut self) -> ErrorEmitted {
        let diag = self.take_diag();

        let guar = self.dcx.emit_diagnostic(diag);
        // エラーを発行したのにも関わらず、送信保証が得られないならパニックすべき
        guar.unwrap()
    }

    pub fn set_code(&mut self, code: i32) {
        self.diag.as_deref_mut().unwrap().code = Some(code);
    }

    pub fn set_message(&mut self, msg: String) {
        self.diag.as_deref_mut().unwrap().msg.push(msg);
    }

    pub fn set_label(&mut self, span: Span, msg: String) {
        self.diag.as_deref_mut().unwrap().label.push((span, msg));
    }

    pub fn set_help(&mut self, msg: String) {
        self.diag.as_deref_mut().unwrap().help.push(msg);
    }

    pub fn emit(self) -> G::EmitResult {
        G::emit_producing_guarantee(self)
    }
}

#[derive(Debug)]
pub struct DiagInner {
    pub code: Option<i32>,
    pub level: Level,
    pub msg: Vec<String>,
    pub label: Vec<(Span, String)>,
    pub help: Vec<String>,
    pub span: Span,
}

impl DiagInner {
    pub fn new(level: Level, span: Span) -> DiagInner {
        DiagInner {
            level,
            msg: Vec::new(),
            label: Vec::new(),
            help: Vec::with_capacity(0),
            span,
            code: None,
        }
    }

        pub fn is_error(&self) -> bool {
        match self.level {
            Level::FatalError | Level::Error => true,
            Level::Warning
            | Level::Help => false,
        }
    }

    /// Hash, PartialEq の実装に使用するためのフィールド取得メソッド
    fn keys(
        &self,
    ) -> (
        &Level,
        &Vec<String>,
        &Option<i32>,
        &Span,
        &Vec<(Span, String)>,
        &Vec<String>,
    ) {
        (
            &self.level,
            &self.msg,
            &self.code,
            &self.span,
            &self.label,
            &self.help,
        )
    }
}

impl Hash for DiagInner {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.keys().hash(state);
    }
}

impl PartialEq for DiagInner {
    fn eq(&self, other: &Self) -> bool {
        self.keys() == other.keys()
    }
}
