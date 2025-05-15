use crate::stelaro_common::SourceMap;
use super::diag::{DiagInner, Level};

use std::rc::Rc;
use ariadne::{Label, Report, Source};


pub type DynEmitter = dyn Emitter;

pub trait Emitter {
    /// 構築された diagnostic を送信する
    fn emit_diagnostic(&mut self, diag: DiagInner);

    fn source_map(&self) -> Option<&SourceMap>;
}

/// `ariadne` クレートを用いた診断を担う
pub struct AriadneEmitter {
    source_map: Option<Rc<SourceMap>>,
}

impl AriadneEmitter {
    pub fn new(source_map: Rc<SourceMap>) -> Self {
        AriadneEmitter { source_map: Some(source_map) }
    }
}

impl Emitter for AriadneEmitter {
    fn emit_diagnostic(&mut self, diag: DiagInner) {
        // TODO: 複数ファイル対応時には、Spanに対して入力ソース、ファイル名を得られるように変更する
        let file = &self.source_map.as_ref().unwrap().file;
        let name = file.name.file_name()
            .map(|name| name.to_str())
            .unwrap_or(file.name.to_str())
            .unwrap_or("unknown");

        let mut report = Report::build(
            level_to_ariadne_kind(diag.level),
            (name, diag.span.as_range_usize())
        );

        if !diag.msg.is_empty() {
            report = report.with_message(diag.msg.join("\n"));
        }

        if !diag.label.is_empty() {
            for (span, msg) in diag.label {
                report = report.with_label(
                    Label::new((name, span.as_range_usize())).with_message(msg)
                );
            }
        }

        if !diag.help.is_empty() {
            for msg in diag.help {
                report = report.with_help(msg);
            }
        }


        report.finish()
            .print((
                name,
                Source::from(
                    file.src.as_ref()
                )
            ))
            .unwrap();
    }

    fn source_map(&self) -> Option<&SourceMap> {
        self.source_map.as_deref()
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

/// テストなどに用いられ、診断が表示されるべきでない場合に用いる
pub struct SilentEmitter;

impl Emitter for SilentEmitter {
    fn source_map(&self) -> Option<&SourceMap> {
        None
    }

    fn emit_diagnostic(&mut self, diag: DiagInner) {
        drop(diag);
    }
}