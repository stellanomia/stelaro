pub mod interface;
pub mod passes;

pub use interface::Config;

use crate::stelaro_common::{create_session_globals_then, RealFileLoader, SourceMapInputs};
use crate::stelaro_session::session::{build_session, CompilerPaths};
use crate::stelaro_session::Session;

pub fn run_compiler<R>(config: Config, f: impl FnOnce(&Session) -> R) -> R {
    let file_loader = Box::new(RealFileLoader);

    create_session_globals_then(Some(SourceMapInputs { file_loader }),|| {
        let sess = build_session(
            config.opts,
            CompilerPaths {
                input: config.input,
                output_dir: config.output_dir,
                output_file: config.output_file,
                temps_dir: dirs::cache_dir(),
            }
        );

        // `f` からの脱出パスは2つある。
        // - 正常終了。
        // - パニック。例： `abort_if_errors` や致命的なエラーによって引き起こされる場合。
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&sess)));

        // エラー診断が出力された場合、この関数の戻り値は `R` であり
        // `Result<R, E>` ではないため、直接エラーを返すことはできない。
        // しかし、エラーの存在を呼び出し元に伝える必要がある。さもないと、
        // 呼び出し元はエラーが発生しなかったと誤解し、終了コード0を
        // 返してしまうかもしれない。そのため、`f` がパニックを起こした場合と
        // 同様に、代わりにパニックさせる。
        if res.is_ok() {
            sess.dcx().abort_if_errors();
        }

        let res = match res {
            Ok(res) => res,
            Err(err) => std::panic::resume_unwind(err),
        };

        drop(sess);

        res
    })
}