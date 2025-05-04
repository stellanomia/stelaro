use crate::stelaro_common::span::Span;
use super::DiagCtxt;

use std::{collections::HashSet, marker::PhantomData, ops::Deref, process, rc::Rc};

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

#[allow(unused)]
#[derive(Debug)]
pub struct DiagCtxtInner {
    err_counts: Vec<ErrorEmitted>,
    emitted_diagnostic_codes: HashSet<i32>,
    src: Rc<String>,
}

impl DiagCtxtInner {
    pub fn new(src: Rc<String>) -> Self {
        Self { err_counts: Vec::new(), emitted_diagnostic_codes: HashSet::new(), src }
    }

    pub fn emit_diagnostic(&mut self, diag: DiagInner) -> Option<ErrorEmitted> {

        let mut report = Report::build (
            level_to_ariadne_kind(diag.level),
            diag.span.start..diag.span.end
        );


        if diag.code.is_some() {
            report = report.with_code(diag.code.unwrap());
            self.emitted_diagnostic_codes.insert(diag.code.unwrap());
        }

        if !diag.msg.is_empty() {
            report = report.with_message(diag.msg.join("\n"));
        }

        if !diag.label.is_empty() {
            for (span, msg) in diag.label {
                report = report.with_label(
                    Label::new(span.start..span.end).with_message(msg)
                );
            }
        }

        if !diag.help.is_empty() {
            for msg in diag.help {
                report = report.with_help(msg);
            }
        }

        #[cfg(not(test))] {
            if report.finish().print(Source::from(self.src.as_ref())).is_err() {
                None
            }else {
                self.err_counts.push(ErrorEmitted(()));
                Some(ErrorEmitted(()))
            }
        }

        #[cfg(test)]
        {
            let _ = report.finish();
            self.err_counts.push(ErrorEmitted(()));
            Some(ErrorEmitted(()))
        }
    }
}

#[derive(Debug)]
pub struct ErrorEmitted(());

#[derive(Debug)]
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

#[derive(Debug)]
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
    code: Option<i32>,
    level: Level,
    msg: Vec<String>,
    label: Vec<(Span, String)>,
    help: Vec<String>,
    span: Span,
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


fn level_to_ariadne_kind(level: Level) -> ariadne::ReportKind<'static> {
    match level {
        Level::Error => ariadne::ReportKind::Error,
        Level::Warning => ariadne::ReportKind::Warning,
        Level::Help => ariadne::ReportKind::Advice,
        Level::FatalError => ariadne::ReportKind::Custom("fatal", ariadne::Color::BrightRed),
    }
}