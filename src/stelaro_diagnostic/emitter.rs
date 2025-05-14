use std::rc::Rc;
#[allow(unused)]
use ariadne::{Label, Report, Source};

use crate::stelaro_common::SourceMap;

use super::diag::{DiagInner, Level};


pub type DynEmitter = dyn Emitter;

pub trait Emitter {
    /// 構築された diagnostic を送信する
    fn emit_diagnostic(&mut self, diag: DiagInner);

    fn source_map(&self) -> Option<&SourceMap>;
}

pub struct AriadneEmitter {
    source_map: Option<Rc<SourceMap>>,
}

impl Emitter for AriadneEmitter {
    fn emit_diagnostic(&mut self, diag: DiagInner) {
        let mut report = Report::build (
            level_to_ariadne_kind(diag.level),
            diag.span.as_range_usize()
        );

        if !diag.msg.is_empty() {
            report = report.with_message(diag.msg.join("\n"));
        }

        if !diag.label.is_empty() {
            for (span, msg) in diag.label {
                report = report.with_label(
                    Label::new(span.as_range_usize()).with_message(msg)
                );
            }
        }

        if !diag.help.is_empty() {
            for msg in diag.help {
                report = report.with_help(msg);
            }
        }

        #[cfg(not(test))]
        {
            report.finish()
                .print(
                    Source::from(
                        self.source_map.as_ref().unwrap().file.src.as_ref()
                    )
                );
        }

        #[cfg(test)]
        {
            let _ = report.finish();
        }
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