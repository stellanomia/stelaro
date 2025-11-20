use crate::stelaro_common::DelayedMap;
use crate::stelaro_sir_typecheck::infer::InferCtxt;
use crate::stelaro_ty::Ty;
use crate::stelaro_ty::fold::TypeFolder;


/// 型変数を、現時点で判明している情報で自分勝手に (opportunistically) 解決するフォルダー。
///
/// このフォルダーはいつでも安全に使用できます。型変数が既に何らかの型に統合されていれば、
/// その型に深く (再帰的に) 置き換えます。もし変数がまだ解決されていなければ、
/// その変数のルート（代表となる変数）に置き換えます。
/// この操作は決して失敗しません。
pub struct OpportunisticVarResolver<'a, 'tcx> {
    infcx: &'a InferCtxt<'tcx>,
    /// フォルダー自身は可変な状態を持たないため (infcxの状態は変化させない)、
    /// 計算結果をキャッシュすることが安全にできます。
    /// `DelayedMap` を使うことで、小さな型に対するキャッシュのオーバーヘッドを回避し、
    /// 巨大な型の場合にのみキャッシュを有効にするという最適化を行っています。
    cache: DelayedMap<Ty<'tcx>, Ty<'tcx>>,
}

impl<'a, 'tcx> OpportunisticVarResolver<'a, 'tcx> {
    pub fn new(infcx: &'a InferCtxt<'tcx>) -> Self {
        OpportunisticVarResolver {
            infcx,
            cache: DelayedMap::default(),
        }
    }
}


impl<'a, 'tcx> TypeFolder<'tcx> for OpportunisticVarResolver<'a, 'tcx> {
    fn tcx(&self) -> crate::stelaro_context::TyCtxt<'tcx> {
        self.infcx.tcx
    }

    #[inline]
    fn fold_ty(&mut self, t: Ty<'tcx>) -> Ty<'tcx> {
        if let Some(ty) = self.cache.get(&t) {
            *ty
        } else {
            todo!()
        }
    }
}