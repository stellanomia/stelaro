use crate::stelaro_common::{span::Span, Hash128};
use super::{emitter::DynEmitter, DiagCtxt};

use std::{collections::HashSet, marker::PhantomData, ops::Deref, process};

#[allow(unused)]
use ariadne::{Label, Report, Source};



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

/// 致命的エラーは発散する (プログラムを終了)
impl EmissionGuarantee for FatalError {
    type EmitResult = !;

    fn emit_producing_guarantee(diag: Diag<'_, Self>) -> ! {
        diag.emit_producing_nothing();
        std::process::exit(1)
    }
}

pub struct DiagCtxtInner {
    pub err_counts: Vec<ErrorEmitted>,
    pub emitted_diagnostics: HashSet<Hash128>,
    pub emitter: Box<DynEmitter>,
}

impl DiagCtxtInner {
    pub fn new(emitter: Box<DynEmitter>) -> Self {
        Self { err_counts: Vec::new(), emitted_diagnostics: HashSet::new(), emitter }
    }

    pub fn emit_diagnostic(&mut self, diag: DiagInner) -> Option<ErrorEmitted> {
        let already_emitted = {
            let mut hasher = StableHasher::new();
            diag.hash(&mut hasher);
            let diagnostic_hash = hasher.finish();
            !self.emitted_diagnostics.insert(diagnostic_hash)
        };
        self.emitter.emit_diagnostic(diag);

        Some(ErrorEmitted(()))
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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
        println!("\x1b[31merror:\x1b[0m {}", msg);
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

    fn emit_producing_nothing(mut self) {
        let diag = self.take_diag();
        self.dcx.emit_diagnostic(diag);
    }

    fn emit_producing_error_guaranteed(mut self) -> ErrorEmitted {
        let diag = self.take_diag();

        let guar = self.dcx.emit_diagnostic(diag);
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
}
