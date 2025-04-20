use std::fs;
use std::path::{Path, PathBuf};
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

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let relative_path = path.strip_prefix(&project_root)
        .unwrap_or_else(|_| {
            eprintln!("Warning: パス '{:?}' からプロジェクトルートのプレフィックスを削除できませんでした。元のパスを使用します。", path);
            path
        });

    let snapshot_name = relative_path.strip_prefix("tests/parser_inputs/")
        .unwrap_or_else(|_| {
            eprintln!("Warning: パス '{:?}' は 'tests/parser_inputs/' で始まりません。完全な相対パスを使用します。", relative_path);
            relative_path
        })
        .to_str().expect("パスに非UTF8文字が含まれています")
        .trim_end_matches(".stelo") // 拡張子を除去
        .to_string();

    insta::assert_debug_snapshot!(snapshot_name, parse_result);
}

#[test]
fn test_parser_inputs() {
    // `tests/parser_inputs/` ディレクトリ以下の全ての `.stelo` ファイルを検索
    insta::glob!("parser_inputs/*.stelo", |path| {
        run_parser_test(path);
    });
}