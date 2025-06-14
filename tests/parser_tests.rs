use std::fs;
use std::path::Path;
use std::rc::Rc;

use insta::{assert_debug_snapshot, with_settings};
use stelaro::stelaro_common::create_default_session_globals_then;
use stelaro::stelaro_session::ParseSess;
use stelaro::stelaro_parse::new_parser_from_source_str;
use stelaro::stelaro_diagnostic::{DiagCtxt, SilentEmitter};
use stelaro::stelaro_common::source_map::SourceMap;

fn create_test_context() -> ParseSess {
    let source_map = Rc::new(SourceMap::new());
    let emitter = SilentEmitter::new();
    let dcx = DiagCtxt::new(Box::new(emitter));
    ParseSess::with_dcx(dcx, source_map)
}

fn run_parser_test(path: &Path) {
    let source_code = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("テストファイルを読み込むことができませんでした {path:?}: {e}"));

    let psess = create_test_context();
    let mut parser = new_parser_from_source_str(
        &psess,
        "parser_tests".into(),
        source_code,
    ).unwrap();

    let parse_result = parser.parse_stelo().unwrap();

    let snapshot_name = path.file_stem() // ファイル名部分を取得 (Option<OsStr>)
        .and_then(|stem| stem.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| panic!("ファイル名からスナップショット名を生成できませんでした: {path:?}"));

    with_settings!(
        {
            filters => vec![
                (r"Symbol\(\s*\d+,\s*\)", "Symbol([ID])"),
            ]
        },
        {
            assert_debug_snapshot!(snapshot_name, parse_result);
        }
    );
}

#[test]
fn test_parser_inputs() {
    insta::glob!("parser_inputs/*.stelo", |path| {
        create_default_session_globals_then(|| {
            run_parser_test(path);
        })
    });
}