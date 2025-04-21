use std::fs;
use std::path::Path;
use std::rc::Rc;

use stelaro::stelaro_session::Session;
use stelaro::stelaro_parse::{new_parser_from_src, parser::Parser};
use stelaro::stelaro_diagnostic::DiagCtxt;
use stelaro::stelaro_common::source_map::SourceMap;


fn create_test_session(src: Rc<String>) -> Session {
    let dcx = DiagCtxt::new(Rc::clone(&src));
    let source_map = Rc::new(SourceMap::new());
    Session::new(dcx, source_map)
}

fn create_test_parser<'a>(sess: &'a Session, src_str: &'a str) -> Parser<'a> {
    new_parser_from_src(sess, src_str.to_owned()).unwrap()
}


fn run_parser_test(path: &Path) {
    let source_code = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("テストファイルを読み込むことができませんでした {:?}: {}", path, e));
    let src = Rc::new(source_code);

    let sess = create_test_session(Rc::clone(&src));
    let mut parser = create_test_parser(&sess, &src);

    let parse_result = parser.parse_stelo().unwrap();

    let snapshot_name = path.file_stem() // ファイル名部分を取得 (Option<OsStr>)
        .and_then(|stem| stem.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| panic!("ファイル名からスナップショット名を生成できませんでした: {:?}", path));

    insta::assert_debug_snapshot!(snapshot_name, parse_result);
}

#[test]
fn test_parser_inputs() {
    insta::glob!("parser_inputs/*.stelo", |path| {
        run_parser_test(path);
    });
}